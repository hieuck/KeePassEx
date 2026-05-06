//! Scheduled Backup Tauri commands

use crate::state::AppState;
use keepassex_core::scheduled_backup::{
    delete_backup, is_backup_due, list_backups, perform_backup, restore_from_backup, BackupConfig,
    BackupRecord, BackupResult,
};
use tauri::State;

#[tauri::command]
pub async fn get_backup_config(state: State<'_, AppState>) -> Result<BackupConfig, String> {
    let settings = state.settings.read().unwrap();
    Ok(settings.backup_config.clone().unwrap_or_default())
}

#[tauri::command]
pub async fn save_backup_config(
    config: BackupConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings = state.settings.write().unwrap();
    settings.backup_config = Some(config);
    Ok(())
}

#[tauri::command]
pub async fn backup_now(state: State<'_, AppState>) -> Result<BackupResult, String> {
    let (vault_path, mut config) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        let settings = state.settings.read().unwrap();
        let config = settings.backup_config.clone().unwrap_or_default();
        (open_vault.path.clone(), config)
    };

    let result = perform_backup(&vault_path, &mut config)
        .await
        .map_err(|e| e.to_string())?;

    // Update last_backup_at in settings
    if result.success {
        let mut settings = state.settings.write().unwrap();
        if let Some(ref mut bc) = settings.backup_config {
            bc.last_backup_at = config.last_backup_at;
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn list_backups_cmd(state: State<'_, AppState>) -> Result<Vec<BackupRecord>, String> {
    let (vault_path, config) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        let settings = state.settings.read().unwrap();
        let config = settings.backup_config.clone().unwrap_or_default();
        (open_vault.path.clone(), config)
    };

    if config.destination.is_empty() {
        return Ok(vec![]);
    }

    let vault_stem = vault_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("vault");

    list_backups(std::path::Path::new(&config.destination), vault_stem)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_from_backup_cmd(
    backup_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault_path = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        open_vault.path.clone()
    };

    restore_from_backup(std::path::Path::new(&backup_path), &vault_path)
        .await
        .map_err(|e| e.to_string())?;

    // Close the vault so user must re-open
    let mut vault_lock = state.vault.write().unwrap();
    *vault_lock = None;

    Ok(())
}

#[tauri::command]
pub async fn delete_backup_cmd(backup_path: String) -> Result<(), String> {
    delete_backup(std::path::Path::new(&backup_path))
        .await
        .map_err(|e| e.to_string())
}
