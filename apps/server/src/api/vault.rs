//! Vault sync API handlers
//!
//! All vault data is end-to-end encrypted by the client.
//! The server stores and retrieves opaque encrypted blobs.
//! The server CANNOT decrypt vault contents.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;

use crate::auth::{extract_bearer_token, validate_token};
use crate::config::ServerConfig;
use crate::error::{Result, ServerError};
use crate::AppState;

#[derive(Serialize)]
pub struct VaultMetaResponse {
    pub version: u32,
    pub size_bytes: u64,
    pub uploaded_at: String,
    pub client_hash: Option<String>,
}

#[derive(Serialize)]
pub struct VaultHistoryResponse {
    pub versions: Vec<VaultMetaResponse>,
}

/// GET /api/v1/vault — get vault metadata (no vault data)
pub async fn get_vault_meta(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<VaultMetaResponse>> {
    let user_id = authenticate(&headers, &state.config)?;

    let meta = state
        .db
        .get_vault_meta(&user_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("No vault found".into()))?;

    Ok(Json(VaultMetaResponse {
        version: meta.version,
        size_bytes: meta.size_bytes,
        uploaded_at: meta.uploaded_at.to_rfc3339(),
        client_hash: meta.client_hash,
    }))
}

/// PUT /api/v1/vault — upload encrypted vault
pub async fn upload_vault(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<VaultMetaResponse>> {
    let user_id = authenticate(&headers, &state.config)?;

    // Check vault size
    let max_bytes = state.config.max_vault_bytes;
    if body.len() as u64 > max_bytes {
        return Err(ServerError::VaultTooLarge {
            max_mb: max_bytes / (1024 * 1024),
        });
    }

    // Extract client hash from header (optional integrity check)
    let client_hash = headers
        .get("x-vault-hash")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let meta = state
        .db
        .upload_vault(
            &user_id,
            &body,
            client_hash.as_deref(),
            ServerConfig::MAX_VAULT_HISTORY,
        )
        .await?;

    // Notify connected WebSocket clients about the update
    // (handled by ws module via broadcast channel)

    tracing::info!(
        "Vault uploaded: user={} version={} size={}B",
        user_id,
        meta.version,
        meta.size_bytes
    );

    Ok(Json(VaultMetaResponse {
        version: meta.version,
        size_bytes: meta.size_bytes,
        uploaded_at: meta.uploaded_at.to_rfc3339(),
        client_hash: meta.client_hash,
    }))
}

/// GET /api/v1/vault/download — download latest encrypted vault
pub async fn download_vault(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(StatusCode, HeaderMap, Bytes)> {
    let user_id = authenticate(&headers, &state.config)?;

    let data = state
        .db
        .download_vault(&user_id)
        .await?
        .ok_or_else(|| ServerError::NotFound("No vault found".into()))?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert("content-type", "application/octet-stream".parse().unwrap());
    response_headers.insert(
        "content-disposition",
        "attachment; filename=\"vault.kdbx\"".parse().unwrap(),
    );

    Ok((StatusCode::OK, response_headers, Bytes::from(data)))
}

/// GET /api/v1/vault/history — list vault versions
pub async fn get_vault_history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<VaultHistoryResponse>> {
    let user_id = authenticate(&headers, &state.config)?;

    let history = state.db.get_vault_history(&user_id).await?;

    Ok(Json(VaultHistoryResponse {
        versions: history
            .into_iter()
            .map(|m| VaultMetaResponse {
                version: m.version,
                size_bytes: m.size_bytes,
                uploaded_at: m.uploaded_at.to_rfc3339(),
                client_hash: m.client_hash,
            })
            .collect(),
    }))
}

/// GET /api/v1/vault/history/:version — download specific vault version
pub async fn get_vault_version(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(version): Path<u32>,
) -> Result<(StatusCode, HeaderMap, Bytes)> {
    let user_id = authenticate(&headers, &state.config)?;

    let data = state
        .db
        .get_vault_version(&user_id, version)
        .await?
        .ok_or_else(|| ServerError::NotFound(format!("Vault version {} not found", version)))?;

    let mut response_headers = HeaderMap::new();
    response_headers.insert("content-type", "application/octet-stream".parse().unwrap());

    Ok((StatusCode::OK, response_headers, Bytes::from(data)))
}

/// Extract and validate JWT from Authorization header
fn authenticate(headers: &HeaderMap, config: &ServerConfig) -> Result<String> {
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(ServerError::Unauthorized)?;

    let token = extract_bearer_token(auth).ok_or(ServerError::Unauthorized)?;
    let claims = validate_token(token, config)?;

    if claims.token_type != "access" {
        return Err(ServerError::Unauthorized);
    }

    Ok(claims.sub)
}
