use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::Engine;

pub struct AppState {
    pub db_pool: PgPool,
    pub wasm_engine: Engine,
    pub guest_path: PathBuf,
}

pub type SharedState = Arc<AppState>;
