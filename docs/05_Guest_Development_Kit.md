# 05 - Guest Development Kit

## 5.1 The ABI Contract

Guests are compiled for the `wasm32-wasi` target. This target provides a standard set of system calls (WASI) for reading input, writing output, and exiting the process.

### Rules of Engagement

1. **Entry Point:** Use the standard Rust `fn main()`. This maps to the `_start` export.
2. **Payload Receipt:** Read the HTTP request body from `std::io::stdin()`.
3. **Response Delivery:** Write the HTTP response body to `std::io::stdout()`.
4. **Error Reporting:** Write logs/errors to `std::io::stderr()`. These are captured by the host runner's console.
5. **Exit Status:** Return `Ok(())` for success (Status Code 0) or an error for failure (Status Code 1+).

## 5.2 Building a Guest (Example: `hello-world`)

Create a new project in the `guests/` folder.

```bash
cargo new guests/hello-world
cd guests/hello-world
```

### Guest Implementation (`src/main.rs`)

```rust
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    // Read the raw input (provided by the Axum POST body)
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    // Process the input
    let name = if buffer.trim().is_empty() { "Guest" } else { buffer.trim() };
    let response = format!("Hello, {}! (Rendered by Wasmtime)\n", name);

    // Write the output (becomes the HTTP response body)
    io::stdout().write_all(response.as_bytes())?;

    Ok(())
}
```

## 5.3 Compilation and Installation

Guests must be compiled using the `wasm32-wasi` target to be compatible with the Runner.

### One-Time Setup:

```bash
rustup target add wasm32-wasi
```

### Build Command:

```bash
# Debug Build
cargo build --target wasm32-wasi

# Release Build (Recommended for benchmarks)
cargo build --target wasm32-wasi --release
```

### Installation (Deploying to Host):

Move the resulting `.wasm` binary to the host's `guests_compiled` folder.

```bash
mkdir -p ../../guests_compiled
cp target/wasm32-wasi/debug/hello-world.wasm ../../guests_compiled/
```

## 5.4 Best Practices

- **No Direct Networking:** Guests cannot open TCP/UDP sockets. All data must be passed in via `stdin`.
- **Statelessness:** Guests are re-instantiated on every request. Do not expect global variables to persist.
- **Size Optimization:** Avoid large dependencies (like GUI crates). Smaller `.wasm` files load and instantiate faster.
- **Panics:** A panic in the guest will exit the WASI environment and return a non-zero status code to the Runner. The Runner will log this failure.
