use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use crate::errors::IndigoError;

pub fn hash_password(password: &str) -> Result<String, IndigoError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Hash error: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, IndigoError> {
    let parsed = PasswordHash::new(hash)
        .map_err(|e| IndigoError::Internal(anyhow::anyhow!("Hash parse error: {}", e)))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}