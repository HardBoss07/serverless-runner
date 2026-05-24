use crate::engine::batcher::Batcher;
use dashmap::DashMap;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::{Engine, Module};

pub struct AppState {
    pub db_pools: Vec<PgPool>, // Multi-shard support
    pub wasm_engine: Engine,
    pub guest_path: PathBuf,
    pub batcher: Batcher,
    pub module_cache: DashMap<String, Module>,
}

pub type SharedState = Arc<AppState>;
