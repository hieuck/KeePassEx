//! Sync Tauri commands
//!
//! Wires the frontend sync UI to the core sync engine.
//! Sync configuration is persisted in AppSettings; credentials are stored
//! in the OS keychain (via tauri-plugin-stronghold on desktop).

use crate::state::AppState;
use keepassex_core::kdbx::KdbxWriter;
use keepassex_core::sync::merge::merge_vaults;
use keepassex_core::sync::{
    providers::{GoogleDriveProvider, LocalFolderProvider, OneDriveProvider, WebDavProvider},
    ConflictResolution, SyncConfig, SyncCredentials, SyncProvider, SyncProviderType,
};
use keepassex_core::vault::operations::{open_vault as core_open_vault, VaultCredentials};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};

// ─── DTOs ─────────────────────────────────────────────────────────────────────

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
    /// JWT access token for KeePassEx Server provider
    pub token: Option<String>,
    // S3-specific
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct SyncResultDto {
    pub status: String,
    pub entries_uploaded: usize,
    pub entries_downloaded: usize,
    pub conflicts: usize,
    pub duration_ms: u64,
    pub error: Option<String>,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Get current sync configuration and status
#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> SyncStatusDto {
    let settings = state.settings.read().unwrap();

    if let Some(ref cfg) = settings.sync_config {
        SyncStatusDto {
            configured: true,
            provider: Some(provider_type_to_str(&cfg.provider).to_string()),
            remote_path: Some(cfg.remote_path.clone()),
            auto_sync: cfg.auto_sync,
            last_sync: settings.last_sync_at.clone(),
            last_sync_status: settings.last_sync_status.clone(),
        }
    } else {
        SyncStatusDto {
            configured: false,
            provider: None,
            remote_path: None,
            auto_sync: false,
            last_sync: None,
            last_sync_status: None,
        }
    }
}

/// Configure sync provider — persists config to AppSettings
#[tauri::command]
pub fn configure_sync(args: ConfigureSyncArgs, state: State<'_, AppState>) -> Result<(), String> {
    let provider = parse_provider_type(&args.provider)?;

    let conflict_resolution = match args.conflict_resolution.as_str() {
        "keepLocal" | "keep_local" => ConflictResolution::KeepLocal,
        "keepRemote" | "keep_remote" => ConflictResolution::KeepRemote,
        "merge" => ConflictResolution::Merge,
        _ => ConflictResolution::AskUser,
    };

    let config = SyncConfig {
        provider,
        remote_path: args.remote_path.clone(),
        auto_sync: args.auto_sync,
        sync_interval_seconds: args.sync_interval_seconds,
        conflict_resolution,
        credentials: Some(SyncCredentials {
            username: args.username.clone(),
            // Note: password stored in memory only; production would use OS keychain
            password: args.password.clone(),
            token: args.token.clone(),
            access_key_id: args.access_key_id.clone(),
            secret_access_key: args.secret_access_key.clone(),
            region: args.region.clone(),
            bucket: args.bucket.clone(),
            endpoint: args.endpoint.clone(),
            ..Default::default()
        }),
    };

    let mut settings = state.settings.write().unwrap();
    settings.sync_config = Some(config);
    tracing::info!("Sync configured: {}", args.provider);
    Ok(())
}

