# Project Scratchpad - serverless-runner

## Status

- [x] Phase 1: Ingestion & Workspace Prep
- [x] Phase 2: Core Domain Implementation (serverless-core)
- [x] Phase 3: Runner Engine & API (serverless-runner)
- [ ] Phase 4: Containerization & Integration

## CRITICAL RULES

- **Docker-only Execution:** ALL runtime execution (tests, runs, builds) must be performed inside the Docker containers defined in `docker-compose.yml` using `docker compose exec`.
- **Host Limitations:** The host system is strictly reserved for file editing, git operations, and static analysis (e.g., `cargo fmt`, `cargo check`). Executing binaries, tests, or build commands on the host is forbidden to ensure environment isolation.
- **PowerShell Compliance:** All host-side shell commands must be valid PowerShell syntax (e.g., use `;` for command chaining instead of `&&`).
- **NO BLOCKING COMMANDS:** Long-lived processes (servers, watch tasks) must NOT be executed via `docker compose exec` or `docker compose run`. Use `docker compose restart` or log analysis instead for managing/verifying background services. Commands executed must be short-lived and terminate gracefully.
- **Test Integrity:** STRICTLY DO NOT modify test logic, inputs, or assertions to force a test to pass. The ONLY permissible changes to test files are: 1) Correcting an `assert` macro if it incorrectly contradicts the `06_Integration_Testing_Playbook.md` specification, or 2) Uncommenting existing tests to activate them.

# Decisions & Rules

- **Error Handling:** Use `AppError` and `AppResult` from `serverless-core`.
- **Wasm Runtime:** Wasmtime with `async` support.
- **Communication:** `stdin`/`stdout` memory pipes.
- **Database:** PostgreSQL with `sqlx`. Log-and-Update pattern.
- **Formatting:** `cargo fmt` after changes.

## Current State

- Documentation ingested.
- Workspace flattened.
- Database schema and migrations implemented.
- serverless-core implemented (errors, models, db logic).
- serverless-runner implemented (Axum API, Wasmtime execution engine).

## Next Steps

- Implement integration testing and full loop validation.
