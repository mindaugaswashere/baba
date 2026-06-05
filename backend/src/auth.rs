//! Auth HTTP handlers, the `AuthUser` extractor, and cookie helpers.

use axum::{
    extract::{FromRequestParts, State},
    http::{request::Parts, StatusCode},
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{AuthResponse, Credentials, User};
use crate::state::AppState;
use crate::{password, token};

const MIN_PASSWORD_LEN: usize = 8;
const MAX_PASSWORD_LEN: usize = 128;

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Trim + lowercase the email and do a light structural check. We deliberately
/// keep this conservative rather than trying to fully validate RFC 5322.
fn normalize_email(raw: &str) -> Result<String, AppError> {
    let email = raw.trim().to_lowercase();
    let looks_valid = email.len() <= 254
        && email
            .split_once('@')
            .map(|(local, domain)| {
                !local.is_empty()
                    && domain.contains('.')
                    && !domain.starts_with('.')
                    && !domain.ends_with('.')
                    && !domain.contains("..")
            })
            .unwrap_or(false);

    if !looks_valid {
        return Err(AppError::Validation(
            "please enter a valid email address".into(),
        ));
    }
    Ok(email)
}

fn validate_password(password: &str) -> Result<(), AppError> {
    let len = password.chars().count();
    if len < MIN_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "password must be at least {MIN_PASSWORD_LEN} characters"
        )));
    }
    if len > MAX_PASSWORD_LEN {
        return Err(AppError::Validation(format!(
            "password must be at most {MAX_PASSWORD_LEN} characters"
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Cookie helpers
// ---------------------------------------------------------------------------

fn session_cookie(state: &AppState, value: String, max_age_secs: i64) -> Cookie<'static> {
    Cookie::build((state.config.cookie_name.clone(), value))
        .http_only(true) // not readable from JS -> XSS can't steal it
        .same_site(SameSite::Lax) // CSRF mitigation for cross-site requests
        .secure(state.config.cookie_secure) // HTTPS-only when enabled
        .path("/")
        .max_age(time::Duration::seconds(max_age_secs))
        .build()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(creds): Json<Credentials>,
) -> Result<(StatusCode, CookieJar, Json<AuthResponse>), AppError> {
    let email = normalize_email(&creds.email)?;
    validate_password(&creds.password)?;

    let password = creds.password;
    let hash = tokio::task::spawn_blocking(move || password::hash_password(&password))
        .await
        .map_err(|e| AppError::Internal(format!("hash task: {e}")))??;

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (id, email, password_hash) VALUES ($1, $2, $3) \
         RETURNING id, email, password_hash, created_at",
    )
    .bind(Uuid::new_v4())
    .bind(&email)
    .bind(&hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.is_unique_violation() => {
            AppError::Conflict("an account with this email already exists".into())
        }
        other => AppError::from(other),
    })?;

    let token = token::issue_token(
        &state.config.jwt_secret,
        user.id,
        &user.email,
        state.config.jwt_ttl_seconds,
    )?;
    let jar = jar.add(session_cookie(&state, token, state.config.jwt_ttl_seconds));

    Ok((StatusCode::CREATED, jar, Json(AuthResponse { user: user.into() })))
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(creds): Json<Credentials>,
) -> Result<(CookieJar, Json<AuthResponse>), AppError> {
    // Normalize, but on failure fall through to the same generic 401 below.
    let email = normalize_email(&creds.email).ok();

    let user = match &email {
        Some(email) => sqlx::query_as::<_, User>(
            "SELECT id, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&state.pool)
        .await?,
        None => None,
    };

    // Always verify *something* so timing doesn't reveal whether the account
    // exists. Unknown users are checked against the throwaway hash.
    let hash = match &user {
        Some(u) => u.password_hash.clone(),
        None => state.dummy_hash.as_ref().clone(),
    };
    let password = creds.password;
    let verified = tokio::task::spawn_blocking(move || password::verify_password(&password, &hash))
        .await
        .map_err(|e| AppError::Internal(format!("verify task: {e}")))?;

    let user = match (user, verified) {
        (Some(user), true) => user,
        _ => return Err(AppError::Unauthorized("invalid email or password")),
    };

    let token = token::issue_token(
        &state.config.jwt_secret,
        user.id,
        &user.email,
        state.config.jwt_ttl_seconds,
    )?;
    let jar = jar.add(session_cookie(&state, token, state.config.jwt_ttl_seconds));

    Ok((jar, Json(AuthResponse { user: user.into() })))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (StatusCode, CookieJar, ()) {
    // Overwrite the cookie with an immediately-expiring one.
    let jar = jar.add(session_cookie(&state, String::new(), 0));
    (StatusCode::NO_CONTENT, jar, ())
}

pub async fn me(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<AuthResponse>, AppError> {
    let row = sqlx::query_as::<_, User>(
        "SELECT id, email, password_hash, created_at FROM users WHERE id = $1",
    )
    .bind(user.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized("not authenticated"))?;

    Ok(Json(AuthResponse { user: row.into() }))
}

// ---------------------------------------------------------------------------
// AuthUser extractor: pulls the session cookie, verifies the JWT, hands back
// the authenticated identity. Any handler that takes `AuthUser` is protected.
// ---------------------------------------------------------------------------

pub struct AuthUser {
    pub id: Uuid,
    #[allow(dead_code)]
    pub email: String,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // CookieJar extraction is infallible.
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .unwrap_or_default();

        let token = jar
            .get(&state.config.cookie_name)
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized("not authenticated"))?;

        let claims = token::verify_token(&state.config.jwt_secret, &token)?;
        let id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::Unauthorized("not authenticated"))?;

        Ok(AuthUser {
            id,
            email: claims.email,
        })
    }
}
