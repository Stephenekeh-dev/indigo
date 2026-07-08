use axum::Router;
use axum::routing::{get, post};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/events",       get(list_events).post(create_event))
        .route("/events/:slug", get(get_event))
        .route("/register",     post(register_for_event))
        .route("/membership",   get(my_membership))
        .with_state(state)
}