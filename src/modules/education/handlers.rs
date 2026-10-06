use axum::{extract::{State, Path, Query}, Json};
use serde::Deserialize;
use uuid::Uuid;
use axum::extract::Multipart;
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
        return Err(IndigoError::Conflict("Already enrolled in this course".into()));
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

    // Send enrollment confirmation email
let course = sqlx::query!(
    "SELECT title, slug FROM courses WHERE id = $1",
    dto.course_id
)
.fetch_optional(&state.db)
.await?;

let user = sqlx::query!(
    "SELECT full_name, email FROM users WHERE id = $1",
    claims.sub
)
.fetch_optional(&state.db)
.await?;

if let (Some(c), Some(u)) = (course, user) {
    let course_url = format!(
        "{}/courses/{}/learn",
        state.config.frontend_url, c.slug
    );
    let _ = crate::utils::email::send_email_smtp(
        &state.config.mail_host,
        state.config.mail_port,
        &state.config.mail_username,
        &state.config.mail_password,
        &state.config.mail_username,
        crate::utils::email::EmailPayload {
            to:      u.email,
            subject: format!("You are enrolled in {} — Indigo", c.title),
            html:    crate::utils::email::enrollment_confirmation_email(
                &u.full_name,
                &c.title,
                &course_url,
            ),
        },
    ).await;

    }

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
pub async fn update_course(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<CreateCourseDto>,
) -> IndigoResult<Json<Course>> {
    let tags    = dto.tags.unwrap_or_default();
    let is_free = dto.is_free.unwrap_or(false);
    let row = sqlx::query_as!(
        Course,
        r#"UPDATE courses SET
              title      = $1,
              description = $2,
              level      = $3::text::course_level,
              price_usd  = $4::float8,
              is_free    = $5,
              tags       = $6,
              updated_at = NOW()
           WHERE id = $7
           RETURNING id, title, slug, description, short_desc,
                     level::text as "level!",
                     status::text as "status!",
                     price_usd::float8 as "price_usd!",
                     thumbnail_url, intro_video_url,
                     total_duration_mins, total_lessons, is_free, tags,
                     published_at, created_at, updated_at"#,
        dto.title, dto.description, dto.level,
        dto.price_usd, is_free, &tags, id
    )
    .fetch_one(&state.db)
    .await?;
    Ok(Json(row))
}

pub async fn delete_course(
    _claims: Claims,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> IndigoResult<Json<serde_json::Value>> {
    sqlx::query!(
        "DELETE FROM courses WHERE id = $1", id
    )
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({ "message": "Course deleted" })))
}

pub async fn upload_lesson_video(
    _claims: Claims,
    State(state): State<AppState>,
    Path(course_id): Path<String>,
    mut multipart: Multipart,
) -> IndigoResult<Json<serde_json::Value>> {
    let mut file_bytes    = Vec::new();
    let mut original_name = String::from("lesson.mp4");
    let mut lesson_title  = String::from("New Lesson");
    let mut lesson_order  = 1i32;

    while let Some(field) = multipart.next_field().await
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_owned();
        match name.as_str() {
            "video" => {
                original_name = field.file_name()
                    .unwrap_or("lesson.mp4")
                    .to_owned();
                file_bytes = field.bytes().await
                    .map_err(|e| IndigoError::Internal(anyhow::anyhow!("{}", e)))?
                    .to_vec();
            }
            "title" => {
                lesson_title = field.text().await
                    .map_err(|e| IndigoError::Internal(anyhow::anyhow!("{}", e)))?;
            }
            "sort_order" => {
                lesson_order = field.text().await
                    .unwrap_or_else(|_| "1".into())
                    .parse()
                    .unwrap_or(1);
            }
            _ => {}
        }
    }

    if file_bytes.is_empty() {
        return Err(IndigoError::Validation("No video file provided".into()));
    }

    let r2_cfg = crate::utils::storage::R2Config {
        account_id:        state.config.r2_account_id.clone(),
        access_key_id:     state.config.r2_access_key_id.clone(),
        secret_access_key: state.config.r2_secret_access_key.clone(),
        bucket_name:       state.config.r2_bucket_name.clone(),
        public_url:        state.config.r2_public_url.clone(),
    };

    let video_url = crate::utils::storage::upload_video(
        &r2_cfg,
        file_bytes,
        &original_name,
        &course_id,
    ).await?;

    // Save lesson to DB
    let lesson_id = uuid::Uuid::new_v4();
    sqlx::query!(
    r#"INSERT INTO lessons
          (id, course_id, title, sort_order, video_url, video_duration)
       VALUES ($1, $2::uuid, $3, $4, $5, 0)"#,
    lesson_id,
    course_id.parse::<uuid::Uuid>()
        .map_err(|_| IndigoError::Validation("Invalid course ID".into()))?,
    lesson_title,
    lesson_order,
    video_url
)
.execute(&state.db)
.await?;
    // Update course lesson count
    sqlx::query!(
    "UPDATE courses SET total_lessons = total_lessons + 1 WHERE id = $1::uuid",
    course_id.parse::<uuid::Uuid>().unwrap()
)
.execute(&state.db)
.await.ok();

    Ok(Json(serde_json::json!({
        "message":   "Lesson uploaded successfully",
        "video_url": video_url,
        "lesson_id": lesson_id,
    })))
}

pub async fn list_lessons(
    State(state): State<AppState>,
    Path(course_id): Path<String>,
) -> IndigoResult<Json<Vec<serde_json::Value>>> {
    let rows = sqlx::query!(
    r#"SELECT id, title, sort_order, video_url, video_duration, created_at
       FROM lessons
       WHERE course_id = $1::uuid
       ORDER BY sort_order ASC"#,
    course_id.parse::<uuid::Uuid>()
        .map_err(|_| IndigoError::Validation("Invalid course ID".into()))?
)
.fetch_all(&state.db)
.await?;

let lessons: Vec<serde_json::Value> = rows.iter().map(|r| serde_json::json!({
    "id":             r.id,
    "title":          r.title,
    "sort_order":     r.sort_order,
    "video_url":      r.video_url,
    "video_duration": r.video_duration,
})).collect();
    Ok(Json(lessons))
}