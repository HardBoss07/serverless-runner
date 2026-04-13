# 04 - Runner Engine Technical Spec

## 4.1 Axum App State

The host application uses a thread-safe `AppState` struct to share critical resources across concurrent HTTP requests.

```rust
use std::sync::Arc;
use sqlx::PgPool;
use wasmtime::Engine;
use std::path::PathBuf;

pub struct AppState {
    pub db_pool: PgPool,
    pub wasm_engine: Engine,
    pub guest_path: PathBuf,
}

pub type SharedState = Arc<AppState>;
```

### Initialization Workflow (main.rs)

1. Load `.env` into environment.
2. Initialize `wasmtime::Config` (with `async_support(true)`).
3. Initialize `wasmtime::Engine` with the config.
4. Establish `sqlx::PgPool`.
5. Wrap in `Arc` and pass to `Router::with_state`.

## 4.2 WASI Configuration with Memory Pipes

We avoid file system access for guest communication. Instead, we use memory-backed pipes for `stdin` and `stdout`.

```rust
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::WasiCtxBuilder;

// Request Phase
let stdout_pipe = MemoryOutputPipe::new(1024 * 1024); // 1MB buffer
let stdin_pipe = MemoryInputPipe::new(body_bytes); // Axum Body Bytes

let mut wasi_ctx = WasiCtxBuilder::new()
    .stdout(Box::new(stdout_pipe.clone()))
    .stdin(Box::new(stdin_pipe))
    .inherit_stderr() // Log guest errors to host console
    .build();
```

## 4.3 The Engine Lifecycle

The following logic must be encapsulated in `runner/src/engine/mod.rs`.

### Function signature:

```rust
pub async fn run_wasm(
    engine: &Engine,
    wasm_path: PathBuf,
    input: Vec<u8>
) -> AppResult<(i32, String)>;
```

### Logical Execution Flow:

1. **Module Load:** `wasmtime::Module::from_file(engine, path)?`.
2. **Linker Setup:**
   - `let mut linker = wasmtime::Linker::new(engine);`.
   - `wasmtime_wasi::add_to_linker(&mut linker, |s: &mut WasiCtx| s)?;`.
3. **Store Creation:**
   - `let mut store = wasmtime::Store::new(engine, wasi_ctx);`.
4. **Instantiation:**
   - `let instance = linker.instantiate(&mut store, &module)?;`.
5. **Invocation:**
   - `let func = instance.get_typed_func::<(), ()>(&mut store, "_start")?;`.
   - `func.call(&mut store, ())?;`.
6. **Result Capture:**
   - Extract content from `stdout_pipe`.
   - Convert to `String`.
   - Read the WASI exit status from the `Store` (defaulting to 0 if not explicitly set).

## 4.4 Resource Safety

To prevent resource exhaustion, the Runner must enforce limits on the `wasmtime::Config`:

- **Memory:** `config.static_memory_maximum_size(64 * 1024 * 1024); // 64MB`.
- **Fuel:** `config.consume_fuel(true);`. Each request should be allotted a fixed fuel amount (e.g., 100,000,000 instructions) to prevent infinite loops.
