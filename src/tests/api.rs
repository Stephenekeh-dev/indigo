#[cfg(test)]
mod auth_api_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;
    use crate::{modules, tests::common::{test_state, cleanup}};
    use uuid::Uuid;

    async fn test_server() -> (TestServer, crate::state::AppState) {
        let state  = test_state().await;
        let app    = modules::routes(state.clone());
        let server = TestServer::new(app).unwrap();
        (server, state)
    }

    #[tokio::test]
    async fn test_register_success() {
        let (server, state) = test_server().await;
        let email = format!("api_{}@indigo.dev", Uuid::new_v4());

        let res = server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "Test User",
                "email":     email,
                "password":  "password123"
            }))
            .await;

        assert_eq!(res.status_code(), StatusCode::OK);

        let body: serde_json::Value = res.json();
        assert!(body["token"].as_str().is_some());
        assert_eq!(body["user"]["email"], email);

        // Cleanup
        if let Some(id) = body["user"]["id"].as_str() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cleanup(&state.db, vec![uuid]).await;
            }
        }
    }

    #[tokio::test]
    async fn test_register_duplicate_email() {
        let (server, state) = test_server().await;
        let email = format!("dup_{}@indigo.dev", Uuid::new_v4());

        // First registration
        let res1 = server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "User One",
                "email":     email,
                "password":  "password123"
            }))
            .await;
        assert_eq!(res1.status_code(), StatusCode::OK);

        // Second registration with same email
        let res2 = server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "User Two",
                "email":     email,
                "password":  "password456"
            }))
            .await;
        assert_eq!(res2.status_code(), StatusCode::CONFLICT);

        // Cleanup
        let body: serde_json::Value = res1.json();
        if let Some(id) = body["user"]["id"].as_str() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cleanup(&state.db, vec![uuid]).await;
            }
        }
    }

    #[tokio::test]
    async fn test_login_success() {
        let (server, state) = test_server().await;
        let email    = format!("login_{}@indigo.dev", Uuid::new_v4());
        let password = "password123";

        // Register first
        server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "Login Test",
                "email":     email,
                "password":  password
            }))
            .await;

        // Now login
        let res = server
            .post("/api/v1/auth/login")
            .json(&json!({
                "email":    email,
                "password": password
            }))
            .await;

        assert_eq!(res.status_code(), StatusCode::OK);
        let body: serde_json::Value = res.json();
        assert!(body["token"].as_str().is_some());

        // Cleanup
        if let Some(id) = body["user"]["id"].as_str() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cleanup(&state.db, vec![uuid]).await;
            }
        }
    }

    #[tokio::test]
    async fn test_login_wrong_password() {
        let (server, state) = test_server().await;
        let email = format!("wrong_{}@indigo.dev", Uuid::new_v4());

        // Register
        let reg = server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "Wrong Pass",
                "email":     email,
                "password":  "correct_password"
            }))
            .await;

        // Login with wrong password
        let res = server
            .post("/api/v1/auth/login")
            .json(&json!({
                "email":    email,
                "password": "wrong_password"
            }))
            .await;

        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);

        // Cleanup
        let body: serde_json::Value = reg.json();
        if let Some(id) = body["user"]["id"].as_str() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cleanup(&state.db, vec![uuid]).await;
            }
        }
    }

    #[tokio::test]
    async fn test_me_requires_auth() {
        let (server, _state) = test_server().await;

        let res = server.get("/api/v1/auth/me").await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_me_with_valid_token() {
        let (server, state) = test_server().await;
        let email = format!("me_{}@indigo.dev", Uuid::new_v4());

        let reg = server
            .post("/api/v1/auth/register")
            .json(&json!({
                "full_name": "Me Test",
                "email":     email,
                "password":  "password123"
            }))
            .await;

        let body: serde_json::Value = reg.json();
        let token = body["token"].as_str().unwrap();

        let res = server
            .get("/api/v1/auth/me")
            .add_header(
                axum::http::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap()
            )
            .await;

        assert_eq!(res.status_code(), StatusCode::OK);
        let me: serde_json::Value = res.json();
        assert_eq!(me["email"], email);

        if let Some(id) = body["user"]["id"].as_str() {
            if let Ok(uuid) = Uuid::parse_str(id) {
                cleanup(&state.db, vec![uuid]).await;
            }
        }
    }
}

#[cfg(test)]
mod services_api_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use serde_json::json;
    use crate::{modules, tests::common::test_state};

    async fn test_server() -> TestServer {
        let state = test_state().await;
        let app   = modules::routes(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_list_services_public() {
        let server = test_server().await;
        let res    = server.get("/api/v1/services").await;
        assert_eq!(res.status_code(), StatusCode::OK);
        let body: serde_json::Value = res.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn test_get_nonexistent_service() {
        let server = test_server().await;
        let res    = server.get("/api/v1/services/nonexistent-slug").await;
        assert_eq!(res.status_code(), StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod education_api_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use crate::{modules, tests::common::test_state};

    async fn test_server() -> TestServer {
        let state = test_state().await;
        let app   = modules::routes(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_list_courses_public() {
        let server = test_server().await;
        let res    = server.get("/api/v1/education").await;
        assert_eq!(res.status_code(), StatusCode::OK);
        let body: serde_json::Value = res.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn test_enroll_requires_auth() {
        let server = test_server().await;
        let res    = server
            .post("/api/v1/education/enroll")
            .json(&serde_json::json!({
                "course_id": "00000000-0000-0000-0000-000000000000"
            }))
            .await;
        assert_eq!(res.status_code(), StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod media_api_tests {
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use crate::{modules, tests::common::test_state};

    async fn test_server() -> TestServer {
        let state = test_state().await;
        let app   = modules::routes(state);
        TestServer::new(app).unwrap()
    }

    #[tokio::test]
    async fn test_list_posts_public() {
        let server = test_server().await;
        let res    = server.get("/api/v1/media/posts").await;
        assert_eq!(res.status_code(), StatusCode::OK);
        let body: serde_json::Value = res.json();
        assert!(body.is_array());
    }

    #[tokio::test]
    async fn test_subscribe_newsletter() {
        let server = test_server().await;
        let res    = server
            .post("/api/v1/media/newsletter/subscribe")
            .json(&serde_json::json!({
                "email":     "newsletter_test@indigo.dev",
                "full_name": "Newsletter Tester"
            }))
            .await;
        assert_eq!(res.status_code(), StatusCode::OK);
    }
}