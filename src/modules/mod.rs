pub mod auth;
pub mod services;
pub mod education;
pub mod blockchain;
pub mod community;
pub mod commerce;
pub mod media;
pub mod ai;

use axum::Router;
use crate::state;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1/auth",       auth::routes(state.clone()))
        .nest("/api/v1/services",   services::routes(state.clone()))
        .nest("/api/v1/education",  education::routes(state.clone()))
        .nest("/api/v1/blockchain", blockchain::routes(state.clone()))
        .nest("/api/v1/community",  community::routes(state.clone()))
        .nest("/api/v1/commerce",   commerce::routes(state.clone()))
        .nest("/api/v1/media",      media::routes(state.clone()))
        .nest("/api/v1/ai",         ai::routes(state.clone()))
        .with_state(state)
}