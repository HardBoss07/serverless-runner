use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::Engine;

pub struct AppState {
    pub db_pools: Vec<PgPool>, // Multi-shard support
    pub wasm_engine: Engine,
    pub guest_path: PathBuf,
}

pub type SharedState = Arc<AppState>;
