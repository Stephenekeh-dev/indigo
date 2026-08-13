use axum::{extract::State, Json};
use chrono::Utc;
use uuid::Uuid;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::{Claims, UserRole},
    utils::{
        hash::{hash_password, verify_password},
        tokens::{generate_jwt, generate_secure_token},
        email::{send_email, EmailPayload, verification_email, password_reset_email},
    },
};
use super::models::*;

pub async fn register(
    State(state): State<AppState>,
    Json(dto): Json<RegisterDto>,
) -> IndigoResult<Json<AuthResponse>> {
    let exists = sqlx::query_scalar!(
        "SELECT id FROM users WHERE email = $1",
        dto.email.to_lowercase()
    )
    .fetch_optional(&state.db)
    .await?;

    if exists.is_some() {
        return Err(IndigoError::Conflict("Email already registered".into()));
    }

    let id            = Uuid::new_v4();
    let password_hash = hash_password(&dto.password)?;

    let user = sqlx::query_as!(
        User,
        r#"INSERT INTO users (id, full_name, email, password_hash)
           VALUES ($1, $2, $3, $4)
           RETURNING id, full_name, email,
                     role::text as "role!",
                     status::text as "status!",
                     avatar_url, bio, github_url, linkedin_url,
                     website_url, timezone, email_verified,
                     created_at, updated_at"#,
        id, dto.full_name, dto.email.to_lowercase(), password_hash
    )
    .fetch_one(&state.db)
    .await?;

    let verify_token = generate_secure_token();
    let expires_at   = Utc::now() + chrono::Duration::hours(24);
    sqlx::query!(
        "INSERT INTO email_verification_tokens (id, user_id, token, expires_at)
         VALUES (uuid_generate_v4(), $1, $2, $3)",
        id, verify_token, expires_at
    )
    .execute(&state.db)
    .await?;

    let verify_link = format!(
        "{}/auth/verify-email/{}",
        state.config.frontend_url, verify_token
    );
    let _ = crate::utils::email::send_email_smtp(
    &state.config.mail_host,
    state.config.mail_port,
    &state.config.mail_username,
    &state.config.mail_password,
    &state.config.mail_username,
    crate::utils::email::EmailPayload {
        to:      user.email.clone(),
        subject: "Welcome to Indigo — please verify your email".into(),
        html:    crate::utils::email::verification_email(&user.full_name, &verify_link),
    },
).await;

    let token = generate_jwt(
        user.id, &user.email, UserRole::User,
        &state.config.jwt_secret,
        state.config.jwt_expires_in_hours,
    )?;
    let refresh_token   = generate_secure_token();
    let refresh_expires = Utc::now()
        + chrono::Duration::days(state.config.jwt_refresh_expires_in_days);

    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
         VALUES (uuid_generate_v4(), $1, $2, $3)",
        user.id, refresh_token, refresh_expires
    )
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse { token, refresh_token, user: user.into() }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> IndigoResult<Json<AuthResponse>> {
    let row = sqlx::query!(
        r#"SELECT id, full_name, email, password_hash,
                  role::text as role, status::text as status,
                  avatar_url, email_verified
           FROM users WHERE email = $1"#,
        dto.email.to_lowercase()
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(IndigoError::InvalidCredentials)?;

    if !verify_password(&dto.password, &row.password_hash)? {
        return Err(IndigoError::InvalidCredentials);
    }

    if row.status.as_deref() == Some("suspended") {
        return Err(IndigoError::Forbidden);
    }

    let role = match row.role.as_deref() {
        Some("admin")      => UserRole::Admin,
        Some("consultant") => UserRole::Consultant,
        _                  => UserRole::User,
    };

    let token = generate_jwt(
        row.id, &row.email, role,
        &state.config.jwt_secret,
        state.config.jwt_expires_in_hours,
    )?;

    let refresh_token   = generate_secure_token();
    let refresh_expires = Utc::now()
        + chrono::Duration::days(state.config.jwt_refresh_expires_in_days);
    sqlx::query!(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
         VALUES (uuid_generate_v4(), $1, $2, $3)",
        row.id, refresh_token, refresh_expires
    )
    .execute(&state.db)
    .await?;

    Ok(Json(AuthResponse {
        token,
        refresh_token,
        user: UserDto {
            id:             row.id,
            full_name:      row.full_name,
            email:          row.email,
            role:           row.role.unwrap_or_else(|| "user".into()),
            avatar_url:     row.avatar_url,
            email_verified: row.email_verified,
            created_at:     Utc::now(),
        },
    }))
}

pub async fn me(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<UserDto>> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, full_name, email,
                  role::text as "role!",
                  status::text as "status!",
                  avatar_url, bio, github_url, linkedin_url,
                  website_url, timezone, email_verified,
                  created_at, updated_at
           FROM users WHERE id = $1"#,
        claims.sub
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("User".into()))?;

    Ok(Json(user.into()))
}

