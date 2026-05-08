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

# Copy only the crates needed for the server
COPY packages/core ./packages/core
COPY apps/server ./apps/server

# Stub out other workspace members so Cargo workspace resolution doesn't fail.
# These crates are not compiled — only their Cargo.toml manifests are needed.
RUN mkdir -p apps/cli/src \
             apps/tui/src \
             apps/desktop/src-tauri/src \
             apps/windows-credprov/src

# apps/cli stub
RUN printf '[package]\nname = "keepassex-cli"\nversion = "0.1.0"\nedition = "2021"\n[[bin]]\nname = "kpx"\npath = "src/main.rs"\n' \
    > apps/cli/Cargo.toml && \
    printf 'fn main() {}\n' > apps/cli/src/main.rs

# apps/tui stub
RUN printf '[package]\nname = "keepassex-tui"\nversion = "0.1.0"\nedition = "2021"\n[[bin]]\nname = "kpx-tui"\npath = "src/main.rs"\n' \
    > apps/tui/Cargo.toml && \
    printf 'fn main() {}\n' > apps/tui/src/main.rs

# apps/desktop/src-tauri stub
RUN printf '[package]\nname = "keepassex-desktop"\nversion = "0.1.0"\nedition = "2021"\n[lib]\nname = "keepassex_desktop_lib"\ncrate-type = ["cdylib","staticlib"]\n' \
    > apps/desktop/src-tauri/Cargo.toml && \
    printf 'pub fn run() {}\n' > apps/desktop/src-tauri/src/lib.rs

# apps/windows-credprov stub
RUN printf '[package]\nname = "keepassex-credprov"\nversion = "0.1.0"\nedition = "2021"\n[lib]\ncrate-type = ["cdylib"]\n' \
    > apps/windows-credprov/Cargo.toml && \
    printf 'pub fn init() {}\n' > apps/windows-credprov/src/lib.rs

# Build only the server binary
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

ENV KPX_PORT=8080
ENV KPX_HOST=0.0.0.0
ENV KPX_DB=/data/keepassex.db
ENV KPX_MAX_VAULT_MB=100

EXPOSE 8080

VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${KPX_PORT}/health || exit 1

ENTRYPOINT ["keepassex-server"]
