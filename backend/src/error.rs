//! A single application error type that knows how to turn itself into an HTTP
//! response. Internal details are logged but never leaked to clients.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    /// Bad input from the client (422).
    Validation(String),
    /// Missing/invalid session (401).
    Unauthorized(&'static str),
    /// A resource already exists, e.g. duplicate email (409).
    Conflict(String),
    /// Something blew up on our side (500). The string is logged, not returned.
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Validation(m) => (StatusCode::UNPROCESSABLE_ENTITY, m),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.to_string()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Internal(detail) => {
                tracing::error!(error = %detail, "internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(format!("database error: {err}"))
    }
}
