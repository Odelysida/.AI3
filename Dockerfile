# Build stage
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libclang-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY core/Cargo.toml ./core/
COPY mining/Cargo.toml ./mining/
COPY contracts/Cargo.toml ./contracts/
COPY network/Cargo.toml ./network/
COPY ai3-lib/Cargo.toml ./ai3-lib/

# Create dummy source files to cache dependencies
RUN mkdir -p src core/src mining/src contracts/src network/src ai3-lib/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > src/miner.rs && \
    echo "" > src/lib.rs && \
    echo "" > core/src/lib.rs && \
    echo "" > mining/src/lib.rs && \
    echo "" > contracts/src/lib.rs && \
    echo "" > network/src/lib.rs && \
    echo "" > ai3-lib/src/lib.rs

# Build dependencies
RUN cargo build --release && rm -rf src core/src mining/src contracts/src network/src ai3-lib/src

# Copy actual source code
COPY src ./src
COPY core/src ./core/src
COPY mining/src ./mining/src
COPY contracts/src ./contracts/src
COPY network/src ./network/src
COPY ai3-lib/src ./ai3-lib/src

# Build the application
RUN cargo build --release --bin tribechain --bin ai3-miner

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false tribechain

# Create data directory
RUN mkdir -p /data && chown tribechain:tribechain /data

# Copy binaries from builder stage
COPY --from=builder /app/target/release/tribechain /usr/local/bin/
COPY --from=builder /app/target/release/ai3-miner /usr/local/bin/

# Set permissions
RUN chmod +x /usr/local/bin/tribechain /usr/local/bin/ai3-miner

# Switch to app user
USER tribechain

# Set working directory
WORKDIR /data

# Expose default port
EXPOSE 8333

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD tribechain stats || exit 1

# Default command
CMD ["tribechain", "node", "--data-dir", "/data"] 