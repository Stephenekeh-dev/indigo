use axum::Router;
use axum::routing::{get, post, put, delete};
use crate::state::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/posts",
            get(list_posts).post(create_post))
        .route("/posts/all",
            get(list_all_posts))
        .route("/posts/:slug",
            get(get_post)
            .put(update_post)
            .delete(delete_post))
        .route("/posts/:slug/like",
            post(like_post))
        .route("/newsletter/subscribe",
            post(subscribe_newsletter))
        .route("/newsletter/confirm/:token",
            get(confirm_newsletter))
        .with_state(state)
}