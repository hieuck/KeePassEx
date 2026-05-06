//! Tauri commands for favicon / site icon fetching

use crate::state::AppState;
use keepassex_core::favicon::{extract_domain, fetch_favicon, fetch_favicons_batch, FaviconResult};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct FaviconDto {
    /// Base64-encoded icon data
    pub data_base64: String,
    /// MIME type (image/png, image/x-icon, etc.)
    pub mime_type: String,
    /// Domain the icon was fetched for
    pub domain: String,
    /// Which strategy was used
    pub source: String,
}

impl From<FaviconResult> for FaviconDto {
    fn from(r: FaviconResult) -> Self {
        Self {
            data_base64: r.data_base64,
            mime_type: r.mime_type,
            domain: r.domain,
            source: r.source.to_string(),
        }
    }
}

/// Fetch a favicon for a single URL.
/// Only the domain is sent to the favicon service — never the full URL.
#[tauri::command]
pub async fn fetch_entry_favicon(url: String) -> Result<FaviconDto, String> {
    if url.is_empty() {
        return Err("URL is empty".into());
    }

    fetch_favicon(&url)
        .await
        .map(FaviconDto::from)
        .map_err(|e| e.to_string())
}

/// Fetch favicons for all entries in the vault that have URLs.
/// Returns a map of domain → base64 icon data.
/// Skips entries that already have custom icons.
#[tauri::command]
pub async fn fetch_all_favicons(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, FaviconDto>, String> {
    let urls: Vec<String> = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }

        open_vault
            .vault
            .all_entries()
            .filter(|e| !e.url.is_empty() && e.custom_icon_uuid.is_none())
            .map(|e| e.url.clone())
            .collect()
    };

    let results = fetch_favicons_batch(&urls).await;

    Ok(results
        .into_iter()
        .map(|(domain, result)| (domain, FaviconDto::from(result)))
        .collect())
}

/// Extract just the domain from a URL (no network request).
#[tauri::command]
pub fn get_domain_from_url(url: String) -> Option<String> {
    extract_domain(&url)
}
