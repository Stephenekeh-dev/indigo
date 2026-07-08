mod config;
mod db;
mod errors;
mod middleware;
mod modules;
mod state;
mod utils;

use axum::http::Method;
use tower_http::{
    cors::{CorsLayer, Any},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::net::SocketAddr;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "indigo=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🦀 Starting Indigo backend...");

    let cfg = config::AppConfig::from_env()?;

    let db_pool = db::create_pool(&cfg.database_url, cfg.database_max_connections).await?;
    tracing::info!("✅ Connected to PostgreSQL");

    let redis_client = redis::Client::open(cfg.redis_url.clone())?;
    let redis = redis::aio::ConnectionManager::new(redis_client).await?;
    tracing::info!("✅ Connected to Redis");

    let state = AppState { db: db_pool, redis, config: cfg.clone() };

    let cors = CorsLayer::new()
        .allow_origin(cfg.frontend_url.parse::<axum::http::HeaderValue>()?)
        .allow_methods([
            Method::GET, Method::POST, Method::PUT,
            Method::PATCH, Method::DELETE, Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_credentials(false);

    let app = modules::routes(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new());

    let addr: SocketAddr = format!("{}:{}", cfg.host, cfg.port).parse()?;
    tracing::info!("🚀 Indigo listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}