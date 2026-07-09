use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db:     sqlx::PgPool,
    pub redis:  redis::aio::ConnectionManager,
    pub config: AppConfig,
}