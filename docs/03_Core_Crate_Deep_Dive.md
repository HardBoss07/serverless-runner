# 03 - Core Crate Deep Dive

## 3.1 Unified Error Handling

All operations within the platform utilize a single, exhaustive error enumeration defined in `core/src/error.rs`.

```rust
use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Wasm engine error: {0}")]
    WasmEngine(String),

    #[error("Wasm execution failed (Code {0})")]
    WasmExecution(i32),

    #[error("Guest '{0}' not found on disk")]
    GuestNotFound(String),

    #[error("Module compilation error: {0}")]
    CompileError(String),

    #[error("Internal system error")]
    Internal,
}

/// The standard Result type for all project functions.
pub type AppResult<T> = Result<T, AppError>;
```

### 3.2 Axum Integration for Errors

The `AppError` enum must implement `IntoResponse` to allow for clean, automatic conversion to HTTP responses.

```rust
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::GuestNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Sqlx(_) => (StatusCode::SERVICE_UNAVAILABLE, "Database is currently unavailable".into()),
            AppError::WasmExecution(code) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Wasm exited with non-zero code: {}", code)),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
```

## 3.3 Core Database Functions

The `core/src/db.rs` module should contain the following functions with exact signatures:

| Function              | Signature                                                                                                             | Use Case                  |
| :-------------------- | :-------------------------------------------------------------------------------------------------------------------- | :------------------------ |
| `log_execution_start` | `pub async fn log_execution_start(pool: &PgPool, function_name: &str) -> AppResult<Uuid>`                             | Initial insert of record. |
| `complete_execution`  | `pub async fn complete_execution(pool: &PgPool, id: Uuid, code: i32, stdout: String, duration: i64) -> AppResult<()>` | Success/Exit update.      |
| `log_execution_error` | `pub async fn log_execution_error(pool: &PgPool, id: Uuid, error: String) -> AppResult<()>`                           | Panic/Engine update.      |

## 3.4 Shared Model: `Execution`

The core struct representing the database table.

```rust
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Execution {
    pub id: Uuid,
    pub function_name: String,
    pub status_code: Option<i32>,
    pub stdout_snippet: Option<String>,
    pub duration_ms: Option<i64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}
```
