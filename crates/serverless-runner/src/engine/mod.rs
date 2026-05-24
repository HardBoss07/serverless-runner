pub mod batcher;

use serverless_core::AppResult;
use std::path::PathBuf;
use wasmtime::{Engine, Linker, Module, ResourceLimiter, Store, StoreLimits, StoreLimitsBuilder};
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1::{add_to_linker_async as add_to_linker, WasiP1Ctx as WasiCtx};
use wasmtime_wasi::WasiCtxBuilder;

struct HostState {
    wasi: WasiCtx,
    limits: StoreLimits,
}

impl ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        self.limits.memory_growing(current, desired, maximum)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        self.limits.table_growing(current, desired, maximum)
    }
}

pub async fn run_wasm(engine: &Engine, module: Module, input: Vec<u8>) -> AppResult<(i32, String)> {
    // 2. WASI Configuration with Memory Pipes
    let stdout_pipe = MemoryOutputPipe::new(1024 * 1024); // 1MB buffer
    let stdin_pipe = MemoryInputPipe::new(input);

    let mut wasi_ctx_builder = WasiCtxBuilder::new();
    wasi_ctx_builder
        .stdout(stdout_pipe.clone())
        .stdin(stdin_pipe)
        .inherit_stderr();

    let wasi_ctx = wasi_ctx_builder.build_p1();

    // 3. Store Creation with Resource Limits
    let state = HostState {
        wasi: wasi_ctx,
        limits: StoreLimitsBuilder::new()
            .memory_size(64 * 1024 * 1024) // 64MB
            .instances(1)
            .build(),
    };

    let mut store = Store::new(engine, state);
    store.limiter(|s| s);

    // Fuel limit for timeouts (100,000,000 instructions)
    store
        .set_fuel(100_000_000)
        .map_err(|e| serverless_core::AppError::WasmEngine(e.to_string()))?;

    // 4. Linker Setup
    let mut linker = Linker::new(engine);
    add_to_linker(&mut linker, |s: &mut HostState| &mut s.wasi)
        .map_err(|e| serverless_core::AppError::WasmEngine(e.to_string()))?;

    // 5. Instantiation
    let instance = linker
        .instantiate_async(&mut store, &module)
        .await
        .map_err(|e| serverless_core::AppError::WasmEngine(e.to_string()))?;

    // 6. Invocation
    let func = instance
        .get_typed_func::<(), ()>(&mut store, "_start")
        .map_err(|e| serverless_core::AppError::WasmEngine(e.to_string()))?;

    tracing::debug!("Invoking Wasm function...");
    let result = func.call_async(&mut store, ()).await;

    // 7. Result Capture
    let stdout_bytes = stdout_pipe.contents();
    let stdout_string = String::from_utf8_lossy(&stdout_bytes).to_string();

    match result {
        Ok(_) => {
            tracing::debug!("Wasm execution successful");
            Ok((0, stdout_string))
        }
        Err(e) => {
            tracing::error!("Wasm execution error: {:?}", e);
            let error_string = format!("{:?}", e);

            // Check for exit status
            if let Some(exit) = e.downcast_ref::<wasmtime_wasi::I32Exit>() {
                if exit.0 != 0 {
                    return Err(serverless_core::AppError::WasmExecution(
                        exit.0,
                        stdout_string,
                    ));
                }
                Ok((0, stdout_string))
            } else if error_string.contains("fuel") || error_string.contains("Fuel") {
                Err(serverless_core::AppError::WasmEngine("Timeout".into()))
            } else if error_string.contains("memory") || error_string.contains("Memory") {
                Err(serverless_core::AppError::WasmEngine(
                    "Memory limit exceeded".into(),
                ))
            } else {
                Err(serverless_core::AppError::WasmEngine(format!("{:#}", e)))
            }
        }
    }
}
