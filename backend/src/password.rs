//! Password hashing with Argon2id (current OWASP-recommended default).
//!
//! These are CPU-bound; callers run them on a blocking thread pool so they
//! don't stall the async runtime.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};

use crate::error::AppError;

/// Hash a plaintext password into an Argon2id PHC string (salt + params included).
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AppError::Internal(format!("password hash: {e}")))
}

/// Verify a plaintext password against a stored PHC hash. Returns false on any
/// mismatch or malformed hash (never panics, never errors out).
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}
