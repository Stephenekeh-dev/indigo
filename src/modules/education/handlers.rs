use axum::{extract::{State, Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
    utils::slug::unique_slug,
};
use super::models::*;

#[derive(Deserialize)]
pub struct CourseQuery {
    pub level: Option<String>,
    pub page:  Option<i64>,
    pub limit: Option<i64>,
}

pub async fn list_courses(
    State(state): State<AppState>,
    Query(q): Query<CourseQuery>,
) -> IndigoResult<Json<Vec<Course>>> {
    let limit  = q.limit.unwrap_or(12).min(50);
    let offset = (q.page.unwrap_or(1) - 1) * limit;
    let rows = sqlx::query_as!(
        Course,
        r#"SELECT id, title, slug, description, short_desc,
                  level::text as "level!",
                  status::text as "status!",
                  price_usd::float8 as "price_usd!",
                  thumbnail_url, intro_video_url,
                  total_duration_mins, total_lessons, is_free, tags,
                  published_at, created_at, updated_at
           FROM courses
           WHERE status = 'published'
           ORDER BY published_at DESC
           LIMIT $1 OFFSET $2"#,
        limit, offset
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn get_course(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> IndigoResult<Json<Course>> {
    sqlx::query_as!(
        Course,
        r#"SELECT id, title, slug, description, short_desc,
                  level::text as "level!",
                  status::text as "status!",
                  price_usd::float8 as "price_usd!",
                  thumbnail_url, intro_video_url,
                  total_duration_mins, total_lessons, is_free, tags,
                  published_at, created_at, updated_at
           FROM courses
           WHERE slug = $1 AND status = 'published'"#,
        slug
    )
    .fetch_optional(&state.db)
    .await?
    .map(Json)
    .ok_or_else(|| IndigoError::NotFound("Course".into()))
}

pub async fn create_course(
    _claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<CreateCourseDto>,
) -> IndigoResult<Json<Course>> {
    let id      = Uuid::new_v4();
    let slug    = unique_slug(&dto.title, &id);
    let tags    = dto.tags.unwrap_or_default();
    let is_free = dto.is_free.unwrap_or(false);
    let row = sqlx::query_as!(
        Course,
        r#"INSERT INTO courses
              (id, title, slug, description, level, price_usd, is_free, tags)
           VALUES ($1,$2,$3,$4,$5::text::course_level,$6::float8,$7,$8)
           RETURNING id, title, slug, description, short_desc,
                     level::text as "level!",
                     status::text as "status!",
                     price_usd::float8 as "price_usd!",
                     thumbnail_url, intro_video_url,
                     total_duration_mins, total_lessons, is_free, tags,
                     published_at, created_at, updated_at"#,
        id, dto.title, slug, dto.description, dto.level,
        dto.price_usd, is_free, &tags
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn enroll(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<EnrollDto>,
) -> IndigoResult<Json<Enrollment>> {
    let existing = sqlx::query_scalar!(
        "SELECT id FROM enrollments WHERE user_id = $1 AND course_id = $2",
        claims.sub, dto.course_id
    )
    .fetch_optional(&state.db)
    .await?;

    if existing.is_some() {
        return Err(IndigoError::Conflict(
            "Already enrolled in this course".into()
        ));
    }

    let id = Uuid::new_v4();
    let row = sqlx::query_as!(
        Enrollment,
        r#"INSERT INTO enrollments (id, user_id, course_id, stripe_payment_id)
           VALUES ($1,$2,$3,$4)
           RETURNING id, user_id, course_id,
                     status::text as "status!",
                     progress_pct::float8 as "progress_pct!",
                     amount_paid_usd::float8 as amount_paid_usd,
                     enrolled_at, completed_at"#,
        id, claims.sub, dto.course_id, dto.stripe_payment_id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn my_enrollments(
    claims: Claims,
    State(state): State<AppState>,
) -> IndigoResult<Json<Vec<Enrollment>>> {
    let rows = sqlx::query_as!(
        Enrollment,
        r#"SELECT id, user_id, course_id,
                  status::text as "status!",
                  progress_pct::float8 as "progress_pct!",
                  amount_paid_usd::float8 as amount_paid_usd,
                  enrolled_at, completed_at
           FROM enrollments
           WHERE user_id = $1
           ORDER BY enrolled_at DESC"#,
        claims.sub
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

pub async fn update_progress(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<UpdateProgressDto>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        r#"INSERT INTO lesson_progress
              (id, user_id, lesson_id, course_id, completed, watch_seconds)
           SELECT uuid_generate_v4(), $1, $2, course_id, $3, $4
           FROM lessons WHERE id = $2
           ON CONFLICT (user_id, lesson_id) DO UPDATE
           SET completed     = EXCLUDED.completed,
               watch_seconds = EXCLUDED.watch_seconds,
               updated_at    = NOW()"#,
        claims.sub, dto.lesson_id, dto.completed,
        dto.watch_seconds.unwrap_or(0)
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Progress updated" })))
}