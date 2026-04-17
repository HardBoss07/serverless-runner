# GEMINI.md - Serverless Runner Platform

This document serves as the primary guidance for Gemini CLI when working on the `serverless-runner` project. It synthesizes architectural mandates, development standards, and operational workflows.

## Project Vision

A high-performance, sandboxed serverless execution platform. It uses **Rust** for the host (Runner) and **WebAssembly (WASI)** for guest functions, providing hard multi-tenant isolation with minimal overhead.

---

## Technical Stack

- **Host Framework:** Axum (Tokio runtime)
- **Wasm Runtime:** Wasmtime (with WASI support)
- **Database:** PostgreSQL (via SQLx for async, type-safe queries)
- **Error Handling:** `thiserror` (unified platform-wide enum)
- **Serialization:** `serde` / `serde_json`
- **Target Architecture:** `wasm32-wasi` for all guest functions

---

## Architectural Mandates

### 1. Crate Responsibility

- **`serverless-core` (Lib):** Domain logic, DB schema, SQLx queries, shared models, and the `AppError` enum.
- **`serverless-runner` (Bin):** Axum HTTP server, Wasmtime orchestration (Engine/Store management), and WASI piping.
- **`guests/` (Independent Workspace):** Standalone Rust projects compiled to `.wasm`.

### 2. Wasmtime Performance & Safety

- **Engine/Store Split:** Initialize one `wasmtime::Engine` at startup. Create a new `wasmtime::Store` for every incoming HTTP request.
- **Memory Pipes:** Communication between Host and Guest MUST happen via `stdin` (request body) and `stdout` (response body) memory pipes. No direct file system access for guests.
- **Resource Constraints:**
  - Memory: Max 64MB per guest.
  - Execution: Use `fuel` consumption to prevent infinite loops.
  - Cleanup: Stores must be dropped immediately after execution.

### 3. Database Lifecycle (Log-and-Update)

Do not hold DB transactions open during Wasm execution. Use atomic operations:

1. `log_execution_start`: INSERT record on request receipt.
2. `complete_execution` or `log_execution_error`: UPDATE record after Wasm yields or fails.

---

## Development Standards

### Rust Conventions

- **Error Handling:** Use `AppResult<T>` (alias for `Result<T, AppError>`) for all fallible operations.
- **Async:** Use `tokio` primitives. Avoid blocking the executor.
- **Safety:** Minimize `unsafe` code. Rely on Wasmtime for sandbox isolation.
- **Typing:** Strict typing for database models and API payloads. Use `sqlx::FromRow`.

### Formatting & Style

- **No Emojis:** Do not use emojis in code, comments, or documentation.
- **Naming:** Follow standard Rust `PascalCase` for types and `snake_case` for functions/variables.
- **Documentation:** Use triple-slash `///` for public function documentation in `core`.

---

## Operational Workflows

### 1. Building Guests

Guests must be compiled to the `wasm32-wasi` target and moved to the distribution folder.

```bash
# Compile
cargo build --target wasm32-wasi --release -p guest-hello-world
# Deploy to runner
cp target/wasm32-wasi/release/guest_hello_world.wasm ./guests_compiled/
```

### 2. Database Migrations

Always use `sqlx-cli` for schema changes.

```bash
sqlx migrate add <description>
sqlx migrate run
```

### 3. Testing

- **Unit Tests:** Inside each crate for logic validation.
- **Integration Tests:** Use `curl` to hit the `/execute/:function_name` endpoint.
- **Verification:** Always check the `executions` table in Postgres after a test run.

---

## Roadmap Context

- [x] Workspace Initialization
- [ ] Database Schema Setup (Pending `sqlx migrate`)
- [ ] Core Crate Implementation (DB & Error handling)
- [ ] Runner Engine (Wasmtime Integration)
- [ ] Full Loop Validation

## Critical File Paths

- `crates/serverless-core/src/error.rs`: Source of truth for error handling.
- `crates/serverless-runner/src/engine/`: Wasmtime configuration and execution logic.
- `docs/`: Technical specifications for every subsystem.
- `guests_compiled/`: The directory where the Runner looks for `.wasm` binaries.
