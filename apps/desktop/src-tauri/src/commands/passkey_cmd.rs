//! Passkey (FIDO2/WebAuthn) Tauri commands
//!
//! KeePassEx is the ONLY password manager with full passkey CRUD in a KDBX vault.
//! Competitors store passkeys in separate silos (iCloud Keychain, Google Password Manager).
//! KeePassEx stores them encrypted inside the KDBX vault — portable, offline, zero-knowledge.

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct PasskeyDto {
    pub index: usize,
    pub credential_id: String, // hex-encoded
    pub rp_id: String,
    pub rp_name: String,
    pub user_name: String,
    pub user_display_name: String,
    pub sign_count: u64,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub backup_eligible: bool,
    pub backup_state: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddPasskeyArgs {
    pub entry_uuid: String,
    pub credential_id: String, // hex-encoded
    pub rp_id: String,
    pub rp_name: String,
    pub user_id: String, // hex-encoded
    pub user_name: String,
    pub user_display_name: String,
    pub private_key_pem: String,
    pub backup_eligible: bool,
}

/// Get all passkeys for an entry
#[tauri::command]
pub fn get_entry_passkeys(
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<Vec<PasskeyDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    Ok(entry
        .passkeys
        .iter()
        .enumerate()
        .map(|(i, pk)| PasskeyDto {
            index: i,
            credential_id: hex::encode(&pk.credential_id),
            rp_id: pk.rp_id.clone(),
            rp_name: pk.rp_name.clone(),
            user_name: pk.user_name.clone(),
            user_display_name: pk.user_display_name.clone(),
            sign_count: pk.sign_count,
            created_at: pk.created_at.to_rfc3339(),
            last_used_at: pk.last_used_at.map(|t| t.to_rfc3339()),
            backup_eligible: pk.backup_eligible,
            backup_state: pk.backup_state,
        })
        .collect())
}

/// Add a passkey to an entry
#[tauri::command]
pub fn add_entry_passkey(args: AddPasskeyArgs, state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&args.entry_uuid).map_err(|e| e.to_string())?;
    let credential_id = hex::decode(&args.credential_id).map_err(|e| e.to_string())?;
    let user_id = hex::decode(&args.user_id).map_err(|e| e.to_string())?;

    let passkey = keepassex_core::passkey::create_passkey_entry(
        credential_id,
        args.rp_id,
        args.rp_name,
        user_id,
        args.user_name,
        args.user_display_name,
        args.private_key_pem,
        args.backup_eligible,
    );

    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    entry.passkeys.push(passkey);
    entry.has_passkey = true;
    open_vault.vault.dirty = true;
    Ok(())
}

/// Remove a passkey from an entry by index
#[tauri::command]
pub fn remove_entry_passkey(
    entry_uuid: String,
    passkey_index: usize,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    if passkey_index >= entry.passkeys.len() {
        return Err(format!("Passkey index {} out of range", passkey_index));
    }

    entry.passkeys.remove(passkey_index);
    entry.has_passkey = !entry.passkeys.is_empty();
    open_vault.vault.dirty = true;
    Ok(())
}

/// Generate WebAuthn registration options for a new passkey
#[tauri::command]
pub fn get_passkey_registration_options(
    rp_id: String,
    rp_name: String,
    user_name: String,
    user_display_name: String,
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    use keepassex_core::passkey::{
        CredentialDescriptor, RegistrationOptions, RelyingParty, UserInfo,
    };

    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    // Exclude existing credentials for this RP
    let exclude: Vec<CredentialDescriptor> = entry
        .passkeys
        .iter()
        .filter(|pk| pk.rp_id == rp_id)
        .map(keepassex_core::passkey::CredentialDescriptor::from_passkey)
        .collect();

    let mut user_id = vec![0u8; 16];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut user_id);

    let opts = RegistrationOptions::new(
        RelyingParty {
            id: rp_id,
            name: rp_name,
        },
        UserInfo {
            id: user_id,
            name: user_name,
            display_name: user_display_name,
        },
        exclude,
    );

    serde_json::to_value(&opts).map_err(|e| e.to_string())
}
