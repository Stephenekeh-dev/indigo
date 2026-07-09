use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Course {
    pub id:                  Uuid,
    pub title:               String,
    pub slug:                String,
    pub description:         String,
    pub short_desc:          Option<String>,
    pub level:               String,
    pub status:              String,
    pub price_usd:           f64,
    pub thumbnail_url:       Option<String>,
    pub intro_video_url:     Option<String>,
    pub total_duration_mins: i32,
    pub total_lessons:       i32,
    pub is_free:             bool,
    pub tags:                Vec<String>,
    pub published_at:        Option<DateTime<Utc>>,
    pub created_at:          DateTime<Utc>,
    pub updated_at:          DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Lesson {
    pub id:             Uuid,
    pub section_id:     Uuid,
    pub course_id:      Uuid,
    pub title:          String,
    pub slug:           String,
    pub lesson_type:    String,
    pub content:        Option<String>,
    pub video_url:      Option<String>,
    pub video_duration: Option<i32>,
    pub is_preview:     bool,
    pub sort_order:     i32,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Enrollment {
    pub id:              Uuid,
    pub user_id:         Uuid,
    pub course_id:       Uuid,
    pub status:          String,
    pub progress_pct:    f64,
    pub amount_paid_usd: Option<f64>,
    pub enrolled_at:     DateTime<Utc>,
    pub completed_at:    Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCourseDto {
    #[validate(length(min = 3, max = 200))]
    pub title:       String,
    pub description: String,
    pub level:       String,
    pub price_usd:   f64,
    pub is_free:     Option<bool>,
    pub tags:        Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct EnrollDto {
    pub course_id:         Uuid,
    pub stripe_payment_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgressDto {
    pub lesson_id:     Uuid,
    pub watch_seconds: Option<i32>,
    pub completed:     bool,
}