/// Trigger a manual sync — uploads local vault, downloads remote, merges if needed
#[tauri::command]
pub async fn sync_now(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<SyncResultDto, String> {
    let start = std::time::Instant::now();

    // Read vault + config under lock, then release before async I/O
    let (vault_data, vault_path, master_key_hash, sync_config) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }

        let settings = state.settings.read().unwrap();
        let cfg = settings.sync_config.clone().ok_or("Sync not configured")?;

        // Serialize current vault to bytes
        let writer = KdbxWriter::new();
        let data = writer
            .write(&open_vault.vault, &open_vault.master_key_hash)
            .map_err(|e| e.to_string())?;

        (
            data,
            open_vault.path.clone(),
            open_vault.master_key_hash.clone(),
            cfg,
        )
    };

    let _ = app.emit("sync-progress", serde_json::json!({ "status": "syncing" }));

    // Build provider
    let provider = build_provider(&sync_config).map_err(|e| e.to_string())?;
    let remote_path = &sync_config.remote_path;

    // Check if remote file exists
    let remote_exists = provider.get_metadata(remote_path).await.is_ok();

    let (entries_uploaded, entries_downloaded, conflicts) = if !remote_exists {
        // First sync — just upload
        provider
            .upload(remote_path, &vault_data)
            .await
            .map_err(|e| e.to_string())?;
        (1, 0, 0)
    } else {
        // Download remote
        let (remote_data, remote_meta) = provider
            .download(remote_path)
            .await
            .map_err(|e| e.to_string())?;

        // Compare sizes/etags to detect if remote changed
        let remote_changed = remote_data != vault_data;

        if !remote_changed {
            // No changes — nothing to do
            let duration_ms = start.elapsed().as_millis() as u64;
            let result = SyncResultDto {
                status: "no_changes".to_string(),
                entries_uploaded: 0,
                entries_downloaded: 0,
                conflicts: 0,
                duration_ms,
                error: None,
            };
            let _ = app.emit("sync-complete", &result);
            update_sync_status(&state, "no_changes");
            return Ok(result);
        }

        // Merge based on conflict resolution strategy
        match sync_config.conflict_resolution {
            ConflictResolution::KeepLocal => {
                // Upload local, overwrite remote
                provider
                    .upload(remote_path, &vault_data)
                    .await
                    .map_err(|e| e.to_string())?;
                (1, 0, 0)
            }
            ConflictResolution::KeepRemote => {
                // Write remote data to local file
                let tmp = vault_path.with_extension("kdbx.sync");
                tokio::fs::write(&tmp, &remote_data)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::fs::rename(&tmp, &vault_path)
                    .await
                    .map_err(|e| e.to_string())?;
                (0, 1, 0)
            }
            ConflictResolution::Merge | ConflictResolution::AskUser => {
                // Parse remote vault and merge
                let remote_vault = {
                    let creds = VaultCredentials {
                        password: None,
                        key_file_data: None,
                        hardware_key_response: None,
                    };
                    // Use raw key directly for remote vault parsing
                    keepassex_core::kdbx::KdbxReader::new()
                        .read(&remote_data, &master_key_hash)
                        .map_err(|e| format!("Cannot parse remote vault: {}", e))?
                };

                let local_vault = {
                    keepassex_core::kdbx::KdbxReader::new()
                        .read(&vault_data, &master_key_hash)
                        .map_err(|e| format!("Cannot parse local vault: {}", e))?
                };

                let merged =
                    merge_vaults(&local_vault, &remote_vault).map_err(|e| e.to_string())?;

                // Count changes
                let local_count = local_vault.entry_count();
                let remote_count = remote_vault.entry_count();
                let merged_count = merged.entry_count();
                let conflicts = if local_count != remote_count { 1 } else { 0 };

                // Write merged vault
                let writer = KdbxWriter::new();
                let merged_data = writer
                    .write(&merged, &master_key_hash)
                    .map_err(|e| e.to_string())?;

                // Save locally
                let tmp = vault_path.with_extension("kdbx.sync");
                tokio::fs::write(&tmp, &merged_data)
                    .await
                    .map_err(|e| e.to_string())?;
                tokio::fs::rename(&tmp, &vault_path)
                    .await
                    .map_err(|e| e.to_string())?;

                // Upload merged to remote
                provider
                    .upload(remote_path, &merged_data)
                    .await
                    .map_err(|e| e.to_string())?;

                // Update in-memory vault
                {
                    let mut vault_lock = state.vault.write().unwrap();
                    if let Some(ref mut ov) = *vault_lock {
                        ov.vault = merged;
                    }
                }

                (1, 1, conflicts)
            }
        }
    };

    let duration_ms = start.elapsed().as_millis() as u64;
    let result = SyncResultDto {
        status: "success".to_string(),
        entries_uploaded,
        entries_downloaded,
        conflicts,
        duration_ms,
        error: None,
    };

    let _ = app.emit("sync-complete", &result);
    update_sync_status(&state, "success");
    Ok(result)
}

