mod api;
mod engine;
mod state;

use axum::{routing::post, Router};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use wasmtime::{Config, Engine};

#[derive(Parser, Debug)]
#[command(version, about = "Serverless Wasm Runner", long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments, this will handle --help and --version and exit automatically
    let _args = Args::parse();

    // 1. Load .env
    dotenv().ok();

    // 2. Initialize tracing
    tracing_subscriber::fmt::init();

    // 3. Database Pool
    let database_url = env::var("DATABASE_URL")?;
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    // 4. Wasmtime Engine
    let mut config = Config::new();
    config.async_support(true);
    // config.consume_fuel(true);
    config.static_memory_maximum_size(64 * 1024 * 1024); // 64MB

    let wasm_engine = Engine::new(&config)?;

    // 5. App State
    let guest_path = PathBuf::from(
        env::var("WASM_STORAGE_DIR").unwrap_or_else(|_| "./guests_compiled".to_string()),
    );

    // Ensure the directory exists
    if !guest_path.exists() {
        std::fs::create_dir_all(&guest_path)?;
    }

    let state = Arc::new(AppState {
        db_pool,
        wasm_engine,
        guest_path,
    });

    // 6. Router
    let app = Router::new()
        .route("/execute/{function_name}", post(api::execute_function))
        .with_state(state);

    // 7. Server
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    tracing::info!("Serverless Runner listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
