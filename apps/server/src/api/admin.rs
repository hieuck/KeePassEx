//! Admin API handlers — protected by admin API key
//!
//! Only available when --admin flag is set and KPX_ADMIN_KEY is configured.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use serde::Serialize;

use crate::error::{Result, ServerError};
use crate::AppState;

#[derive(Serialize)]
pub struct UserSummary {
    pub id: String,
    pub email: String,
    pub created_at: String,
    pub last_login: Option<String>,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub user_count: u64,
    pub vault_count: u64,
    pub total_storage_mb: f64,
    pub server_version: &'static str,
}

/// GET /api/v1/admin/users
pub async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<UserSummary>>> {
    require_admin(&headers, &state)?;

    let users = state.db.list_users().await?;

    Ok(Json(
        users
            .into_iter()
            .map(|u| UserSummary {
                id: u.id,
                email: u.email,
                created_at: u.created_at.to_rfc3339(),
                last_login: u.last_login.map(|t| t.to_rfc3339()),
            })
            .collect(),
    ))
}

/// DELETE /api/v1/admin/users/:id
pub async fn delete_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    require_admin(&headers, &state)?;

    let deleted = state.db.delete_user(&user_id).await?;
    if !deleted {
        return Err(ServerError::NotFound(format!("User {} not found", user_id)));
    }

    tracing::info!("Admin deleted user: {}", user_id);

    Ok(Json(serde_json::json!({ "message": "User deleted" })))
}

/// GET /api/v1/admin/stats
pub async fn server_stats(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<StatsResponse>> {
    require_admin(&headers, &state)?;

    let stats = state.db.get_stats().await?;

    Ok(Json(StatsResponse {
        user_count: stats.user_count,
        vault_count: stats.vault_count,
        total_storage_mb: stats.total_storage_bytes as f64 / (1024.0 * 1024.0),
        server_version: env!("CARGO_PKG_VERSION"),
    }))
}

/// Verify the admin API key from the X-Admin-Key header
fn require_admin(headers: &HeaderMap, state: &AppState) -> Result<()> {
    if !state.config.admin_enabled {
        return Err(ServerError::Forbidden);
    }

    let expected_key = state
        .config
        .admin_key
        .as_deref()
        .ok_or(ServerError::Forbidden)?;

    let provided_key = headers
        .get("x-admin-key")
        .and_then(|v| v.to_str().ok())
        .ok_or(ServerError::Forbidden)?;

    // Constant-time comparison to prevent timing attacks
    if !constant_time_eq(provided_key.as_bytes(), expected_key.as_bytes()) {
        return Err(ServerError::Forbidden);
    }

    Ok(())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}
