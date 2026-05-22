use crate::error::AppResult;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn log_execution_start(pool: &PgPool, function_name: &str) -> AppResult<Uuid> {
    let row = sqlx::query!(
        "INSERT INTO executions (function_name) VALUES ($1) RETURNING id",
        function_name
    )
    .fetch_one(pool)
    .await?;

    Ok(row.id)
}

pub async fn complete_execution(
    pool: &PgPool,
    id: Uuid,
    code: i32,
    stdout: String,
    duration: i64,
    error_message: Option<String>,
) -> AppResult<()> {
    // We only store the first 2048 characters of stdout
    let stdout_snippet = if stdout.len() > 2048 {
        &stdout[..2048]
    } else {
        &stdout
    };

    sqlx::query(
        "UPDATE executions SET status_code = $1, stdout_snippet = $2, duration_ms = $3, error_message = $4 WHERE id = $5"
    )
    .bind(code)
    .bind(stdout_snippet)
    .bind(duration)
    .bind(error_message)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn log_execution_error(pool: &PgPool, id: Uuid, error: String) -> AppResult<()> {
    sqlx::query!(
        "UPDATE executions SET error_message = $1 WHERE id = $2",
        error,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
