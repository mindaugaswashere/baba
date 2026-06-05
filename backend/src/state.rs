//! Shared application state handed to every request.

use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    /// A valid-but-throwaway Argon2 hash, computed once at startup. Login
    /// verifies against this when the email is unknown so the response time is
    /// the same whether or not the account exists (mitigates user enumeration).
    pub dummy_hash: Arc<String>,
}
