//! KeePassEx Server — Self-hosted sync server
//!
//! A lightweight Rust server that enables teams and individuals to sync
//! their KeePassEx vaults without relying on third-party cloud services.
//!
//! # Key design principles
//! - **Zero-knowledge**: The server never sees plaintext passwords or vault contents
//! - **End-to-end encrypted**: Vaults are encrypted client-side before upload
//! - **Self-hostable**: Single binary, SQLite database, no external dependencies
//! - **Real-time sync**: WebSocket-based live sync across all connected clients
//!
//! # Competitor gap
//! - KeePass/KeePassXC: no server, manual file sync only
//! - Keepassium: iCloud/Dropbox only, no self-hosted option
//! - KeePass2Android: no server
//! - Bitwarden: has a server but requires complex setup + cloud account
//! - KeePassEx: single binary, zero-knowledge, self-hosted ✅
//!
//! # Quick start
//! ```bash
//! keepassex-server --port 8080 --db ./keepassex.db
//! ```

mod api;
mod auth;
mod config;
mod db;
mod error;
mod ws;

use anyhow::Result;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::ServerConfig;
use crate::db::Database;

/// KeePassEx Server — self-hosted vault sync
#[derive(Parser)]
#[command(
    name = "keepassex-server",
    about = "KeePassEx self-hosted sync server",
    version,
    long_about = "KeePassEx Server enables zero-knowledge, end-to-end encrypted vault sync.\n\nThe server never sees your passwords or vault contents — all encryption\nhappens on the client before data is sent to the server."
)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "8080", env = "KPX_PORT")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "0.0.0.0", env = "KPX_HOST")]
    host: String,

    /// SQLite database path
    #[arg(long, default_value = "./keepassex.db", env = "KPX_DB")]
    db: String,

    /// JWT secret (auto-generated if not set)
    #[arg(long, env = "KPX_JWT_SECRET")]
    jwt_secret: Option<String>,

    /// Maximum vault size in MB
    #[arg(long, default_value = "100", env = "KPX_MAX_VAULT_MB")]
    max_vault_mb: u64,

    /// Enable admin API (default: disabled)
    #[arg(long, env = "KPX_ADMIN_ENABLED")]
    admin: bool,

    /// Admin API key (required if --admin is set)
    #[arg(long, env = "KPX_ADMIN_KEY")]
    admin_key: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "keepassex_server=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Build config
    let jwt_secret = cli.jwt_secret.unwrap_or_else(|| {
        let secret = generate_random_secret();
        tracing::warn!(
            "No JWT secret provided — using random secret. Set KPX_JWT_SECRET for persistence."
        );
        secret
    });

    let config = Arc::new(ServerConfig {
        jwt_secret,
        max_vault_bytes: cli.max_vault_mb * 1024 * 1024,
        admin_enabled: cli.admin,
        admin_key: cli.admin_key,
    });

    // Initialize database
    tracing::info!("Opening database: {}", cli.db);
    let db = Arc::new(Database::new(&cli.db).await?);
    db.migrate().await?;

    // Build router
    let app = build_router(config, db);

    // Start server
    let addr = format!("{}:{}", cli.host, cli.port);
    tracing::info!("KeePassEx Server listening on http://{}", addr);
    tracing::info!("WebSocket sync endpoint: ws://{}/ws", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_router(config: Arc<ServerConfig>, db: Arc<Database>) -> Router {
    let state = AppState { config, db };

    Router::new()
        // ─── Auth ─────────────────────────────────────────────────────────────
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login))
        .route("/api/v1/auth/refresh", post(api::auth::refresh_token))
        .route("/api/v1/auth/logout", post(api::auth::logout))
        // ─── Vault sync ───────────────────────────────────────────────────────
        .route("/api/v1/vault", get(api::vault::get_vault_meta))
        .route("/api/v1/vault", put(api::vault::upload_vault))
        .route("/api/v1/vault/download", get(api::vault::download_vault))
        .route("/api/v1/vault/history", get(api::vault::get_vault_history))
        .route(
            "/api/v1/vault/history/:version",
            get(api::vault::get_vault_version),
        )
        // ─── WebSocket real-time sync ─────────────────────────────────────────
        .route("/ws", get(ws::handler::ws_handler))
        // ─── Health check ─────────────────────────────────────────────────────
        .route("/health", get(api::health::health_check))
        .route("/api/v1/server/info", get(api::health::server_info))
        // ─── Admin (optional) ─────────────────────────────────────────────────
        .route("/api/v1/admin/users", get(api::admin::list_users))
        .route("/api/v1/admin/users/:id", delete(api::admin::delete_user))
        .route("/api/v1/admin/stats", get(api::admin::server_stats))
        // ─── Middleware ───────────────────────────────────────────────────────
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub db: Arc<Database>,
}

fn generate_random_secret() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}
