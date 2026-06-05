//! JWT session tokens (HS256). The token is what we put inside the httpOnly
//! cookie; it carries the user id and email and is self-expiring via `exp`.

use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id
    pub email: String,
    pub iat: u64,
    pub exp: u64,
}

pub fn issue_token(
    secret: &str,
    user_id: Uuid,
    email: &str,
    ttl_seconds: i64,
) -> Result<String, AppError> {
    let now = get_current_timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + ttl_seconds.max(0) as u64,
    };
    encode(
        &Header::default(), // HS256
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("jwt encode: {e}")))
}

pub fn verify_token(secret: &str, token: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::new(Algorithm::HS256), // expiry is validated by default
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("not authenticated"))
}
