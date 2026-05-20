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
RUN mkdir -p guests_compiled && \
    cargo build --release -p serverless-runner && \
    rustup target add wasm32-wasip1 && \
    cargo build --release -p hello-world --target wasm32-wasip1 && \
    cp target/wasm32-wasip1/release/hello-world.wasm guests_compiled/ && \
    cargo build --release -p fibonacci --target wasm32-wasip1 && \
    ls -l target/wasm32-wasip1/release/fibonacci.wasm && \
    cp target/wasm32-wasip1/release/fibonacci.wasm guests_compiled/

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
