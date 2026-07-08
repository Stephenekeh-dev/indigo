use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// All possible Indigo errors — each maps to an HTTP status code
#[derive(Debug, Error)]
pub enum IndigoError {
    // ── Auth ───────────────────────────────────────────────────────────────
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired or invalid")]
    InvalidToken,

    #[error("Insufficient permissions")]
    Forbidden,

    #[error("Authentication required")]
    Unauthorized,

    // ── Resource ───────────────────────────────────────────────────────────
    #[error("{0} not found")]
    NotFound(String),

    #[error("{0} already exists")]
    Conflict(String),

    // ── Validation ─────────────────────────────────────────────────────────
    #[error("Validation error: {0}")]
    Validation(String),

    // ── External services ──────────────────────────────────────────────────
    #[error("Payment processing failed: {0}")]
    Payment(String),

    #[error("Email delivery failed: {0}")]
    Email(String),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Zoom API error: {0}")]
    Zoom(String),

    // ── Infrastructure ─────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error: {0}")]
    Cache(#[from] redis::RedisError),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

/// Convert IndigoError into an Axum HTTP response
impl IntoResponse for IndigoError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            IndigoError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            IndigoError::InvalidToken      => (StatusCode::UNAUTHORIZED, self.to_string()),
            IndigoError::Unauthorized      => (StatusCode::UNAUTHORIZED, self.to_string()),
            IndigoError::Forbidden         => (StatusCode::FORBIDDEN, self.to_string()),
            IndigoError::NotFound(_)       => (StatusCode::NOT_FOUND, self.to_string()),
            IndigoError::Conflict(_)       => (StatusCode::CONFLICT, self.to_string()),
            IndigoError::Validation(_)     => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            IndigoError::Payment(_)        => (StatusCode::PAYMENT_REQUIRED, self.to_string()),
            IndigoError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            IndigoError::Cache(e) => {
                tracing::error!("Cache error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".into())
            }
            _ => {
                tracing::error!("Internal error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        let body = Json(json!({
            "error": {
                "message": message,
                "status": status.as_u16()
            }
        }));

        (status, body).into_response()
    }
}

/// Shorthand result type used across all Indigo handlers
pub type IndigoResult<T> = Result<T, IndigoError>;