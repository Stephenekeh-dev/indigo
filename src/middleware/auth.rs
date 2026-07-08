use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    RequestPartsExt,
};
use axum_extra::{
    headers::{Authorization, authorization::Bearer},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{AppState, errors::IndigoError};

/// JWT payload — embedded in every protected request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,          // user id
    pub email: String,
    pub role: UserRole,
    pub exp: usize,         // expiry (unix timestamp)
    pub iat: usize,         // issued at
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    User,
    Consultant,
    Admin,
}

/// Axum extractor — pulls and validates JWT from Authorization: Bearer header
/// Usage in handlers:  `async fn handler(claims: Claims, ...)`
#[axum::async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = IndigoError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Extract bearer token from header
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| IndigoError::Unauthorized)?;

        // Decode and validate JWT
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
            &Validation::new(Algorithm::HS256),
        )
        .map_err(|_| IndigoError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// Generate a JWT token for a user
pub fn generate_token(
    user_id: Uuid,
    email: &str,
    role: UserRole,
    secret: &str,
    expires_in_hours: i64,
) -> anyhow::Result<String> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use chrono::Utc;

    let now = Utc::now().timestamp() as usize;
    let exp = (Utc::now() + chrono::Duration::hours(expires_in_hours)).timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_owned(),
        role,
        iat: now,
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}