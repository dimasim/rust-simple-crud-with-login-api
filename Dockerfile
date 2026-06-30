# Build stage
FROM rust:1.75-slim AS builder

WORKDIR /usr/src/app

# Install dependencies needed for compiling
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy src/main.rs to build dependencies and cache them
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/my_rust_app*

# Copy actual source and migrations
COPY src ./src
COPY migrations ./migrations

# Build the release binary
RUN cargo build --release

# Runner stage
FROM debian:bookworm-slim

WORKDIR /app

# Install ca-certificates and openssl for PostgreSQL SSL connections if needed
RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*

# Copy the binary from the build stage
COPY --from=builder /usr/src/app/target/release/my_rust_app /app/my_rust_app
COPY migrations /app/migrations

# Expose port
EXPOSE 8080

# Run the app
CMD ["/app/my_rust_app"]
