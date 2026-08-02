# syntax=docker/dockerfile:1

# ---- Stage 1: Chef — prepare the recipe for dependency caching ----
FROM rust:bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# ---- Stage 2: Planner — extract dependency info from Cargo.toml/lock ----
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ---- Stage 3: Builder — compile dependencies, then the app ----
FROM chef AS builder

# System dependencies for the heic crate
RUN apt-get update && apt-get install -y libheif-dev && rm -rf /var/lib/apt/lists/*

# Copy the recipe and build dependencies first (cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy the actual source and build the app
COPY . .
RUN cargo build --release --bin metallian-photos

# ---- Stage 4: Runtime — slim image with only what's needed to run ----
FROM debian:bookworm-slim AS runtime

# Runtime system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libheif1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/metallian-photos /app/metallian-photos

# Copy runtime files that are read from disk
COPY --from=builder /app/templates /app/templates
COPY --from=builder /app/static /app/static
COPY --from=builder /app/configuration /app/configuration
COPY --from=builder /app/migrations /app/migrations

# Data volume for SQLite
RUN mkdir -p /data

EXPOSE 8000

ENTRYPOINT ["/app/metallian-photos"]