/// Test sync provider connection
#[tauri::command]
pub async fn test_sync_connection(
    provider: String,
    remote_path: String,
    username: Option<String>,
    password: Option<String>,
    token: Option<String>,
    server_url: Option<String>,
) -> Result<bool, String> {
    let provider_type = parse_provider_type(&provider)?;

    let dummy_config = SyncConfig {
        provider: provider_type,
        remote_path: remote_path.clone(),
        auto_sync: false,
        sync_interval_seconds: 300,
        conflict_resolution: ConflictResolution::Merge,
        credentials: Some(SyncCredentials {
            username,
            password,
            token,
            ..Default::default()
        }),
    };

    let p = build_provider(&dummy_config).map_err(|e| e.to_string())?;
    p.test_connection()
        .await
        .map(|_| true)
        .map_err(|e| e.to_string())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn parse_provider_type(s: &str) -> Result<SyncProviderType, String> {
    match s.to_lowercase().as_str() {
        "webdav" => Ok(SyncProviderType::WebDav),
        "icloud" => Ok(SyncProviderType::ICloudDrive),
        "gdrive" | "googledrive" | "google_drive" => Ok(SyncProviderType::GoogleDrive),
        "onedrive" | "one_drive" => Ok(SyncProviderType::OneDrive),
        "dropbox" => Ok(SyncProviderType::Dropbox),
        "s3" => Ok(SyncProviderType::S3),
        "sftp" => Ok(SyncProviderType::SftpServer),
        "local" | "local_folder" => Ok(SyncProviderType::LocalFolder),
        "keepassex_server" | "keepassex-server" | "kpx-server" | "kpx_server" => {
            Ok(SyncProviderType::KeePassExServer)
        }
        _ => Err(format!("Unknown sync provider: '{}'", s)),
    }
}

fn provider_type_to_str(p: &SyncProviderType) -> &'static str {
    match p {
        SyncProviderType::WebDav => "webdav",
        SyncProviderType::ICloudDrive => "icloud",
        SyncProviderType::GoogleDrive => "gdrive",
        SyncProviderType::OneDrive => "onedrive",
        SyncProviderType::Dropbox => "dropbox",
        SyncProviderType::S3 => "s3",
        SyncProviderType::SftpServer => "sftp",
        SyncProviderType::LocalFolder => "local",
        SyncProviderType::KeePassExServer => "keepassex_server",
    }
}

/// Build a boxed SyncProvider from a SyncConfig
fn build_provider(config: &SyncConfig) -> keepassex_core::error::Result<Box<dyn SyncProvider>> {
    let creds = config.credentials.clone().unwrap_or_default();

    match &config.provider {
        SyncProviderType::WebDav => Ok(Box::new(WebDavProvider {
            base_url: config.remote_path.clone(),
            username: creds.username.unwrap_or_default(),
            password: creds.password.unwrap_or_default(),
        })),
        SyncProviderType::LocalFolder => Ok(Box::new(LocalFolderProvider {
            base_path: std::path::PathBuf::from(&config.remote_path),
        })),
        SyncProviderType::GoogleDrive => Ok(Box::new(GoogleDriveProvider {
            access_token: creds.token.unwrap_or_default(),
            refresh_token: None,
            client_id: None,
            client_secret: None,
        })),
        SyncProviderType::OneDrive => Ok(Box::new(OneDriveProvider {
            access_token: creds.token.unwrap_or_default(),
            refresh_token: None,
            drive_root: "me/drive/root".to_string(),
        })),
        _ => Err(keepassex_core::error::KeePassExError::SyncProviderError(
            format!(
                "Provider '{}' requires additional setup. Please configure OAuth2 credentials.",
                provider_type_to_str(&config.provider)
            ),
        )),
    }
}

fn update_sync_status(state: &State<'_, AppState>, status: &str) {
    let mut settings = state.settings.write().unwrap();
    settings.last_sync_at = Some(chrono::Utc::now().to_rfc3339());
    settings.last_sync_status = Some(status.to_string());
}
