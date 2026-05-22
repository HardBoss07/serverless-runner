mod api;
mod engine;
mod state;

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;
use wasmtime::{Config, Engine};

#[derive(Parser, Debug)]
#[command(version, about = "Serverless Wasm Runner", long_about = None)]
struct Args {}

/// Global readiness state for Kubernetes readiness probe
struct ServerState {
    app: Arc<AppState>,
    is_ready: Arc<AtomicBool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse arguments, this will handle --help and --version and exit automatically
    let _args = Args::parse();

    // 1. Load .env
    dotenv().ok();

    // 2. Initialize tracing
    tracing_subscriber::fmt::init();

    // 3. Database Pools (Multi-shard support)
    let database_urls = env::var("DATABASE_URLS").unwrap_or_else(|_| env::var("DATABASE_URL").expect("DATABASE_URL or DATABASE_URLS must be set"));
    let mut db_pools = Vec::new();

    for url in database_urls.split(',') {
        let pool = PgPoolOptions::new()
            .max_connections(20)
            .connect(url.trim())
            .await?;
        db_pools.push(pool);
    }

    // 4. Wasmtime Engine
    let mut config = Config::new();
    config.async_support(true);
    config.consume_fuel(true);
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

    let is_ready = Arc::new(AtomicBool::new(true));

    let app_state = Arc::new(AppState {
        db_pools,
        wasm_engine,
        guest_path,
    });

    let server_state = Arc::new(ServerState {
        app: app_state.clone(),
        is_ready: is_ready.clone(),
    });

    // 6. Router
    let app = Router::new()
        .route("/ready", get(readiness_handler))
        .route("/live", get(|| async { "OK" }))
        .route("/execute/{function_name}", post(api::execute_function))
        .with_state(app_state)
        .layer(axum::extract::Extension(server_state));

    // 7. Server
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));

    tracing::info!("Serverless Runner listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;

    // Implement graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(is_ready))
        .await?;

    Ok(())
}

async fn readiness_handler(
    axum::extract::Extension(state): axum::extract::Extension<Arc<ServerState>>,
) -> (StatusCode, &'static str) {
    if state.is_ready.load(Ordering::SeqCst) {
        (StatusCode::OK, "Ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "Not Ready")
    }
}

async fn shutdown_signal(is_ready: Arc<AtomicBool>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Termination signal received. Failing readiness probe...");
    is_ready.store(false, Ordering::SeqCst);

    tracing::info!("Sleeping for 5 seconds to allow load balancer update...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    tracing::info!("Starting graceful drain of in-flight requests...");
}
