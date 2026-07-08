use axum::Router;
use axum::routing::{get, post, patch};
use crate::AppState;
use super::handlers::*;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/register",              post(register))
        .route("/login",                 post(login))
        .route("/logout",                post(logout))
        .route("/me",                    get(me).patch(update_profile))
        .route("/verify-email/:token",   get(verify_email))
        .route("/forgot-password",       post(forgot_password))
        .route("/reset-password",        post(reset_password))
        .with_state(state)
}