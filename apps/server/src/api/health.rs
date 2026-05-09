//! Health check and server info endpoints

use axum::{extract::State, Json};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_seconds: u64,
}

#[derive(Serialize)]
pub struct ServerInfoResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub user_count: u64,
    pub vault_count: u64,
    pub total_storage_mb: f64,
    pub features: ServerFeatures,
}

#[derive(Serialize)]
pub struct ServerFeatures {
    pub zero_knowledge: bool,
    pub end_to_end_encrypted: bool,
    pub websocket_sync: bool,
    pub vault_versioning: bool,
    pub admin_api: bool,
    pub rate_limiting: bool,
}

// Server start time (set once at startup)
static SERVER_START: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

pub fn init_start_time() {
    SERVER_START.get_or_init(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });
}

/// GET /health — lightweight health check for load balancers
pub async fn health_check() -> Json<HealthResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let start = SERVER_START.get().copied().unwrap_or(now);

    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        uptime_seconds: now.saturating_sub(start),
    })
}

/// GET /api/v1/server/info — detailed server information
pub async fn server_info(State(state): State<AppState>) -> Result<Json<ServerInfoResponse>> {
    let stats = state.db.get_stats().await?;

    Ok(Json(ServerInfoResponse {
        name: "KeePassEx Server",
        version: env!("CARGO_PKG_VERSION"),
        user_count: stats.user_count,
        vault_count: stats.vault_count,
        total_storage_mb: stats.total_storage_bytes as f64 / (1024.0 * 1024.0),
        features: ServerFeatures {
            zero_knowledge: true,
            end_to_end_encrypted: true,
            websocket_sync: true,
            vault_versioning: true,
            admin_api: state.config.admin_enabled,
            rate_limiting: true,
        },
    }))
}
