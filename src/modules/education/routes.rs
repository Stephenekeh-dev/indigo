use axum::Router;
use axum::routing::{get, post};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/",            get(list_courses).post(create_course))
        .route("/:slug",       get(get_course))
        .route("/enroll",      post(enroll))
        .route("/my-courses",  get(my_enrollments))
        .route("/progress",    post(update_progress))
        .with_state(state)
}