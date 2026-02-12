use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::json;

/// Cuerpo de respuesta de error para documentación OpenAPI.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ErrorBody {
    /// Mensaje de error.
    pub error: String,
}

/// Error unificado de la aplicación.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Insufficient stock for inventory item {0}")]
    InsufficientStock(uuid::Uuid),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::InsufficientStock(id) => (
                StatusCode::CONFLICT,
                format!("Stock insuficiente para el item {id}"),
            ),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Error interno".into())
            }
            AppError::Sqlx(e) => {
                tracing::error!("Database error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Error de base de datos".into(),
                )
            }
            AppError::SerdeJson(e) => {
                tracing::error!("Serialization error: {e}");
                (StatusCode::BAD_REQUEST, "Error de serialización".into())
            }
            AppError::Jwt(e) => {
                tracing::error!("JWT error: {e}");
                (StatusCode::UNAUTHORIZED, "Token inválido".into())
            }
        };

        let body = json!({ "error": message });
        (status, axum::Json(body)).into_response()
    }
}
