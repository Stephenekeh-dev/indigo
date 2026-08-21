use axum::{extract::{State, Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::{
        slug::unique_slug,
        tokens::generate_secure_token,
        email::EmailPayload,
    },
};
use super::models::*;

#[derive(Deserialize)]
pub struct PostQuery {
    pub category: Option<String>,
    pub tag:      Option<String>,
    pub page:     Option<i64>,
    pub limit:    Option<i64>,
}

pub async fn list_posts(
    State(state): State<AppState>,
    Query(q): Query<PostQuery>,
) -> IndigoResult<Json<Vec<Post>>> {
    let limit  = q.limit.unwrap_or(10).min(50);
    let offset = (q.page.unwrap_or(1) - 1) * limit;
    let rows = sqlx::query_as!(
        Post,
        r#"SELECT id, author_id, title, slug, excerpt, content,
                  status::text as "status!",
                  category::text as "category!",
                  cover_image_url, tags, read_time_mins,
                  view_count, likes_count, seo_title, seo_description,
                  published_at, created_at, updated_at
           FROM posts
           WHERE status = 'published'
           ORDER BY published_at DESC
           LIMIT $1 OFFSET $2"#,
        limit, offset
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_post(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<Post>> {
    sqlx::query!(
        "UPDATE posts SET view_count = view_count + 1 WHERE slug = $1",
        slug
    )
    .execute(&state.db)
    .await
    .ok();

    sqlx::query_as!(
        Post,
        r#"SELECT id, author_id, title, slug, excerpt, content,
                  status::text as "status!",
                  category::text as "category!",
                  cover_image_url, tags, read_time_mins,
                  view_count, likes_count, seo_title, seo_description,
                  published_at, created_at, updated_at
           FROM posts
           WHERE slug = $1 AND status = 'published'"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Post".into()))
}

pub async fn create_post(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreatePostDto>,
) -> IndigoResult<Json<Post>> {
    let id         = Uuid::new_v4();
    let slug       = unique_slug(&dto.title, &id);
    let tags       = dto.tags.unwrap_or_default();
    let status     = dto.status.unwrap_or_else(|| "draft".into());
    let word_count = dto.content.split_whitespace().count() as i32;
    let read_time  = (word_count / 200).max(1);
    let published_at = if status == "published" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let row = sqlx::query_as!(
        Post,
        r#"INSERT INTO posts
              (id, author_id, title, slug, excerpt, content,
               status, category, tags, read_time_mins,
               seo_title, seo_description, published_at)
           VALUES ($1,$2,$3,$4,$5,$6,
                   $7::text::post_status,$8::text::post_category,
                   $9,$10,$11,$12,$13)
           RETURNING id, author_id, title, slug, excerpt, content,
                     status::text as "status!",
                     category::text as "category!",
                     cover_image_url, tags, read_time_mins,
                     view_count, likes_count, seo_title, seo_description,
                     published_at, created_at, updated_at"#,
        id, claims.sub, dto.title, slug, dto.excerpt, dto.content,
        status, dto.category, &tags, read_time,
        dto.seo_title, dto.seo_description, published_at
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn like_post(
    claims: Claims,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<serde_json::Value>> {
    let post_id = sqlx::query_scalar!(
        "SELECT id FROM posts WHERE slug = $1", slug
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| IndigoError::NotFound("Post".into()))?;

    let inserted = sqlx::query!(
        "INSERT INTO post_likes (id, post_id, user_id)
         VALUES (uuid_generate_v4(), $1, $2)
         ON CONFLICT DO NOTHING",
        post_id, claims.sub
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if inserted > 0 {
        sqlx::query!(
            "UPDATE posts SET likes_count = likes_count + 1 WHERE id = $1",
            post_id
        )
        .execute(&state.db)
        .await?;
    }

    Ok(Json(serde_json::json!({ "message": "Post liked" })))
}

pub async fn subscribe_newsletter(
    State(state): State<AppState>,
    Json(dto): Json<SubscribeDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    let existing = sqlx::query_scalar!(
        "SELECT id FROM newsletter_subscribers WHERE email = $1",
        dto.email
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Ok(Json(serde_json::json!({ "message": "Already subscribed" })));
    }

    let confirm_token = crate::utils::tokens::generate_secure_token();
    sqlx::query!(
        "INSERT INTO newsletter_subscribers
            (id, email, full_name, confirm_token, source)
         VALUES (uuid_generate_v4(), $1, $2, $3, $4)",
        dto.email, dto.full_name, confirm_token, dto.source
    )
    .execute(&state.db)
    .await?;

    let confirm_link = format!(
        "{}/newsletter/confirm/{}",
        state.config.frontend_url, confirm_token
    );
    let name = dto.full_name.as_deref().unwrap_or("there");

    tracing::info!("Sending newsletter confirmation to {}", dto.email);
    match crate::utils::email::send_email_smtp(
        &state.config.mail_host,
        state.config.mail_port,
        &state.config.mail_username,
        &state.config.mail_password,
        &state.config.mail_username,
        crate::utils::email::EmailPayload {
            to:      dto.email.clone(),
            subject: "Confirm your Indigo newsletter subscription".into(),
            html:    crate::utils::email::newsletter_confirm_email(name, &confirm_link),
        },
    ).await {
        Ok(_)  => tracing::info!("Newsletter email sent to {}", dto.email),
        Err(e) => tracing::error!("Newsletter email FAILED: {:?}", e),
    }

    Ok(Json(serde_json::json!({
        "message": "Check your email to confirm your subscription"
    })))
}
pub async fn confirm_newsletter(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> IndigoResult<Json<serde_json::Value>> {
    let affected = sqlx::query!(
        "UPDATE newsletter_subscribers
         SET is_confirmed = true, confirm_token = NULL
         WHERE confirm_token = $1 AND is_confirmed = false",
        token
    )
    .execute(&state.db)
    .await?
    .rows_affected();

    if affected == 0 {
        return Err(IndigoError::NotFound("Token".into()));
    }
    Ok(Json(serde_json::json!({
        "message": "Subscription confirmed. Welcome to Indigo!"
    })))
}
pub async fn update_post(
    _claims: Claims,
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(dto): Json<CreatePostDto>,
) -> IndigoResult<Json<Post>> {
    let tags   = dto.tags.unwrap_or_default();
    let status = dto.status.unwrap_or_else(|| "draft".into());
    let published_at = if status == "published" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    let row = sqlx::query_as!(
        Post,
        r#"UPDATE posts SET
              title        = $1,
              excerpt      = $2,
              content      = $3,
              status       = $4::text::post_status,
              category     = $5::text::post_category,
              tags         = $6,
              published_at = COALESCE($7, published_at),
              updated_at   = NOW()
           WHERE slug = $8
           RETURNING id, author_id, title, slug, excerpt, content,
                     status::text as "status!",
                     category::text as "category!",
                     cover_image_url, tags, read_time_mins,
                     view_count, likes_count, seo_title, seo_description,
                     published_at, created_at, updated_at"#,
        dto.title, dto.excerpt, dto.content, status,
        dto.category, &tags, published_at, slug
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete_post(
    _claims: Claims,
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        "DELETE FROM posts WHERE slug = $1", slug
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Post deleted" })))
}

pub async fn list_all_posts(
    State(state): State<AppState>,
    Query(q): Query<PostQuery>,
) -> IndigoResult<Json<Vec<Post>>> {
    let limit  = q.limit.unwrap_or(20).min(100);
    let offset = (q.page.unwrap_or(1) - 1) * limit;
    let rows = sqlx::query_as!(
        Post,
        r#"SELECT id, author_id, title, slug, excerpt, content,
                  status::text as "status!",
                  category::text as "category!",
                  cover_image_url, tags, read_time_mins,
                  view_count, likes_count, seo_title, seo_description,
                  published_at, created_at, updated_at
           FROM posts
           ORDER BY created_at DESC
           LIMIT $1 OFFSET $2"#,
        limit, offset
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}