use axum::{extract::{State, Query}, Json};
use crate::{
    state::AppState,
    errors::{IndigoError, IndigoResult},
    middleware::auth::Claims,
};
use super::models::*;

pub async fn initialize_payment(
    claims: Claims,
    State(state): State<AppState>,
    Json(dto): Json<InitializePaymentDto>,
) -> IndigoResult<Json<PaymentInitResponse>> {

    // Convert USD to kobo (Paystack uses smallest currency unit)
    // 1 USD ≈ 1600 NGN, 1 NGN = 100 kobo
    let amount_kobo = (dto.amount_usd * 1600.0 * 100.0) as i64;

    let callback_url = dto.callback_url.unwrap_or_else(|| {
        format!("{}/shop/checkout/success", state.config.frontend_url)
    });

    let body = serde_json::json!({
        "email":        dto.email,
        "amount":       amount_kobo,
        "currency":     "NGN",
        "callback_url": callback_url,
        "metadata": {
            "order_type": dto.order_type,
            "item_id":    dto.item_id,
            "user_id":    claims.sub.to_string(),
        }
    });

    let response = reqwest::Client::new()
        .post("https://api.paystack.co/transaction/initialize")
        .header("Authorization", format!("Bearer {}", state.config.paystack_secret_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Paystack error: {}", e)))?;

    let ps: PaystackInitResponse = response
        .json()
        .await
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Paystack parse error: {}", e)))?;

    let data = ps.data
        .ok_or_else(|| IndigoError::Internal(anyhow::anyhow!("No data from Paystack: {}", ps.message)))?;

    Ok(Json(PaymentInitResponse {
        authorization_url: data.authorization_url,
        access_code:       data.access_code,
        reference:         data.reference,
    }))
}

pub async fn verify_payment(
    _claims: Claims,
    State(state): State<AppState>,
    Query(q): Query<VerifyPaymentQuery>,
) -> IndigoResult<Json<PaymentVerifyResponse>> {

    let response = reqwest::Client::new()
        .get(format!("https://api.paystack.co/transaction/verify/{}", q.reference))
        .header("Authorization", format!("Bearer {}", state.config.paystack_secret_key))
        .send()
        .await
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Paystack verify error: {}", e)))?;

    let ps: PaystackVerifyResponse = response
        .json()
        .await
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Paystack parse error: {}", e)))?;

    let data = ps.data
        .ok_or_else(|| IndigoError::Internal(anyhow::anyhow!("No data from Paystack")))?;

    let paid      = data.status == "success";
    let amount_usd = data.amount as f64 / 160000.0;

    if paid {
        // Create order in database
        sqlx::query!(
            r#"INSERT INTO orders (id, user_id, status, total_usd, stripe_payment_id)
               SELECT uuid_generate_v4(), id, 'paid', $1::float8, $2
               FROM users WHERE email = $3
               ON CONFLICT DO NOTHING"#,
            amount_usd,
            data.reference,
            data.customer.email
        )
        .execute(&state.db)
        .await?;

        // Send order confirmation email
        let user = sqlx::query!(
    "SELECT full_name FROM users WHERE email = $1",
    data.customer.email
)
.fetch_optional(&state.db)
.await?;

if let Some(u) = user {
    let amount_str = format!("${:.2}", amount_usd);
    let _ = crate::utils::email::send_email_smtp(
        &state.config.mail_host,
        state.config.mail_port,
        &state.config.mail_username,
        &state.config.mail_password,
        &state.config.mail_username,
        crate::utils::email::EmailPayload {
            to:      data.customer.email.clone(),
            subject: format!("Order Confirmed — Ref: {}", data.reference),
            html:    crate::utils::email::order_confirmation_email(
                &u.full_name,
                &data.reference,
                &amount_str,
                "Digital products from Indigo Shop",
            ),
        },
    ).await;
}
    }

    Ok(Json(PaymentVerifyResponse {
        status:    data.status,
        reference: data.reference,
        amount:    amount_usd,
        email:     data.customer.email,
        paid,
    }))
}

pub async fn paystack_webhook(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> IndigoResult<Json<serde_json::Value>> {

    let event = body["event"].as_str().unwrap_or("");

    if event == "charge.success" {
        let data      = &body["data"];
        let reference = data["reference"].as_str().unwrap_or("");
        let email     = data["customer"]["email"].as_str().unwrap_or("");
        let amount    = data["amount"].as_i64().unwrap_or(0);
        let amount_usd = amount as f64 / 160000.0;

        if !reference.is_empty() && !email.is_empty() {
            // 1. Insert order ends here after .await?;
            sqlx::query!(
                r#"INSERT INTO orders (id, user_id, status, total_usd, stripe_payment_id)
                   SELECT uuid_generate_v4(), id, 'paid', $1::float8, $2
                   FROM users WHERE email = $3
                   ON CONFLICT DO NOTHING"#,
                amount_usd,
                reference,
                email
            )
            .execute(&state.db)
            .await?;

            // 2. Your email logic starts here
            let user = sqlx::query!(
                "SELECT full_name FROM users WHERE email = $1", email
            )
            .fetch_optional(&state.db)
            .await?;

            if let Some(u) = user {
                let amount_str = format!("${:.2}", amount_usd);
                let _ = crate::utils::email::send_email_smtp(
                    &state.config.mail_host,
                    state.config.mail_port,
                    &state.config.mail_username,
                    &state.config.mail_password,
                    &state.config.mail_username,
                    crate::utils::email::EmailPayload {
                        to:      email.to_string(),
                        subject: format!("Payment Received — Ref: {}", reference),
                        html:    crate::utils::email::order_confirmation_email(
                            &u.full_name,
                            reference,
                            &amount_str,
                            "Indigo Shop purchase",
                        ),
                    },
                ).await;
            }
        }
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
