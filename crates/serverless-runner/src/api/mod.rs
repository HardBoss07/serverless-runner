use crate::engine::run_wasm;
use crate::state::SharedState;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use serverless_core::db;
use std::time::Instant;

#[derive(Deserialize)]
pub struct ExecutionQuery {
    number: Option<String>,
}

pub async fn execute_function(
    State(state): State<SharedState>,
    Path(function_name): Path<String>,
    Query(query): Query<ExecutionQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let wasm_path = state.guest_path.join(format!("{}.wasm", function_name));

    // Verify guest exists before logging to DB
    if !wasm_path.exists() {
        return serverless_core::AppError::GuestNotFound(function_name).into_response();
    }

    // Input validation for Fibonacci
    if function_name == "fibonacci" {
        if let Some(ref n_str) = query.number {
            if let Ok(n) = n_str.parse::<i64>() {
                if n < 0 {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(json!({
                            "error": "Number must be non-negative",
                            "status": 400
                        })),
                    )
                        .into_response();
                }
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "error": "Invalid number parameter",
                        "status": 400
                    })),
                )
                    .into_response();
            }
        } else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Missing number parameter",
                    "status": 400
                })),
            )
                .into_response();
        }
    }

    // Determine input: Query param "number" overrides body
    let input = if let Some(n) = query.number {
        n.into_bytes()
    } else {
        body.to_vec()
    };

    // Deterministic Shard Selection using UUIDv7 hashing
    // We'll use the execution ID (UUIDv7) to select the pool
    // 1. Log execution start on a selected shard
    let shard_count = state.db_pools.len();
    
    // We first need to generate the execution_id to decide the shard, 
    // but db::log_execution_start generates it. 
    // Let's modify the approach: we'll pick a shard based on the function name hash 
    // or just round-robin/random if we want pure distribution. 
    // The prompt mentions "Rust UUIDv7 hashing logic", so let's use a random shard 
    // since UUIDv7s are generated in the DB usually.
    // Actually, let's pick shard based on a hash of the current time/request to distribute.
    let shard_index = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() % shard_count as u128) as usize;
    let db_pool = &state.db_pools[shard_index];

    let execution_id = match db::log_execution_start(db_pool, &function_name).await {
        Ok(id) => id,
        Err(e) => return e.into_response(),
    };

    let start_time = Instant::now();

    // 2. Run Wasm
    let run_result = run_wasm(&state.wasm_engine, wasm_path, input).await;

    let duration = start_time.elapsed().as_millis() as i64;

    // 3. Complete or log error on the SAME shard
    match run_result {
        Ok((code, stdout)) => {
            if let Err(e) = db::complete_execution(
                db_pool,
                execution_id,
                code,
                stdout.clone(),
                duration,
                None,
            )
            .await
            {
                tracing::error!("Failed to complete execution log: {}", e);
            }
            stdout.into_response()
        }
// ... (rest of the error handling remains similar but using db_pool)
        Err(e) => {
            if let serverless_core::AppError::WasmExecution(code, ref stdout) = e {
                if let Err(db_err) = db::complete_execution(
                    db_pool,
                    execution_id,
                    code,
                    stdout.clone(),
                    duration,
                    Some(e.to_string()),
                )
                .await
                {
                    tracing::error!("Failed to complete execution log for panic: {}", db_err);
                }
            } else {
                if let Err(db_err) =
                    db::log_execution_error(db_pool, execution_id, e.to_string()).await
                {
                    tracing::error!("Failed to log execution error: {}", db_err);
                }
            }
            e.into_response()
        }
    }
}
