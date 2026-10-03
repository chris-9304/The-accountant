use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Duplicate: {0}")]
    Duplicate(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Duplicate(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::ParseError(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Database(e) => { tracing::error!("Database error: {:?}", e); (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()) }
            AppError::Internal(e) => { tracing::error!("Internal error: {:?}", e); (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()) }
        };
        let body = json!({"success": false, "data": null, "error": message, "meta": null});
        (status, Json(body)).into_response()
    }
}
