//! Runtime configuration, loaded from environment variables (.env).

use std::fmt;

/// Minimum acceptable length for the JWT signing secret.
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_seconds: i64,
    pub bind_addr: String,
    pub cookie_name: String,
    pub cookie_secure: bool,
    pub cors_origin: String,
}

// Manual Debug so secrets never end up in logs, panics, or error output.
impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("database_url", &"***redacted***")
            .field("jwt_secret", &"***redacted***")
            .field("jwt_ttl_seconds", &self.jwt_ttl_seconds)
            .field("bind_addr", &self.bind_addr)
            .field("cookie_name", &self.cookie_name)
            .field("cookie_secure", &self.cookie_secure)
            .field("cors_origin", &self.cors_origin)
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        fn required(key: &str) -> Result<String, String> {
            std::env::var(key).map_err(|_| format!("missing required env var: {key}"))
        }
        fn optional(key: &str, default: &str) -> String {
            std::env::var(key)
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| default.to_string())
        }

        let jwt_secret = required("JWT_SECRET")?;
        if jwt_secret.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "JWT_SECRET is too short ({} chars); use at least {MIN_JWT_SECRET_LEN} \
                 (e.g. `openssl rand -hex 32`)",
                jwt_secret.len()
            ));
        }

        Ok(Self {
            database_url: required("DATABASE_URL")?,
            jwt_secret,
            jwt_ttl_seconds: optional("JWT_TTL_SECONDS", "604800")
                .split_whitespace() // tolerate trailing inline comments
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(604_800),
            bind_addr: optional("BIND_ADDR", "127.0.0.1:8080"),
            cookie_name: optional("COOKIE_NAME", "auth_token"),
            cookie_secure: optional("COOKIE_SECURE", "false")
                .split_whitespace()
                .next()
                .map(|v| v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            cors_origin: optional("CORS_ORIGIN", "http://127.0.0.1:5174"),
        })
    }
}
