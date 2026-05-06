//! Sync Tauri commands

use crate::state::AppState;
use keepassex_core::sync::{SyncConfig, SyncProviderType, ConflictResolution};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Clone)]
pub struct SyncStatusDto {
    pub configured: bool,
    pub provider: Option<String>,
    pub remote_path: Option<String>,
    pub auto_sync: bool,
    pub last_sync: Option<String>,
    pub last_sync_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureSyncArgs {
    pub provider: String,
    pub remote_path: String,
    pub auto_sync: bool,
    pub sync_interval_seconds: u64,
    pub conflict_resolution: String,
    // Provider-specific credentials
    pub username: Option<String>,
    pub password: Option<String>,
    pub server_url: Option<String>,
}

/// Get current sync configuration and status
#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> SyncStatusDto {
    let settings = state.settings.read().unwrap();

    // In production: read from persisted sync config
    SyncStatusDto {
        configured: false,
        provider: None,
        remote_path: None,
        auto_sync: false,
        last_sync: None,
        last_sync_status: None,
    }
}

/// Configure sync provider
#[tauri::command]
pub fn configure_sync(
    args: ConfigureSyncArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let provider = match args.provider.to_lowercase().as_str() {
        "webdav" => SyncProviderType::WebDav,
        "icloud" => SyncProviderType::ICloudDrive,
        "gdrive" | "googledrive" => SyncProviderType::GoogleDrive,
        "onedrive" => SyncProviderType::OneDrive,
        "dropbox" => SyncProviderType::Dropbox,
        "s3" => SyncProviderType::S3,
        "sftp" => SyncProviderType::SftpServer,
        "local" => SyncProviderType::LocalFolder,
        _ => return Err(format!("Unknown provider: {}", args.provider)),
    };

    let conflict_resolution = match args.conflict_resolution.as_str() {
        "keepLocal" | "keep_local" => ConflictResolution::KeepLocal,
        "keepRemote" | "keep_remote" => ConflictResolution::KeepRemote,
        "merge" => ConflictResolution::Merge,
        _ => ConflictResolution::AskUser,
    };

    let config = SyncConfig {
        provider,
        remote_path: args.remote_path,
        auto_sync: args.auto_sync,
        sync_interval_seconds: args.sync_interval_seconds,
        conflict_resolution,
    };

    // TODO: persist config and store credentials in OS keychain
    tracing::info!("Sync configured: {:?}", config.provider);
    Ok(())
}

/// Trigger a manual sync
#[tauri::command]
pub async fn sync_now(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<SyncResultDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    // Emit progress event
    let _ = app.emit("sync-progress", serde_json::json!({ "status": "syncing" }));

    // In production: use configured provider
    // For now: return a placeholder result
    let result = SyncResultDto {
        status: "success".to_string(),
        entries_uploaded: 0,
        entries_downloaded: 0,
        conflicts: 0,
        error: None,
    };

    let _ = app.emit("sync-complete", &result);
    Ok(result)
}

/// Test sync provider connection
#[tauri::command]
pub async fn test_sync_connection(
    provider: String,
    remote_path: String,
    username: Option<String>,
    password: Option<String>,
) -> Result<bool, String> {
    match provider.to_lowercase().as_str() {
        "local" => {
            let path = std::path::Path::new(&remote_path);
            Ok(path.exists())
        }
        _ => {
            // In production: test actual connection
            Err(format!("Provider '{}' connection test not yet implemented in this version", provider))
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SyncResultDto {
    pub status: String,
    pub entries_uploaded: usize,
    pub entries_downloaded: usize,
    pub conflicts: usize,
    pub error: Option<String>,
}
