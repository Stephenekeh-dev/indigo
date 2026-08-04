use axum::Router;
use axum::routing::{get, post, put, delete};
use crate::state::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/",
            get(list_courses).post(create_course))
        .route("/enroll",
            post(enroll))
        .route("/my-courses",
            get(my_enrollments))
        .route("/progress",
            post(update_progress))
        .route("/:slug",
            get(get_course)
            .put(update_course)
            .delete(delete_course))
        .with_state(state)
}