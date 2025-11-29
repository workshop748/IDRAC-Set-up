# Build stage
FROM rust:1.83 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY static ./static

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install required runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/idrac-controller /app/idrac-controller
COPY --from=builder /app/static /app/static

# Create data directory for database
RUN mkdir -p /data

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_PATH=/data/idrac.db

# Run the application
CMD ["/app/idrac-controller"]
