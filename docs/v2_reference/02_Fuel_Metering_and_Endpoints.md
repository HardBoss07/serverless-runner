# Fuel Metering and Endpoints

This document provides a technical deep dive into the resource management system and the available API surface of the Serverless Runner.

## API Endpoints

The runner exposes a single, highly flexible endpoint for guest execution.

### POST `/execute/{function_name}`

Invokes a specific Wasm guest module.

- **Parameters:**
  - `function_name` (Path): The filename of the `.wasm` module (without extension) stored in the `guests_compiled/` directory.
  - `number` (Query, Optional): Specifically used by the `fibonacci` guest to specify the sequence index. Overrides the request body if present.
- **Request Body:**
  - Raw bytes or text. Passed to the Wasm guest via `stdin`.
- **Responses:**
  - `200 OK`: Successful execution. Body contains the guest's `stdout`.
  - `400 Bad Request`: Validation failure (e.g., missing or invalid `number` for Fibonacci).
  - `404 Not Found`: The specified `function_name` does not exist.
  - `500 Internal Server Error`: Wasm panic, compilation error, or internal database failure.
  - `504 Gateway Timeout`: Wasm execution exceeded the allocated fuel budget.

---

## Fuel Metering Logic

To prevent resource exhaustion and provide a deterministic measure of computation, the runner employs **Fuel Metering** via the `wasmtime` engine.

### Configuration

Fuel is enabled in the `Engine` configuration and enforced in the `Store`:

```rust
// Engine Configuration
config.consume_fuel(true);

// Store Limit
store.set_fuel(100_000_000);
```

### Mathematical Breakdown

The fuel consumption is calculated based on the instruction count of the Wasm module during execution.

#### 1. Initial Budget

The total budget $F_{total}$ is fixed at 100 million units.

$$F_{total} = 100,000,000$$

#### 2. Consumption Model

As the Wasm Virtual Machine executes instructions, the remaining fuel $F_{remaining}$ is decremented. The cost of an instruction $c(i)$ is determined by the `wasmtime` compiler (typically 1 unit per basic instruction).

The fuel consumed $F_{consumed}$ after $N$ instructions is:

$$F_{consumed} = \sum_{n=1}^{N} c(i_n)$$

#### 3. Termination Condition

The execution is allowed to continue as long as the remaining fuel is non-negative:

$$F_{remaining} = F_{total} - F_{consumed} \ge 0$$

If $F_{consumed} > F_{total}$, the `wasmtime` engine triggers a **Fuel Trap**, which is caught by the runner and mapped to a `504 Gateway Timeout`.

---

## Resource Limits (Beyond Fuel)

While fuel controls CPU usage, other resources are strictly limited to ensure total sandbox isolation:

| Resource          | Limit | Mechanism                                                    |
| :---------------- | :---- | :----------------------------------------------------------- |
| **Memory**        | 64 MB | `StoreLimits` / `ResourceLimiter`                            |
| **Stdout Buffer** | 1 MB  | `MemoryOutputPipe`                                           |
| **Instances**     | 1     | `StoreLimits` (per request)                                  |
| **Wall Clock**    | N/A   | Implicitly bounded by Fuel (approx. ~2-5s depending on host) |

### Memory Growth Calculation

The memory limit is enforced during the `memory.grow` Wasm instruction:

$$M_{requested} = M_{current} + \Delta M$$
$$Trap \text{ if } M_{requested} > 64 \text{ MiB}$$

This prevents "Memory Hog" guests from impacting the host system or other concurrent executions.
