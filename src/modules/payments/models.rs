use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InitializePaymentDto {
    pub email:      String,
    pub amount_usd: f64,
    pub order_type: String,   // "cart" | "course" | "booking"
    pub item_id:    Option<String>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaymentInitResponse {
    pub authorization_url: String,
    pub access_code:       String,
    pub reference:         String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyPaymentQuery {
    pub reference: String,
}

#[derive(Debug, Serialize)]
pub struct PaymentVerifyResponse {
    pub status:    String,
    pub reference: String,
    pub amount:    f64,
    pub email:     String,
    pub paid:      bool,
}

// Paystack API response shapes
#[derive(Debug, Deserialize)]
pub struct PaystackInitResponse {
    pub status:  bool,
    pub message: String,
    pub data:    Option<PaystackInitData>,
}

#[derive(Debug, Deserialize)]
pub struct PaystackInitData {
    pub authorization_url: String,
    pub access_code:       String,
    pub reference:         String,
}

#[derive(Debug, Deserialize)]
pub struct PaystackVerifyResponse {
    pub status:  bool,
    pub message: String,
    pub data:    Option<PaystackVerifyData>,
}

#[derive(Debug, Deserialize)]
pub struct PaystackVerifyData {
    pub status:    String,
    pub reference: String,
    pub amount:    i64,
    pub currency:  String,
    pub customer:  PaystackCustomer,
}

#[derive(Debug, Deserialize)]
pub struct PaystackCustomer {
    pub email: String,
}