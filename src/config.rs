use anyhow::Context;

#[derive(Clone, Debug)]
pub struct AppConfig {
    // Server
    pub host: String,
    pub port: u16,
    pub environment: Environment,

    // Database
    pub database_url: String,
    pub database_max_connections: u32,

    // Redis
    pub redis_url: String,

    // Auth
    pub jwt_secret: String,
    pub jwt_expires_in_hours: i64,
    pub jwt_refresh_expires_in_days: i64,

    // Frontend
    pub frontend_url: String,

    // Stripe
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,

    // Zoom
    pub zoom_account_id: String,
    pub zoom_client_id: String,
    pub zoom_client_secret: String,

    // Email
    pub resend_api_key: String,
    pub email_from: String,

    // AI
    pub anthropic_api_key: String,
    pub anthropic_model: String,

    // R2 storage
    pub r2_account_id: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_bucket_name: String,
    pub r2_public_url: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8080".into())
                .parse()
                .context("PORT must be a number")?,
            environment: match std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".into())
                .as_str()
            {
                "production" => Environment::Production,
                _ => Environment::Development,
            },
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            database_max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".into())
                .parse()
                .context("DATABASE_MAX_CONNECTIONS must be a number")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            jwt_secret: std::env::var("JWT_SECRET")
                .context("JWT_SECRET must be set")?,
            jwt_expires_in_hours: std::env::var("JWT_EXPIRES_IN_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()
                .context("JWT_EXPIRES_IN_HOURS must be a number")?,
            jwt_refresh_expires_in_days: std::env::var("JWT_REFRESH_EXPIRES_IN_DAYS")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .context("JWT_REFRESH_EXPIRES_IN_DAYS must be a number")?,
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:4200".into()),
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY")
                .unwrap_or_default(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET")
                .unwrap_or_default(),
            zoom_account_id: std::env::var("ZOOM_ACCOUNT_ID").unwrap_or_default(),
            zoom_client_id: std::env::var("ZOOM_CLIENT_ID").unwrap_or_default(),
            zoom_client_secret: std::env::var("ZOOM_CLIENT_SECRET").unwrap_or_default(),
            resend_api_key: std::env::var("RESEND_API_KEY").unwrap_or_default(),
            email_from: std::env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "hello@indigo.dev".into()),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
            anthropic_model: std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-6".into()),
            r2_account_id: std::env::var("R2_ACCOUNT_ID").unwrap_or_default(),
            r2_access_key_id: std::env::var("R2_ACCESS_KEY_ID").unwrap_or_default(),
            r2_secret_access_key: std::env::var("R2_SECRET_ACCESS_KEY").unwrap_or_default(),
            r2_bucket_name: std::env::var("R2_BUCKET_NAME")
                .unwrap_or_else(|_| "indigo-assets".into()),
            r2_public_url: std::env::var("R2_PUBLIC_URL").unwrap_or_default(),
        })
    }

    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
}