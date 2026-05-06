//! Vault Tauri commands

use crate::state::{AppState, OpenVault};
use keepassex_core::{
    vault::operations::{open_vault as core_open_vault, save_vault as core_save_vault, VaultCredentials},
    Vault,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct OpenVaultArgs {
    pub path: String,
    pub password: Option<String>,
    pub key_file_data: Option<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVaultArgs {
    pub path: String,
    pub name: String,
    pub password: String,
    pub key_file_data: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct VaultMetaDto {
    pub name: String,
    pub description: String,
    pub entry_count: usize,
    pub group_count: usize,
    pub path: String,
}

/// Open an existing KDBX vault
#[tauri::command]
pub async fn open_vault(
    args: OpenVaultArgs,
    state: State<'_, AppState>,
) -> Result<VaultMetaDto, String> {
    let path = std::path::Path::new(&args.path);

    let credentials = VaultCredentials {
        password: args.password,
        key_file_data: args.key_file_data,
        hardware_key_response: None,
    };

    let vault = core_open_vault(path, &credentials)
        .await
        .map_err(|e| e.to_string())?;

    let meta = VaultMetaDto {
        name: vault.meta.name.clone(),
        description: vault.meta.description.clone(),
        entry_count: vault.entry_count(),
        group_count: vault.group_count(),
        path: args.path.clone(),
    };

    let composite_key = credentials
        .build_composite_key()
        .map_err(|e| e.to_string())?;
    let raw_key = composite_key.build().map_err(|e| e.to_string())?;

    let mut vault_lock = state.vault.write().unwrap();
    *vault_lock = Some(OpenVault {
        vault,
        path: path.to_path_buf(),
        master_key_hash: raw_key,
        locked: false,
    });

    // Add to recent vaults
    let mut settings = state.settings.write().unwrap();
    settings.recent_vaults.retain(|v| v.path != args.path);
    settings.recent_vaults.insert(
        0,
        crate::state::RecentVault {
            path: args.path,
            name: meta.name.clone(),
            last_opened: chrono::Utc::now(),
        },
    );
    settings.recent_vaults.truncate(10);

    Ok(meta)
}

/// Create a new KDBX vault
#[tauri::command]
pub async fn create_vault(
    args: CreateVaultArgs,
    state: State<'_, AppState>,
) -> Result<VaultMetaDto, String> {
    let vault = Vault::new(&args.name);
    let path = std::path::Path::new(&args.path);

    let credentials = VaultCredentials {
        password: Some(args.password),
        key_file_data: args.key_file_data,
        hardware_key_response: None,
    };

    core_save_vault(&vault, path, &credentials)
        .await
        .map_err(|e| e.to_string())?;

    let meta = VaultMetaDto {
        name: vault.meta.name.clone(),
        description: vault.meta.description.clone(),
        entry_count: 0,
        group_count: 1,
        path: args.path,
    };

    let composite_key = credentials
        .build_composite_key()
        .map_err(|e| e.to_string())?;
    let raw_key = composite_key.build().map_err(|e| e.to_string())?;

    let mut vault_lock = state.vault.write().unwrap();
    *vault_lock = Some(OpenVault {
        vault,
        path: path.to_path_buf(),
        master_key_hash: raw_key,
        locked: false,
    });

    Ok(meta)
}

/// Close the current vault
#[tauri::command]
pub fn close_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    *vault_lock = None;
    Ok(())
}

/// Save the current vault using the stored master key
#[tauri::command]
pub async fn save_vault(state: State<'_, AppState>) -> Result<(), String> {
    // Read vault data and path without holding the lock during async I/O
    let (vault_snapshot, path, master_key_hash) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }
        (
            // We need to serialize the vault — clone the relevant data
            open_vault.vault.meta.name.clone(),
            open_vault.path.clone(),
            open_vault.master_key_hash.clone(),
        )
    };

    // Use the stored composite key hash directly to re-save
    // The master_key_hash IS the raw composite key (SHA-256 of password components)
    // We pass it through a dummy VaultCredentials that uses it directly
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    // Build credentials from stored key — use a special "raw key" path
    // In production this would use OS keychain; here we use the stored hash
    let credentials = VaultCredentials {
        password: None,
        key_file_data: None,
        hardware_key_response: None,
    };

    // Write vault using the raw key directly
    use keepassex_core::kdbx::KdbxWriter;
    use keepassex_core::crypto::kdf::{KdfParams, ArgonParams, derive_master_key};
    use keepassex_core::crypto::keys::MasterKey;
    use keepassex_core::crypto::cipher::{Cipher, CipherAlgorithm};
    use keepassex_core::crypto::hmac::compute_header_hmac;
    use rand::RngCore;

    let writer = KdbxWriter::new();
    let data = writer.write(&open_vault.vault, &master_key_hash)
        .map_err(|e| e.to_string())?;

    // Atomic write
    let tmp = path.with_extension("kdbx.tmp");
    tokio::fs::write(&tmp, &data).await.map_err(|e| e.to_string())?;
    tokio::fs::rename(&tmp, &path).await.map_err(|e| e.to_string())?;

    // Clear dirty flag
    drop(vault_lock);
    if let Some(ref mut ov) = *state.vault.write().unwrap() {
        ov.vault.dirty = false;
    }

    Ok(())
}

/// Lock the vault (keep in memory but require re-auth)
#[tauri::command]
pub fn lock_vault(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    if let Some(ref mut open_vault) = *vault_lock {
        open_vault.locked = true;
        // Zero out sensitive data
        use zeroize::Zeroize;
        open_vault.master_key_hash.zeroize();
    }
    Ok(())
}

/// Change vault master password
#[tauri::command]
pub async fn change_credentials(
    old_password: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let old_creds = VaultCredentials::password_only(old_password);
    let new_creds = VaultCredentials::password_only(new_password);

    keepassex_core::vault::operations::change_credentials(
        &open_vault.vault,
        &open_vault.path,
        &old_creds,
        &new_creds,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Get vault metadata
#[tauri::command]
pub fn get_vault_meta(state: State<'_, AppState>) -> Result<VaultMetaDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    Ok(VaultMetaDto {
        name: open_vault.vault.meta.name.clone(),
        description: open_vault.vault.meta.description.clone(),
        entry_count: open_vault.vault.entry_count(),
        group_count: open_vault.vault.group_count(),
        path: open_vault.path.to_string_lossy().to_string(),
    })
}
