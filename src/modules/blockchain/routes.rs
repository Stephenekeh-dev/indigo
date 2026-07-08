use axum::Router;
use axum::routing::{get, post};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/",         get(list_services))
        .route("/:slug",    get(get_service))
        .route("/inquiry",  post(submit_inquiry))
        .route("/projects", get(my_projects).post(create_project))
        .with_state(state)
}