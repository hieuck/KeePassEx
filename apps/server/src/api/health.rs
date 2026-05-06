//! Health check and server info endpoints

use axum::{extract::State, Json};
use serde::Serialize;

use crate::error::Result;
use crate::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ServerInfoResponse {
    pub name: &'static str,
    pub version: &'static str,
    pub user_count: u64,
    pub vault_count: u64,
    pub total_storage_mb: f64,
}

/// GET /health
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /api/v1/server/info
pub async fn server_info(State(state): State<AppState>) -> Result<Json<ServerInfoResponse>> {
    let stats = state.db.get_stats().await?;

    Ok(Json(ServerInfoResponse {
        name: "KeePassEx Server",
        version: env!("CARGO_PKG_VERSION"),
        user_count: stats.user_count,
        vault_count: stats.vault_count,
        total_storage_mb: stats.total_storage_bytes as f64 / (1024.0 * 1024.0),
    }))
}
