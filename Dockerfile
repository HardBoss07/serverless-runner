# Build stage
FROM rust:1.85-slim-bookworm AS builder
WORKDIR /usr/src/app

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY .sqlx/ ./.sqlx/
COPY crates/ ./crates/
COPY guests/ ./guests/
COPY tests/ ./tests/

# Build the runner and guests
ENV SQLX_OFFLINE=true
RUN mkdir -p guests_compiled && \
    # 1. Build the host runner
    cargo build --release -p serverless-runner && \
    # 2. Add WASI target
    rustup target add wasm32-wasip1 && \
    # 3. Build all Wasm guests in a single Cargo invocation
    cargo build --release --target wasm32-wasip1 \
        -p env-reader \
        -p fibonacci \
        -p fs-reader \
        -p hello-world \
        -p infinite-loop \
        -p long-output-guest \
        -p memory-hog \
        -p net-guest \
        -p panic-guest \
        -p stdout-spammer && \
    # 4. Copy all compiled .wasm files to the execution folder
    cp target/wasm32-wasip1/release/*.wasm guests_compiled/

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/app
COPY --from=builder /usr/src/app/target/release/serverless-runner /usr/app/
COPY --from=builder /usr/src/app/guests_compiled/ /usr/app/guests_compiled/

EXPOSE 8080
CMD ["./serverless-runner"]
