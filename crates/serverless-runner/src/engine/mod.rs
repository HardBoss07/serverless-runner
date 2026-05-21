use serverless_core::AppResult;
use std::path::PathBuf;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::pipe::{MemoryInputPipe, MemoryOutputPipe};
use wasmtime_wasi::preview1::{add_to_linker_async as add_to_linker, WasiP1Ctx as WasiCtx};
use wasmtime_wasi::WasiCtxBuilder;

pub async fn run_wasm(
    engine: &Engine,
    wasm_path: PathBuf,
    input: Vec<u8>,
) -> AppResult<(i32, String)> {
    // 1. Module Load
    let module = Module::from_file(engine, &wasm_path)
        .map_err(|e| serverless_core::AppError::CompileError(e.to_string()))?;

    // 2. WASI Configuration with Memory Pipes
    let stdout_pipe = MemoryOutputPipe::new(1024 * 1024); // 1MB buffer
    let stdin_pipe = MemoryInputPipe::new(input);

    let mut wasi_ctx_builder = WasiCtxBuilder::new();
    wasi_ctx_builder
        .stdout(stdout_pipe.clone())
        .stdin(stdin_pipe)
        .inherit_stderr();

    let wasi_ctx = wasi_ctx_builder.build_p1();

    // 3. Store Creation
    let mut store = Store::new(engine, wasi_ctx);
    // store.add_fuel(10_000_000_000).map_err(|e| serverless_core::AppError::WasmEngine(e.to_string()))?;

    // 4. Linker Setup
    let mut linker = Linker::new(engine);
    add_to_linker(&mut linker, |s: &mut WasiCtx| s)
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

    let result = func.call_async(&mut store, ()).await;

    // 7. Result Capture
    let stdout_bytes = stdout_pipe.contents();
    let stdout_string = String::from_utf8_lossy(&stdout_bytes).to_string();

    match result {
        Ok(_) => Ok((0, stdout_string)),
        Err(e) => {
            tracing::error!("Wasm execution error: {:?}", e);
            // Check for exit status
            if let Some(exit) = e.downcast_ref::<wasmtime_wasi::I32Exit>() {
                Ok((exit.0, stdout_string))
            } else {
                Err(serverless_core::AppError::WasmEngine(e.to_string()))
            }
        }
    }
}
