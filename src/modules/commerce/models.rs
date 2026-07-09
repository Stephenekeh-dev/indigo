use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id:            Uuid,
    pub title:         String,
    pub slug:          String,
    pub description:   String,
    pub short_desc:    Option<String>,
    pub product_type:  String,
    pub status:        String,
    pub price_usd:     f64,
    pub compare_price: Option<f64>,
    pub is_digital:    bool,
    pub download_url:  Option<String>,
    pub thumbnail_url: Option<String>,
    pub tags:          Vec<String>,
    pub stock_count:   Option<i32>,
    pub created_at:    DateTime<Utc>,
    pub updated_at:    DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id:                Uuid,
    pub user_id:           Uuid,
    pub status:            String,
    pub total_usd:         f64,
    pub stripe_payment_id: Option<String>,
    pub billing_email:     Option<String>,
    pub created_at:        DateTime<Utc>,
    pub updated_at:        DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CartItem {
    pub id:         Uuid,
    pub user_id:    Uuid,
    pub product_id: Uuid,
    pub quantity:   i32,
    pub added_at:   DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProductDto {
    #[validate(length(min = 3, max = 200))]
    pub title:        String,
    pub description:  String,
    pub short_desc:   Option<String>,
    pub product_type: String,
    pub price_usd:    f64,
    pub is_digital:   Option<bool>,
    pub tags:         Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AddToCartDto {
    pub product_id: Uuid,
    pub quantity:   Option<i32>,
}