pub async fn update_profile(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<UpdateProfileDto>,
) -> IndigoResult<Json<UserDto>> {
    let user = sqlx::query_as!(
        User,
        r#"UPDATE users SET
                full_name    = COALESCE($1, full_name),
                bio          = COALESCE($2, bio),
                github_url   = COALESCE($3, github_url),
                linkedin_url = COALESCE($4, linkedin_url),
                website_url  = COALESCE($5, website_url),
                timezone     = COALESCE($6, timezone),
                updated_at   = NOW()
           WHERE id = $7
           RETURNING id, full_name, email,
                     role::text as "role!",
                     status::text as "status!",
                     avatar_url, bio, github_url, linkedin_url,
                     website_url, timezone, email_verified,
                     created_at, updated_at"#,
        dto.full_name, dto.bio, dto.github_url,
        dto.linkedin_url, dto.website_url, dto.timezone,
        claims.sub
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(user.into()))
}

pub async fn verify_email(
    State(state): State<AppState>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> IndigoResult<Json<serde_json::Value>> {
    let row = sqlx::query!(
        "SELECT user_id, expires_at, used_at
         FROM email_verification_tokens WHERE token = $1",
        token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Token".into()))?;

    if row.used_at.is_some() {
        return Err(IndigoError::Validation("Token already used".into()));
    }
    if row.expires_at < Utc::now() {
        return Err(IndigoError::Validation("Token expired".into()));
    }

    sqlx::query!(
        "UPDATE email_verification_tokens SET used_at = NOW() WHERE token = $1",
        token
    )
    .execute(&state.db)
    .await?;

    sqlx::query!(
        "UPDATE users SET email_verified = true, status = 'active' WHERE id = $1",
        row.user_id
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Email verified successfully" })))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(dto): Json<ForgotPasswordDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    let user = sqlx::query!(
        "SELECT id, full_name FROM users WHERE email = $1",
        dto.email.to_lowercase()
    )
    .fetch_optional(&state.db)
    .await?;

    if let Some(u) = user {
        let reset_token = generate_secure_token();
        let expires_at  = Utc::now() + chrono::Duration::hours(1);
        sqlx::query!(
            "INSERT INTO password_reset_tokens (id, user_id, token, expires_at)
             VALUES (uuid_generate_v4(), $1, $2, $3)",
            u.id, reset_token, expires_at
        )
        .execute(&state.db)
        .await?;

        let reset_link = format!(
            "{}/auth/reset-password/{}",
            state.config.frontend_url, reset_token
        );
        let _ = crate::utils::email::send_email_smtp(
    &state.config.mail_host,
    state.config.mail_port,
    &state.config.mail_username,
    &state.config.mail_password,
    &state.config.mail_username,
    crate::utils::email::EmailPayload {
        to:      dto.email.clone(),
        subject: "Reset your Indigo password".into(),
        html:    crate::utils::email::password_reset_email(&u.full_name, &reset_link),
    },
).await;
    }

    Ok(Json(serde_json::json!({
        "message": "If that email exists, a reset link has been sent"
    })))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(dto): Json<ResetPasswordDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    let row = sqlx::query!(
        "SELECT user_id, expires_at, used_at
         FROM password_reset_tokens WHERE token = $1",
        dto.token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Token".into()))?;

    if row.used_at.is_some() {
        return Err(IndigoError::Validation("Token already used".into()));
    }
    if row.expires_at < Utc::now() {
        return Err(IndigoError::Validation("Token expired".into()));
    }

    let new_hash = hash_password(&dto.password)?;
    sqlx::query!(
        "UPDATE users SET password_hash = $1 WHERE id = $2",
        new_hash, row.user_id
    )
    .execute(&state.db)
    .await?;

    sqlx::query!(
        "UPDATE password_reset_tokens SET used_at = NOW() WHERE token = $1",
        dto.token
    )
    .execute(&state.db)
    .await?;

    Ok(Json(serde_json::json!({ "message": "Password reset successfully" })))
}

pub async fn logout(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> IndigoResult<Json<serde_json::Value>> {
    if let Some(refresh_token) = body["refresh_token"].as_str() {
        sqlx::query!(
            "UPDATE refresh_tokens SET revoked_at = NOW()
             WHERE token_hash = $1 AND revoked_at IS NULL",
            refresh_token
        )
        .execute(&state.db)
        .await?;
    }
    Ok(Json(serde_json::json!({ "message": "Logged out successfully" })))
}