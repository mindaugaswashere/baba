//! auth-backend: a small, secure-by-default authentication API.
//!
//! Endpoints (all under /api):
//!   POST /api/auth/register  {email, password}  -> 201 {user} + session cookie
//!   POST /api/auth/login     {email, password}  -> 200 {user} + session cookie
//!   POST /api/auth/logout                        -> 204, clears cookie
//!   GET  /api/auth/me                            -> 200 {user} | 401
//!   GET  /api/health                             -> "ok"

mod auth;
mod config;
mod error;
mod models;
mod password;
mod state;
mod token;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderValue, Method},
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = Config::from_env()?;
    let bind_addr = config.bind_addr.clone();

    // Connection pool + run migrations on startup.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(Duration::from_secs(5)) // don't hang forever on a sick DB
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    // Constant-time-login decoy hash (see AppState::dummy_hash).
    let dummy_hash = password::hash_password("argon2-constant-time-decoy")
        .map_err(|e| format!("failed to build decoy hash: {e:?}"))?;

    let cors = CorsLayer::new()
        .allow_origin(config.cors_origin.parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(true);

    let state = AppState {
        pool,
        config: Arc::new(config),
        dummy_hash: Arc::new(dummy_hash),
    };

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        // Auth payloads are tiny (email + password); cap the body to 64 KiB.
        .layer(DefaultBodyLimit::max(64 * 1024))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("auth-backend listening on http://{bind_addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
