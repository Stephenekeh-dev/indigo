use jsonwebtoken::{
    decode, encode,
    DecodingKey, EncodingKey,
    Header, Validation, Algorithm,
};
use uuid::Uuid;
use chrono::Utc;
use crate::{
    errors::IndigoError,
    middleware::auth::{Claims, UserRole},
};

pub fn generate_jwt(
    user_id:          Uuid,
    email:            &str,
    role:             UserRole,
    secret:           &str,
    expires_in_hours: i64,
) -> Result<String, IndigoError> {
    let now = Utc::now().timestamp() as usize;
    let exp = (Utc::now() + chrono::Duration::hours(expires_in_hours))
        .timestamp() as usize;

    let claims = Claims {
        sub:   user_id,
        email: email.to_owned(),
        role,
        iat:   now,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| IndigoError::Internal(anyhow::anyhow!("JWT encode error: {}", e)))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, IndigoError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map(|d| d.claims)
    .map_err(|_| IndigoError::InvalidToken)
}

pub fn generate_secure_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..48)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}