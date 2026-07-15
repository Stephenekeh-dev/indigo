use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiSession {
    pub id:            Uuid,
    pub user_id:       Option<Uuid>,
    pub session_token: String,
    pub context:       String,
    pub title:         Option<String>,
    pub message_count: i32,
    pub is_active:     bool,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiMessage {
    pub id:         Uuid,
    pub session_id: Uuid,
    pub role:       String,
    pub content:    String,
    pub tokens_used: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ── Request DTOs ───────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
pub struct StartSessionDto {
    pub context: Option<String>,  // "general" | "rust_help" | "course_support" etc.
}

#[derive(Debug, Deserialize, Validate)]
pub struct SendMessageDto {
    pub session_token: String,
    #[validate(length(min = 1, max = 4000))]
    pub message:       String,
}

// ── Response DTOs ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub session_token: String,
    pub context:       String,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub reply:        String,
    pub tokens_used:  i32,
    pub session_token: String,
}
// ── OpenAI API types ───────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OpenAiRequest {
    pub model:       String,
    pub messages:    Vec<OpenAiMessage>,
    pub max_tokens:  u32,
    pub temperature: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OpenAiMessage {
    pub role:    String,   // "system" | "user" | "assistant"
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<OpenAiChoice>,
    pub usage:   Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiChoice {
    pub message: OpenAiMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiUsage {
    pub prompt_tokens:     i32,
    pub completion_tokens: i32,
    pub total_tokens:      i32,
}