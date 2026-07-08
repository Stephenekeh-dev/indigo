use axum::Router;
use axum::routing::{get, post};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/session",          post(start_session))
        .route("/message",          post(send_message))
        .route("/history/:token",   get(get_history))
        .route("/session/end",      post(end_session))
        .with_state(state)
}