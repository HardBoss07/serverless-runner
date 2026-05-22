# Guest Catalog

This catalog provides a comprehensive matrix of all Wasm guests currently supported by the Serverless Runner, detailing their use cases, behaviors, and expected outcomes.

## Guest Matrix

| Guest Name | Use Case | Expected Input | Expected Outcome | Fuel/Resource Behavior |
| :--- | :--- | :--- | :--- | :--- |
| `env-reader` | Security / Isolation | None | Prints "Env var not found" | Baseline fuel. Verifies environment isolation. |
| `fibonacci` | Compute / Fuel Test | Query `?number=n` | n-th Fibonacci number | Recursive. High fuel consumption for large `n`. |
| `fs-reader` | Security / Sandbox | None | Prints "pre-opened" error | Verifies filesystem isolation. |
| `hello-world` | Basic Smoke Test | Optional `stdin` name | "Hello, [name]!" | Minimal fuel consumption. |
| `infinite-loop` | Timeout Testing | None | `504 Gateway Timeout` | Consumes all 100M fuel units and traps. |
| `long-output` | Persistence Test | None | 3000 bytes of 'B' | Verifies DB snippet truncation (2048 chars). |
| `memory-hog` | Memory Limit Test | None | `500 Internal Error` | Traps when exceeding 64MB memory limit. |
| `net-guest` | Security / Network | None | Prints "Operation not supported" | Verifies network sandbox. |
| `panic-guest` | Error Handling | None | `500 Internal Error` | Exits with code 101. Verifies error mapping. |
| `stdout-spammer` | Buffer Limit Test | None | 1MB of 'A' | Truncated by the 1MB host pipe limit. |

---

## Detailed Guest Analysis

### Security and Sandbox Verification

- **`env-reader`**: Demonstrates that environment variables from the host (like `DATABASE_URL`) are not leaked into the guest unless explicitly configured.
- **`fs-reader`**: Attempts to access `/etc/passwd`. Because the host does not pre-open any directories with write/read permissions for the guest, `wasmtime-wasi` returns an error indicating that no pre-opened file descriptor is available for that path.
- **`net-guest`**: Attempts a TCP connection. Standard WASI preview1 does not provide network socket capabilities, resulting in an "Operation not supported" error.

### Resource Boundary Testing

- **`infinite-loop`**: A critical test for the fuel metering system. It ensures that a single misbehaving guest cannot hang the runner thread or starve other requests.
- **`memory-hog`**: Iteratively allocates 1MB blocks. The runner's `ResourceLimiter` intercepts growth requests and traps the execution before it can impact host stability.
- **`stdout-spammer`**: Generates a 2MB payload. The runner configures a 1MB `MemoryOutputPipe`, effectively capping the maximum response size from any single guest.

### Functional Utilities

- **`fibonacci`**: Useful for benchmarking the overhead of the Wasm runtime and fuel metering.
- **`hello-world`**: The standard entry point for verifying that `stdin` and `stdout` piping is correctly configured.

### Persistence and Observability

- **`long-output-guest`**: Used to verify the "Log-and-Update" pattern. While the full 3000 bytes are returned to the user, the database record is strictly capped to ensure the persistence layer is not overwhelmed by large logs.
- **`panic-guest`**: Specifically used to verify that non-zero exit codes are correctly captured and stored in the `status_code` column of the `executions` table.
