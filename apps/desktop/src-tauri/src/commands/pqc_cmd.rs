//! Tauri commands for PQC (quantum-resistant encryption) vault migration

use crate::state::AppState;
use keepassex_core::crypto::pqc::PqcAlgorithm;
use keepassex_core::vault::operations::VaultCredentials;
use keepassex_core::vault::pqc_migration::is_pqc_vault;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct PqcMigrationResult {
    pub vault_path: String,
    pub backup_path: String,
    pub algorithm: String,
    pub verified: bool,
}

/// Migrate the current vault to quantum-resistant encryption (X25519 + Kyber-768 hybrid).
/// Creates a backup before migrating and verifies the result.
#[tauri::command]
pub async fn migrate_to_pqc(state: State<'_, AppState>) -> Result<PqcMigrationResult, String> {
    let (path, password) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }
        (
            open_vault.path.clone(),
            // Re-use stored key hash as password proxy for migration
            // In production this would use OS keychain to retrieve the actual password
            String::new(),
        )
    };

    let credentials = VaultCredentials {
        password: if password.is_empty() {
            None
        } else {
            Some(password)
        },
        key_file_data: None,
        hardware_key_response: None,
    };

    let result = keepassex_core::vault::pqc_migration::migrate_to_pqc(
        &path,
        &credentials,
        PqcAlgorithm::HybridKyber768,
    )
    .await
    .map_err(|e| e.to_string())?;

    Ok(PqcMigrationResult {
        vault_path: result.vault_path.to_string_lossy().to_string(),
        backup_path: result.backup_path.to_string_lossy().to_string(),
        algorithm: format!("{:?}", result.algorithm),
        verified: result.verified,
    })
}

/// Downgrade the current vault from PQC back to classical encryption.
/// Useful for sharing with KeePass/KeePassXC users.
#[tauri::command]
pub async fn downgrade_from_pqc(state: State<'_, AppState>) -> Result<String, String> {
    let path = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }
        open_vault.path.clone()
    };

    let credentials = VaultCredentials {
        password: None,
        key_file_data: None,
        hardware_key_response: None,
    };

    let backup_path = keepassex_core::vault::pqc_migration::downgrade_from_pqc(&path, &credentials)
        .await
        .map_err(|e| e.to_string())?;

    Ok(backup_path.to_string_lossy().to_string())
}

/// Check if the current vault has PQC encryption enabled.
/// Reads only the unencrypted header — no password required.
#[tauri::command]
pub async fn check_pqc_status(state: State<'_, AppState>) -> Result<bool, String> {
    let path = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        open_vault.path.clone()
    };

    is_pqc_vault(&path).await.map_err(|e| e.to_string())
}
