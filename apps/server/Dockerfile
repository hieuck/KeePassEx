# KeePassEx Server — Docker image
# Multi-stage build: builder + minimal runtime

# ─── Stage 1: Build ───────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY packages/core ./packages/core
COPY apps/server ./apps/server

# Build release binary
RUN cargo build --release -p keepassex-server

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false keepassex

# Copy binary
COPY --from=builder /build/target/release/keepassex-server /usr/local/bin/keepassex-server

# Create data directory
RUN mkdir -p /data && chown keepassex:keepassex /data

USER keepassex
WORKDIR /data

# Default environment
ENV KPX_PORT=8080
ENV KPX_HOST=0.0.0.0
ENV KPX_DB=/data/keepassex.db
ENV KPX_MAX_VAULT_MB=100

EXPOSE 8080

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

ENTRYPOINT ["keepassex-server"]
