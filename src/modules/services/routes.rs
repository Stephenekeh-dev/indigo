use axum::Router;
use axum::routing::{get, post, patch};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/",                      get(list_services).post(create_service))
        .route("/:slug",                 get(get_service))
        .route("/bookings",              get(list_my_bookings).post(create_booking))
        .route("/bookings/:id/cancel",   patch(cancel_booking))
        .route("/projects",              get(list_my_projects).post(create_project))
        .with_state(state)
}