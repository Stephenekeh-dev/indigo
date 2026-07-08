use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use bigdecimal::BigDecimal;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ServiceListing {
    pub id:             Uuid,
    pub title:          String,
    pub slug:           String,
    pub description:    String,
    pub short_desc:     Option<String>,
    pub service_type:   String,
    pub price_usd:      BigDecimal,
    pub duration_hours: Option<BigDecimal>,
    pub is_active:      bool,
    pub sort_order:     i32,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Booking {
    pub id:                Uuid,
    pub service_id:        Uuid,
    pub client_id:         Uuid,
    pub scheduled_at:      DateTime<Utc>,
    pub duration_minutes:  i32,
    pub status:            String,
    pub zoom_meeting_id:   Option<String>,
    pub zoom_join_url:     Option<String>,
    pub zoom_start_url:    Option<String>,
    pub client_notes:      Option<String>,
    pub consultant_notes:  Option<String>,
    pub amount_paid_usd:   Option<BigDecimal>,
    pub stripe_payment_id: Option<String>,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ClientProject {
    pub id:           Uuid,
    pub client_id:    Uuid,
    pub title:        String,
    pub description:  Option<String>,
    pub service_type: String,
    pub status:       String,
    pub budget_usd:   Option<BigDecimal>,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateServiceDto {
    #[validate(length(min = 3, max = 200))]
    pub title:          String,
    #[validate(length(min = 10))]
    pub description:    String,
    pub short_desc:     Option<String>,
    pub service_type:   String,
    pub price_usd:      f64,
    pub duration_hours: Option<f64>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBookingDto {
    pub service_id:   Uuid,
    pub scheduled_at: DateTime<Utc>,
    pub client_notes: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 3, max = 200))]
    pub title:        String,
    pub description:  Option<String>,
    pub service_type: String,
    pub budget_usd:   Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data:  Vec<T>,
    pub total: i64,
    pub page:  i64,
    pub limit: i64,
}