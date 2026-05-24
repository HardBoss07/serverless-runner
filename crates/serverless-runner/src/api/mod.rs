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
    let shard_count = state.db_pools.len();
    let execution_id = uuid::Uuid::now_v7();

    // Pick shard based on execution_id hash for uniform distribution
    let shard_index = (execution_id.as_u128() % shard_count as u128) as usize;

    // 1. Get or compile Wasm module
    let module = if let Some(m) = state.module_cache.get(&function_name) {
        m.clone()
    } else {
        let m = match wasmtime::Module::from_file(&state.wasm_engine, &wasm_path) {
            Ok(m) => m,
            Err(e) => {
                return serverless_core::AppError::CompileError(e.to_string()).into_response()
            }
        };
        state.module_cache.insert(function_name.clone(), m.clone());
        m
    };

    let start_time = Instant::now();

    // 2. Run Wasm on a blocking thread to prevent async starvation
    let engine_clone = state.wasm_engine.clone();
    let input_clone = input.clone();

    let run_result = match tokio::task::spawn_blocking(move || {
        let handle = tokio::runtime::Handle::current();
        handle.block_on(async move { run_wasm(&engine_clone, module, input_clone).await })
    })
    .await
    {
        Ok(res) => res,
        Err(e) => return serverless_core::AppError::WasmEngine(e.to_string()).into_response(),
    };
    let duration = start_time.elapsed().as_millis() as i64;

    // 3. Log asynchronously via batcher
    let db_message = match &run_result {
        Ok((code, stdout)) => crate::engine::batcher::DbMessage::Execution {
            id: execution_id,
            function_name: function_name.clone(),
            status_code: *code,
            stdout_snippet: if stdout.len() > 2048 {
                stdout[..2048].to_string()
            } else {
                stdout.clone()
            },
            duration_ms: duration,
            error_message: None,
            shard_index,
        },
        Err(e) => {
            let (code, stdout, error_msg) =
                if let serverless_core::AppError::WasmExecution(c, ref s) = e {
                    (*c, s.clone(), Some(e.to_string()))
                } else {
                    (-1, String::new(), Some(e.to_string()))
                };
            crate::engine::batcher::DbMessage::Execution {
                id: execution_id,
                function_name: function_name.clone(),
                status_code: code,
                stdout_snippet: if stdout.len() > 2048 {
                    stdout[..2048].to_string()
                } else {
                    stdout
                },
                duration_ms: duration,
                error_message: error_msg,
                shard_index,
            }
        }
    };

    if let Err(e) = state.batcher.send(db_message).await {
        tracing::error!("Failed to send to batcher: {}", e);
    }

    // 4. Return result to client
    match run_result {
        Ok((_, stdout)) => stdout.into_response(),
        Err(e) => e.into_response(),
    }
}
