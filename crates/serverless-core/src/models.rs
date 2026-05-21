use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

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
