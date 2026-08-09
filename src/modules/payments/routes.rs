use axum::Router;
use axum::routing::{get, post};
use crate::state::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/initialize", post(initialize_payment))
        .route("/verify",     get(verify_payment))
        .route("/webhook",    post(paystack_webhook))
        .with_state(state)
}