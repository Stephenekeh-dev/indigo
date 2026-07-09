use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id:               Uuid,
    pub title:            String,
    pub slug:             String,
    pub description:      String,
    pub event_type:       String,
    pub status:           String,
    pub is_online:        bool,
    pub is_free:          bool,
    pub price_usd:        Option<f64>,
    pub max_attendees:    Option<i32>,
    pub scheduled_at:     DateTime<Utc>,
    pub duration_minutes: i32,
    pub timezone:         String,
    pub zoom_join_url:    Option<String>,
    pub thumbnail_url:    Option<String>,
    pub tags:             Vec<String>,
    pub created_at:       DateTime<Utc>,
    pub updated_at:       DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Membership {
    pub id:                     Uuid,
    pub user_id:                Uuid,
    pub tier:                   String,
    pub status:                 String,
    pub stripe_subscription_id: Option<String>,
    pub current_period_end:     Option<DateTime<Utc>>,
    pub created_at:             DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEventDto {
    #[validate(length(min = 3, max = 200))]
    pub title:            String,
    pub description:      String,
    pub event_type:       String,
    pub scheduled_at:     DateTime<Utc>,
    pub duration_minutes: i32,
    pub is_free:          Option<bool>,
    pub price_usd:        Option<f64>,
    pub max_attendees:    Option<i32>,
    pub timezone:         Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterEventDto {
    pub event_id: Uuid,
}