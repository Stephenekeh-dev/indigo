use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use validator::Validate;
use bigdecimal::BigDecimal;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlockchainService {
    pub id:             Uuid,
    pub title:          String,
    pub slug:           String,
    pub description:    String,
    pub network:        String,
    pub project_type:   String,
    pub price_from_usd: Option<BigDecimal>,
    pub is_active:      bool,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BlockchainProject {
    pub id:           Uuid,
    pub client_id:    Uuid,
    pub title:        String,
    pub description:  Option<String>,
    pub network:      String,
    pub project_type: String,
    pub status:       String,
    pub budget_usd:   Option<BigDecimal>,
    pub created_at:   DateTime<Utc>,
    pub updated_at:   DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInquiryDto {
    #[validate(length(min = 2, max = 150))]
    pub name:         String,
    #[validate(email)]
    pub email:        String,
    pub company:      Option<String>,
    pub network:      Option<String>,
    pub project_type: Option<String>,
    #[validate(length(min = 20))]
    pub description:  String,
    pub budget_range: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectDto {
    #[validate(length(min = 3, max = 200))]
    pub title:        String,
    pub description:  Option<String>,
    pub network:      String,
    pub project_type: String,
    pub budget_usd:   Option<BigDecimal>,
}