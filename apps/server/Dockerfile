# KeePassEx Server — Docker image
# Multi-stage build: builder + minimal runtime
# Build context: repository root (docker build -f apps/server/Dockerfile .)

# ─── Stage 1: Build ───────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace manifest and lock file first (for layer caching)
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members (needed for Cargo workspace resolution)
# Core library
COPY packages/core ./packages/core

# Server app
COPY apps/server ./apps/server

# Stub out other workspace members so Cargo doesn't fail on missing paths
# These are not needed for the server build but must exist in the workspace
RUN mkdir -p apps/cli/src apps/tui/src apps/desktop/src-tauri/src apps/windows-credprov/src && \
    echo '[package]\nname = "keepassex-cli"\nversion = "0.1.0"\nedition = "2021"\n[[bin]]\nname = "kpx"\npath = "src/main.rs"' > apps/cli/Cargo.toml && \
    echo 'fn main() {}' > apps/cli/src/main.rs && \
    echo '[package]\nname = "keepassex-tui"\nversion = "0.1.0"\nedition = "2021"\n[[bin]]\nname = "kpx-tui"\npath = "src/main.rs"' > apps/tui/Cargo.toml && \
    echo 'fn main() {}' > apps/tui/src/main.rs && \
    echo '[package]\nname = "keepassex-desktop"\nversion = "0.1.0"\nedition = "2021"\n[lib]\nname = "keepassex_desktop_lib"\ncrate-type = ["cdylib","staticlib"]' > apps/desktop/src-tauri/Cargo.toml && \
    echo 'pub fn run() {}' > apps/desktop/src-tauri/src/lib.rs && \
    echo '[package]\nname = "keepassex-credprov"\nversion = "0.1.0"\nedition = "2021"\n[lib]\ncrate-type = ["cdylib"]' > apps/windows-credprov/Cargo.toml && \
    echo 'pub fn init() {}' > apps/windows-credprov/src/lib.rs

# Build only the server (not the whole workspace)
RUN cargo build --release -p keepassex-server

# ─── Stage 2: Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false keepassex

# Copy binary from builder
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
    CMD curl -f http://localhost:${KPX_PORT}/health || exit 1

ENTRYPOINT ["keepassex-server"]
