use sqlx::PgPool;
use crate::{
    config::AppConfig,
    state::AppState,
};

/// Spin up a test AppState using a real test database
pub async fn test_state() -> AppState {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env()
        .expect("Failed to load test config");

    let db = sqlx::PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations on test DB
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");
    let redis = redis::aio::ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    AppState { db, redis, config }
}

/// Clean up test data after each test
pub async fn cleanup(pool: &PgPool, user_ids: Vec<uuid::Uuid>) {
    for id in user_ids {
        sqlx::query!("DELETE FROM users WHERE id = $1", id)
            .execute(pool)
            .await
            .ok();
    }
}

/// Create a test user and return (user_id, token)
pub async fn create_test_user(
    pool: &PgPool,
    email: &str,
    password: &str,
) -> (uuid::Uuid, String) {
    use crate::utils::{hash::hash_password, tokens::generate_jwt};
    use crate::middleware::auth::UserRole;

    let id           = uuid::Uuid::new_v4();
    let password_hash = hash_password(password).expect("Failed to hash password");

    sqlx::query!(
        "INSERT INTO users (id, full_name, email, password_hash, status, email_verified)
         VALUES ($1, 'Test User', $2, $3, 'active', true)",
        id, email, password_hash
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    let token = generate_jwt(
        id, email, UserRole::User,
        "test-secret", 24
    )
    .expect("Failed to generate test JWT");

    (id, token)
}