#[cfg(test)]
mod hash_tests {
    use crate::utils::hash::{hash_password, verify_password};

    #[test]
    fn test_hash_password_produces_hash() {
        let hash = hash_password("my_secure_password").unwrap();
        assert!(!hash.is_empty());
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "correct_password_123";
        let hash     = hash_password(password).unwrap();
        let result   = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_wrong() {
        let hash   = hash_password("correct_password").unwrap();
        let result = verify_password("wrong_password", &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let hash1 = hash_password("password1").unwrap();
        let hash2 = hash_password("password1").unwrap();
        // Argon2 uses random salt — same password produces different hashes
        assert_ne!(hash1, hash2);
    }
}

#[cfg(test)]
mod slug_tests {
    use crate::utils::slug::{make_slug, unique_slug};
    use uuid::Uuid;

    #[test]
    fn test_make_slug_basic() {
        let slug = make_slug("Hello World");
        assert_eq!(slug, "hello-world");
    }

    #[test]
    fn test_make_slug_special_chars() {
        let slug = make_slug("Rust & Blockchain: A Guide!");
        assert_eq!(slug, "rust-blockchain-a-guide");
    }

    #[test]
    fn test_make_slug_multiple_spaces() {
        let slug = make_slug("  Hello   World  ");
        assert_eq!(slug, "hello-world");
    }

    #[test]
    fn test_unique_slug_contains_id() {
        let id   = Uuid::new_v4();
        let slug = unique_slug("My Course Title", &id);
        assert!(slug.starts_with("my-course-title-"));
        assert_eq!(slug.len(), "my-course-title-".len() + 8);
    }

    #[test]
    fn test_unique_slug_different_for_same_title() {
        let id1   = Uuid::new_v4();
        let id2   = Uuid::new_v4();
        let slug1 = unique_slug("Same Title", &id1);
        let slug2 = unique_slug("Same Title", &id2);
        assert_ne!(slug1, slug2);
    }
}

#[cfg(test)]
mod token_tests {
    use crate::utils::tokens::{generate_jwt, verify_jwt, generate_secure_token};
    use crate::middleware::auth::UserRole;
    use uuid::Uuid;

    const SECRET: &str = "test-secret-key";

    #[test]
    fn test_generate_and_verify_jwt() {
        let user_id = Uuid::new_v4();
        let email   = "test@indigo.dev";

        let token  = generate_jwt(user_id, email, UserRole::User, SECRET, 24).unwrap();
        let claims = verify_jwt(&token, SECRET).unwrap();

        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_jwt_wrong_secret_fails() {
        let user_id = Uuid::new_v4();
        let token   = generate_jwt(user_id, "test@test.com", UserRole::User, SECRET, 24).unwrap();
        let result  = verify_jwt(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_secure_token_length() {
        let token = generate_secure_token();
        // 48 bytes * 2 hex chars = 96 chars
        assert_eq!(token.len(), 96);
    }

    #[test]
    fn test_generate_secure_token_unique() {
        let token1 = generate_secure_token();
        let token2 = generate_secure_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_jwt_role_consultant() {
        let user_id = Uuid::new_v4();
        let token   = generate_jwt(user_id, "test@test.com", UserRole::Consultant, SECRET, 24).unwrap();
        let claims  = verify_jwt(&token, SECRET).unwrap();
        assert!(matches!(claims.role, UserRole::Consultant));
    }
}