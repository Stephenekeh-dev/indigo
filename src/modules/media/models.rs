use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Post {
    pub id:              Uuid,
    pub author_id:       Uuid,
    pub title:           String,
    pub slug:            String,
    pub excerpt:         Option<String>,
    pub content:         String,
    pub status:          String,
    pub category:        String,
    pub cover_image_url: Option<String>,
    pub tags:           Option<Vec<String>>,
    pub read_time_mins:  Option<i32>,
    pub view_count:      i32,
    pub likes_count:     i32,
    pub seo_title:       Option<String>,
    pub seo_description: Option<String>,
    pub published_at:    Option<DateTime<Utc>>,
    pub created_at:      DateTime<Utc>,
    pub updated_at:      DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NewsletterSubscriber {
    pub id:           Uuid,
    pub email:        String,
    pub full_name:    Option<String>,
    pub is_confirmed: bool,
    pub subscribed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePostDto {
    #[validate(length(min = 3, max = 300))]
    pub title:    String,
    pub content:  String,
    pub excerpt:  Option<String>,
    pub category: String,
    pub tags:     Option<Vec<String>>,
    pub status:   Option<String>,
    pub seo_title:       Option<String>,
    pub seo_description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SubscribeDto {
    #[validate(email)]
    pub email:     String,
    pub full_name: Option<String>,
    pub source:    Option<String>,
}