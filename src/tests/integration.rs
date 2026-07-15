#[cfg(test)]
mod auth_integration {
    use crate::tests::common::{test_state, cleanup, create_test_user};
    use crate::utils::hash::{hash_password, verify_password};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_user_in_db() {
        let state = test_state().await;
        let id    = Uuid::new_v4();
        let email = format!("test_{}@indigo.dev", id);
        let hash  = hash_password("password123").unwrap();

        let result = sqlx::query!(
            "INSERT INTO users (id, full_name, email, password_hash)
             VALUES ($1, 'Test User', $2, $3)
             RETURNING id",
            id, email, hash
        )
        .fetch_one(&state.db)
        .await;

        assert!(result.is_ok());
        cleanup(&state.db, vec![id]).await;
    }

    #[tokio::test]
    async fn test_duplicate_email_fails() {
        let state = test_state().await;
        let id1   = Uuid::new_v4();
        let id2   = Uuid::new_v4();
        let email = format!("duplicate_{}@indigo.dev", id1);
        let hash  = hash_password("password123").unwrap();

        // First insert succeeds
        sqlx::query!(
            "INSERT INTO users (id, full_name, email, password_hash)
             VALUES ($1, 'User One', $2, $3)",
            id1, email, hash
        )
        .execute(&state.db)
        .await
        .unwrap();

        // Second insert with same email fails
        let result = sqlx::query!(
            "INSERT INTO users (id, full_name, email, password_hash)
             VALUES ($1, 'User Two', $2, $3)",
            id2, email, hash
        )
        .execute(&state.db)
        .await;

        assert!(result.is_err());
        cleanup(&state.db, vec![id1]).await;
    }

    #[tokio::test]
    async fn test_fetch_user_by_email() {
        let state  = test_state().await;
        let email  = format!("fetch_{}@indigo.dev", Uuid::new_v4());
        let (id, _) = create_test_user(&state.db, &email, "password123").await;

        let user = sqlx::query!(
            "SELECT id, email FROM users WHERE email = $1", email
        )
        .fetch_optional(&state.db)
        .await
        .unwrap();

        assert!(user.is_some());
        assert_eq!(user.unwrap().email, email);
        cleanup(&state.db, vec![id]).await;
    }

    #[tokio::test]
    async fn test_password_hash_stored_correctly() {
        let state    = test_state().await;
        let email    = format!("hash_{}@indigo.dev", Uuid::new_v4());
        let password = "test_password_456";
        let (id, _)  = create_test_user(&state.db, &email, password).await;

        let row = sqlx::query!(
            "SELECT password_hash FROM users WHERE id = $1", id
        )
        .fetch_one(&state.db)
        .await
        .unwrap();

        assert!(verify_password(password, &row.password_hash).unwrap());
        cleanup(&state.db, vec![id]).await;
    }
}

#[cfg(test)]
mod services_integration {
    use crate::tests::common::test_state;
    use crate::utils::slug::unique_slug;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_service_listing() {
        let state = test_state().await;
        let id    = Uuid::new_v4();
        let slug  = unique_slug("Rust Migration Consulting", &id);

        let result = sqlx::query!(
            r#"INSERT INTO service_listings
                  (id, title, slug, description, service_type, price_usd)
               VALUES ($1, $2, $3, 'Full Rust migration service', 'migration'::service_type, 500.00)
               RETURNING id"#,
            id, "Rust Migration Consulting", slug
        )
        .fetch_one(&state.db)
        .await;

        assert!(result.is_ok());

        // Cleanup
        sqlx::query!("DELETE FROM service_listings WHERE id = $1", id)
            .execute(&state.db)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_list_active_services() {
        let state = test_state().await;

        let rows = sqlx::query!(
            "SELECT id FROM service_listings WHERE is_active = true"
        )
        .fetch_all(&state.db)
        .await;

        assert!(rows.is_ok());
    }
}

#[cfg(test)]
mod education_integration {
    use crate::tests::common::test_state;
    use crate::utils::slug::unique_slug;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_course() {
        let state = test_state().await;
        let id    = Uuid::new_v4();
        let slug  = unique_slug("Rust for Beginners", &id);

        let result = sqlx::query!(
            r#"INSERT INTO courses
                  (id, title, slug, description, level, price_usd)
               VALUES ($1, $2, $3, 'Learn Rust from scratch', 'beginner'::course_level, 49.99)
               RETURNING id"#,
            id, "Rust for Beginners", slug
        )
        .fetch_one(&state.db)
        .await;

        assert!(result.is_ok());

        sqlx::query!("DELETE FROM courses WHERE id = $1", id)
            .execute(&state.db)
            .await
            .ok();
    }

    #[tokio::test]
    async fn test_slug_unique_constraint() {
        let state = test_state().await;
        let id1   = Uuid::new_v4();
        let id2   = Uuid::new_v4();
        let slug  = format!("unique-slug-{}", Uuid::new_v4());

        sqlx::query!(
            r#"INSERT INTO courses (id, title, slug, description, level, price_usd)
               VALUES ($1, 'Course 1', $2, 'Desc', 'beginner'::course_level, 0)"#,
            id1, slug
        )
        .execute(&state.db)
        .await
        .unwrap();

        let result = sqlx::query!(
            r#"INSERT INTO courses (id, title, slug, description, level, price_usd)
               VALUES ($1, 'Course 2', $2, 'Desc', 'beginner'::course_level, 0)"#,
            id2, slug
        )
        .execute(&state.db)
        .await;

        assert!(result.is_err()); // Unique constraint violation

        sqlx::query!("DELETE FROM courses WHERE id = $1", id1)
            .execute(&state.db)
            .await
            .ok();
    }
}

#[cfg(test)]
mod media_integration {
    use crate::tests::common::{test_state, create_test_user, cleanup};
    use crate::utils::slug::unique_slug;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_blog_post() {
        let state  = test_state().await;
        let email  = format!("blog_{}@indigo.dev", Uuid::new_v4());
        let (user_id, _) = create_test_user(&state.db, &email, "password").await;

        let post_id = Uuid::new_v4();
        let slug    = unique_slug("My First Rust Post", &post_id);

        let result = sqlx::query!(
            r#"INSERT INTO posts
                  (id, author_id, title, slug, content, status, category)
               VALUES ($1, $2, $3, $4, 'Post content here',
                       'draft'::post_status, 'rust_basics'::post_category)
               RETURNING id"#,
            post_id, user_id, "My First Rust Post", slug
        )
        .fetch_one(&state.db)
        .await;

        assert!(result.is_ok());

        sqlx::query!("DELETE FROM posts WHERE id = $1", post_id)
            .execute(&state.db)
            .await
            .ok();
        cleanup(&state.db, vec![user_id]).await;
    }

    #[tokio::test]
    async fn test_newsletter_subscribe() {
        let state = test_state().await;
        let email = format!("newsletter_{}@indigo.dev", Uuid::new_v4());

        let result = sqlx::query!(
            "INSERT INTO newsletter_subscribers (id, email, confirm_token)
             VALUES (uuid_generate_v4(), $1, uuid_generate_v4()::text)
             RETURNING id",
            email
        )
        .fetch_one(&state.db)
        .await;

        assert!(result.is_ok());

        sqlx::query!(
            "DELETE FROM newsletter_subscribers WHERE email = $1", email
        )
        .execute(&state.db)
        .await
        .ok();
    }
}