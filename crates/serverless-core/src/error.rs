use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Wasm engine error: {0}")]
    WasmEngine(String),

    #[error("Wasm exited with code {0}")]
    WasmExecution(i32, String),

    #[error("Guest '{0}' not found on disk")]
    GuestNotFound(String),

    #[error("Module compilation error: {0}")]
    CompileError(String),

    #[error("Internal system error")]
    Internal,
}

/// The standard Result type for all project functions.
pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::GuestNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Sqlx(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "Database unavailable".into(),
            ),
            AppError::WasmExecution(code, _) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Wasm exited with code {}", code),
            ),
            AppError::WasmEngine(ref msg) if msg.contains("Timeout") || msg.contains("fuel") => {
                (StatusCode::GATEWAY_TIMEOUT, "Execution timed out".into())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
