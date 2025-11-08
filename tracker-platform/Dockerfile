# ================================
# Stage 1: Build Dependencies
# ================================
FROM rust:1.75-slim-bookworm AS chef

# Install cargo-chef for dependency caching
RUN cargo install cargo-chef

WORKDIR /app

# ================================
# Stage 2: Plan Dependencies
# ================================
FROM chef AS planner

# Copy all source files
COPY . .

# Analyze dependencies
RUN cargo chef prepare --recipe-path recipe.json

# ================================
# Stage 3: Build Dependencies
# ================================
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libsasl2-dev \
    libzstd-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency recipe
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies (this layer is cached)
RUN cargo chef cook --release --recipe-path recipe.json

# ================================
# Stage 4: Build Application
# ================================
FROM builder AS build

# Copy entire source tree
COPY . .

# Build the application
RUN cargo build --release --bin tracker-platform

# Strip debug symbols to reduce binary size
RUN strip /app/target/release/tracker-platform

# ================================
# Stage 5: Runtime
# ================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    libsasl2-2 \
    libzstd1 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 -s /bin/bash app

WORKDIR /app

# Copy binary from build stage
COPY --from=build /app/target/release/tracker-platform /app/tracker-platform

# Copy migrations
COPY --from=build /app/migrations /app/migrations

# Create necessary directories
RUN mkdir -p /app/uploads /app/config && \
    chown -R app:app /app

# Switch to app user
USER app

# Expose application port
EXPOSE 8080

# Expose metrics port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
CMD ["/app/tracker-platform"]

# ================================
# Stage 6: Development (Optional)
# ================================
FROM rust:1.75-slim-bookworm AS development

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    libsasl2-dev \
    libzstd-dev \
    cmake \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install cargo watch for hot reloading
RUN cargo install cargo-watch

WORKDIR /app

# Copy source
COPY . .

# Expose application and metrics ports
EXPOSE 8080 9090

# Development command with hot reload
CMD ["cargo", "watch", "-x", "run --bin tracker-platform"]
