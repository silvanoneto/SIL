# ==============================================================================
# Edge Deployment Dockerfile
# ==============================================================================
# Multi-stage build for minimal, optimized edge deployment
# Supports: ARM64 (Raspberry Pi, Jetson) and AMD64 (x86_64)
# ==============================================================================

# ------------------------------------------------------------------------------
# Stage 1: Builder - Compile the Rust binaries
# ------------------------------------------------------------------------------
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy dependency manifests first (for layer caching)
COPY Cargo.toml Cargo.lock ./
COPY sil-core/Cargo.toml ./sil-core/
COPY sil-photonic/Cargo.toml ./sil-photonic/
COPY sil-acoustic/Cargo.toml ./sil-acoustic/
COPY sil-olfactory/Cargo.toml ./sil-olfactory/
COPY sil-gustatory/Cargo.toml ./sil-gustatory/
COPY sil-haptic/Cargo.toml ./sil-haptic/
COPY sil-electronic/Cargo.toml ./sil-electronic/
COPY sil-actuator/Cargo.toml ./sil-actuator/
COPY sil-environment/Cargo.toml ./sil-environment/
COPY sil-network/Cargo.toml ./sil-network/
COPY sil-governance/Cargo.toml ./sil-governance/
COPY sil-swarm/Cargo.toml ./sil-swarm/
COPY sil-orchestration/Cargo.toml ./sil-orchestration/
COPY sil-quantum/Cargo.toml ./sil-quantum/
COPY sil-superposition/Cargo.toml ./sil-superposition/
COPY sil-entanglement/Cargo.toml ./sil-entanglement/
COPY sil-collapse/Cargo.toml ./sil-collapse/
COPY lis-core/Cargo.toml ./lis-core/
COPY lis-cli/Cargo.toml ./lis-cli/
COPY lis-format/Cargo.toml ./lis-format/
COPY lis-runtime/Cargo.toml ./lis-runtime/

# Create stub source files to cache dependencies
RUN mkdir -p sil-core/src sil-photonic/src sil-acoustic/src sil-olfactory/src \
    sil-gustatory/src sil-haptic/src sil-electronic/src sil-actuator/src \
    sil-environment/src sil-network/src sil-governance/src sil-swarm/src \
    sil-orchestration/src sil-quantum/src sil-superposition/src \
    sil-entanglement/src sil-collapse/src lis-core/src lis-cli/src \
    lis-format/src lis-runtime/src && \
    echo "fn main() {}" > lis-cli/src/main.rs && \
    find . -name "Cargo.toml" -not -path "./Cargo.toml" -exec sh -c 'echo "// stub" > $(dirname {})/src/lib.rs' \;

# Build dependencies only (this layer will be cached)
RUN cargo build --release --bin lis

# Remove stub files
RUN find . -type f -name "*.rs" -delete

# Copy actual source code
COPY . .

# Build release binaries with optimizations for size and edge deployment
ENV RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat -C codegen-units=1 -C strip=symbols"
RUN cargo build --release --bin lis --bin lis-api

# Strip binaries further (edge deployment optimization)
RUN strip /build/target/release/lis /build/target/release/lis-api

# ------------------------------------------------------------------------------
# Stage 2: Runtime - Minimal image with just the binaries
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd -m -u 1000 -s /bin/bash sil

# Create application directories
RUN mkdir -p /app/bin /app/examples /app/data && \
    chown -R sil:sil /app

# Copy binaries from builder
COPY --from=builder --chown=sil:sil /build/target/release/lis /app/bin/
COPY --from=builder --chown=sil:sil /build/target/release/lis-api /app/bin/

# Copy example files
COPY --chown=sil:sil lis-cli/examples/ /app/examples/

# Set working directory
WORKDIR /app

# Switch to non-root user
USER sil

# Add binary to PATH
ENV PATH="/app/bin:${PATH}"

# Health check (for orchestration systems)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["lis", "--version"]

# Default command - show help
CMD ["lis", "--help"]

# ------------------------------------------------------------------------------
# Stage 3: API Server - REST API service for LIS
# ------------------------------------------------------------------------------
FROM debian:bookworm-slim AS api

# Install runtime dependencies only
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN useradd -m -u 1000 -s /bin/bash sil

# Create application directories
RUN mkdir -p /app/bin /app/data && \
    chown -R sil:sil /app

# Copy API binary from builder
COPY --from=builder --chown=sil:sil /build/target/release/lis-api /app/bin/

# Set working directory
WORKDIR /app

# Switch to non-root user
USER sil

# Add binary to PATH
ENV PATH="/app/bin:${PATH}"

# API server configuration
ENV LIS_API_HOST=0.0.0.0
ENV LIS_API_PORT=3000
ENV RUST_LOG=info

# Expose API port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Default command - start API server
CMD ["lis-api", "--host", "0.0.0.0", "--port", "3000"]

# ------------------------------------------------------------------------------
# Stage 4: Development - Includes Rust toolchain for edge development
# ------------------------------------------------------------------------------
FROM rust:1.75-slim AS development

# Install development dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    git \
    vim \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash sil

# Set working directory
WORKDIR /workspace

# Copy project files
COPY --chown=sil:sil . .

# Switch to non-root user
USER sil

# Install cargo tools for development
RUN cargo install cargo-watch cargo-edit

# Default command - interactive shell
CMD ["/bin/bash"]
