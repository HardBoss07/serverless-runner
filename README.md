# Serverless Runner

A high-performance, trust-free Wasm function execution platform built in Rust using Axum, Wasmtime, and PostgreSQL to host WebAssembly microservices on Kubernetes with minimal lock contention and hardware-level isolation.

**Key Features**

- **High-Throughput Sandbox**: Executes compiled Wasm binaries inside Wasmtime capability-based sandboxes using isolated virtual pipes for `stdin` and `stdout`.

- **Async-Sync Isolation**: Decouples the non-blocking Axum HTTP gateway from CPU-heavy Wasm execution using `tokio::task::spawn_blocking` to prevent async worker thread starvation.

- **Sharded Module Cache**: Uses a thread-safe `DashMap` to store pre-compiled `Arc<Module>` instances across requests without global `Mutex` lock contention.

- **Micro-Batched Logging**: Buffers execution log entries in an MPSC channel and writes them asynchronously using PostgreSQL vectorized `UNNEST` SQL queries.

- **Time-Ordered Partitioning**: Leverages `UUIDv7` keys to maintain database B-Tree index locality and distribute write operations evenly across PostgreSQL shards.

**System Architecture**

```
[ NGINX Ingress ] ──> [ Axum Gateway ]
                             │
            ┌────────────────┴────────────────┐
            ▼                                 ▼
   [ DashMap Cache ]                 [ MPSC Log Buffer ]
            │                                 │
   (Hit) ───┼─── (Miss: Compile)              ▼
            ▼                        [ Async SQL Batcher ]
   [ spawn_blocking ]                         │
            │                                 ▼
  [ Wasmtime Instance ]              [ PostgreSQL Shards ]
```

**Performance Benchmarks**

| Metric              | Raw Axum (`/live`) | Wasm Execution (Fibonacci)        |
| :------------------ | :----------------- | :-------------------------------- |
| **Throughput**      | 38,728.8 RPS       | 14,972.8 RPS                      |
| **Average Latency** | 12.9 ms            | 6.6 ms                            |
| **P99 Latency**     | 67.2 ms            | 31.5 ms                           |
| **Success Rate**    | 100%               | 100.00% (under 2,000 connections) |

**Resource Guardrails & Security**

- **Memory Ceiling**: Enforces a strict 64 MiB limit on linear memory growth per execution context.
- **Fuel Metering**: Imposes a deterministic execution budget of 100,000,000 fuel units to prevent infinite loop traps.
- **Output Truncation**: Bounds stdout buffer output to 1 MiB to protect runner pod memory.
- **Sandbox Isolation**: Restricts guest WASI instances from host network and filesystem capabilities.

**Tech Stack**

- **Gateway Runtime**: Rust, Axum, Tokio$
- **Wasm Engine**: Wasmtime JIT, WASI capability model
- **Persistence & Caching**: DashMap, PostgreSQL, PgBouncer pool, `UNNEST` batching, `UUIDv7`
- **Deployment Target**: Kubernetes
