use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

// ── DB row types ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id:             Uuid,
    pub full_name:      String,
    pub email:          String,
    pub role:           String,
    pub status:         String,
    pub avatar_url:     Option<String>,
    pub bio:            Option<String>,
    pub github_url:     Option<String>,
    pub linkedin_url:   Option<String>,
    pub website_url:    Option<String>,
    pub timezone:       Option<String>,
    pub email_verified: bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

// ── Request DTOs ───────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterDto {
    #[validate(length(min = 2, max = 150))]
    pub full_name: String,
    #[validate(email)]
    pub email:     String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password:  String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email)]
    pub email:    String,
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordDto {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordDto {
    pub token:    String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProfileDto {
    pub full_name:    Option<String>,
    pub bio:          Option<String>,
    pub github_url:   Option<String>,
    pub linkedin_url: Option<String>,
    pub website_url:  Option<String>,
    pub timezone:     Option<String>,
}

// ── Response DTOs ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token:        String,
    pub refresh_token: String,
    pub user:         UserDto,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id:             Uuid,
    pub full_name:      String,
    pub email:          String,
    pub role:           String,
    pub avatar_url:     Option<String>,
    pub email_verified: bool,
    pub created_at:     DateTime<Utc>,
}

impl From<User> for UserDto {
    fn from(u: User) -> Self {
        Self {
            id:             u.id,
            full_name:      u.full_name,
            email:          u.email,
            role:           u.role,
            avatar_url:     u.avatar_url,
            email_verified: u.email_verified,
            created_at:     u.created_at,
        }
    }
}