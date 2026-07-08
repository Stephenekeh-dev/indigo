use axum::{extract::State, Json};
use uuid::Uuid;
use redis::AsyncCommands;
use crate::{
    AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
};
use super::models::*;

// ── System prompts per context ─────────────────────────────────

fn system_prompt(context: &str) -> String {
    let base = "You are the Indigo AI assistant — a specialist in the Rust programming language \
                and the Indigo platform. Indigo offers Rust consulting, courses, blockchain \
                development, community events, and a shop. Always be helpful, precise, and concise.";
    match context {
        "rust_help" => format!(
            "{} The user needs help with Rust code. Provide working code examples, \
             explain ownership, borrowing, lifetimes, and best practices clearly.",
            base
        ),
        "course_support" => format!(
            "{} The user is taking a course on the Indigo platform. Help them understand \
             the lesson content, answer questions about exercises, and encourage progress.",
            base
        ),
        "booking_inquiry" => format!(
            "{} Help the user understand Indigo's consulting services and guide them toward \
             booking the right session for their needs.",
            base
        ),
        "sales" => format!(
            "{} Help the user find the right Indigo product, course, or service. \
             Be helpful but never pushy.",
            base
        ),
        "enterprise_inquiry" => format!(
            "{} The user represents a company interested in Rust migration or blockchain \
             development. Gather their requirements professionally and suggest next steps.",
            base
        ),
        _ => base.to_string(),
    }
}

// ── Start a new session ────────────────────────────────────────

pub async fn start_session(
    State(state): State<AppState>,
    claims: Option<Claims>,
    Json(dto): Json<StartSessionDto>,
) -> IndigoResult<Json<SessionResponse>> {
    let context = dto.context.unwrap_or_else(|| "general".into());
    let session_token = crate::utils::tokens::generate_secure_token();
    let user_id = claims.as_ref().map(|c| c.sub);
    let id = Uuid::new_v4();

    // Save session to DB
    sqlx::query!(
        "INSERT INTO ai_sessions (id, user_id, session_token, context)
         VALUES ($1, $2, $3, $4::ai_session_context)",
        id, user_id, session_token, context
    )
    .execute(&state.db)
    .await?;

    // Initialise empty history in Redis (expires in 2 hours)
    let redis_key = format!("ai:session:{}", session_token);
    let mut conn = state.redis.clone();
    let _: () = conn
        .set_ex(&redis_key, serde_json::to_string(&Vec::<ClaudeMessage>::new()).unwrap(), 7200)
        .await
        .map_err(IndigoError::Cache)?;

    Ok(Json(SessionResponse { session_token, context }))
}

// ── Send a message and get a reply ────────────────────────────

pub async fn send_message(
    State(state): State<AppState>,
    Json(dto): Json<SendMessageDto>,
) -> IndigoResult<Json<MessageResponse>> {
    // 1. Load conversation history from Redis
    let redis_key = format!("ai:session:{}", dto.session_token);
    let mut conn  = state.redis.clone();

    let history_json: String = conn
        .get(&redis_key)
        .await
        .map_err(|_| IndigoError::NotFound("Session".into()))?;

    let mut history: Vec<ClaudeMessage> = serde_json::from_str(&history_json)
        .unwrap_or_default();

    // 2. Fetch session context from DB for the system prompt
    let session = sqlx::query!(
        "SELECT id, context::text as context, user_id
         FROM ai_sessions WHERE session_token = $1 AND is_active = true",
        dto.session_token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Session".into()))?;

    let system = system_prompt(session.context.as_deref().unwrap_or("general"));

    // 3. Append user message to history
    history.push(ClaudeMessage {
        role:    "user".into(),
        content: dto.message.clone(),
    });

    // 4. Call Claude API
    let request = ClaudeRequest {
        model:      state.config.anthropic_model.clone(),
        max_tokens: 1024,
        system,
        messages:   history.clone(),
    };

    let response = reqwest::Client::new()
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &state.config.anthropic_api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| IndigoError::AiService(e.to_string()))?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(IndigoError::AiService(format!("Claude API error: {}", err)));
    }

    let claude_res: ClaudeResponse = response
        .json()
        .await
        .map_err(|e| IndigoError::AiService(e.to_string()))?;

    let reply = claude_res
        .content
        .into_iter()
        .find_map(|c| c.text)
        .unwrap_or_else(|| "Sorry, I could not generate a response.".into());

    let tokens_used = claude_res
        .usage
        .map(|u| u.input_tokens + u.output_tokens)
        .unwrap_or(0);

    // 5. Append assistant reply to history and save back to Redis
    history.push(ClaudeMessage {
        role:    "assistant".into(),
        content: reply.clone(),
    });

    // Keep only last 20 messages to stay within context limits
    if history.len() > 20 {
        history = history[history.len() - 20..].to_vec();
    }

    let _: () = conn
        .set_ex(&redis_key, serde_json::to_string(&history).unwrap(), 7200)
        .await
        .map_err(IndigoError::Cache)?;

    // 6. Persist messages to DB
    let session_id = session.id;
    sqlx::query!(
        "INSERT INTO ai_messages (id, session_id, role, content, tokens_used, model)
         VALUES (uuid_generate_v4(), $1, 'user', $2, NULL, $3)",
        session_id, dto.message, state.config.anthropic_model
    )
    .execute(&state.db)
    .await?;

    sqlx::query!(
        "INSERT INTO ai_messages (id, session_id, role, content, tokens_used, model)
         VALUES (uuid_generate_v4(), $1, 'assistant', $2, $3, $4)",
        session_id, reply, tokens_used, state.config.anthropic_model
    )
    .execute(&state.db)
    .await?;

    // 7. Update session message count
    sqlx::query!(
        "UPDATE ai_sessions SET message_count = message_count + 2, updated_at = NOW()
         WHERE session_token = $1",
        dto.session_token
    )
    .execute(&state.db)
    .await?;

    Ok(Json(MessageResponse {
        reply,
        tokens_used,
        session_token: dto.session_token,
    }))
}

// ── Get session history ────────────────────────────────────────

pub async fn get_history(
    State(state): State<AppState>,
    axum::extract::Path(session_token): axum::extract::Path<String>,
) -> IndigoResult<Json<Vec<AiMessage>>> {
    let session = sqlx::query_scalar!(
        "SELECT id FROM ai_sessions WHERE session_token = $1",
        session_token
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Session".into()))?;

    let messages = sqlx::query_as!(
        AiMessage,
        r#"SELECT id, session_id, role::text as "role!", content,
                  tokens_used, created_at
           FROM ai_messages WHERE session_id = $1
           ORDER BY created_at ASC"#,
        session
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(messages))
}

// ── End a session ─────────────────────────────────────────────

pub async fn end_session(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> IndigoResult<Json<serde_json::Value>> {
    let token = body["session_token"]
        .as_str()
        .ok_or_else(|| IndigoError::Validation("session_token required".into()))?;

    sqlx::query!(
        "UPDATE ai_sessions SET is_active = false WHERE session_token = $1",
        token
    )
    .execute(&state.db)
    .await?;

    let mut conn = state.redis.clone();
    let _: () = conn
        .del(format!("ai:session:{}", token))
        .await
        .map_err(IndigoError::Cache)?;

    Ok(Json(serde_json::json!({ "message": "Session ended" })))
}