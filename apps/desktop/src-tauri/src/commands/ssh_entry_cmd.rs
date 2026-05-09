//! SSH key entry Tauri commands
//!
//! Manage SSH keys stored inside KDBX entries.
//! KeePassEx provides better SSH key management than KeePassXC:
//! - Per-entry SSH key with agent duration control
//! - Confirm-before-use protection
//! - Fingerprint display
//! - One-click copy public key

use crate::state::AppState;
use keepassex_core::types::{SshKeyEntry, SshKeyType};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct SshKeyDto {
    pub key_type: String,
    pub public_key: String,
    pub comment: String,
    pub fingerprint: String,
    pub add_to_agent: bool,
    pub agent_duration: Option<u64>,
    pub confirm_before_use: bool,
}

#[derive(Debug, Deserialize)]
pub struct SetSshKeyArgs {
    pub entry_uuid: String,
    pub private_key: String,
    pub public_key: String,
    pub comment: String,
    pub key_type: String,
    pub add_to_agent: bool,
    pub agent_duration: Option<u64>,
    pub confirm_before_use: bool,
}

/// Get SSH key for an entry
#[tauri::command]
pub fn get_entry_ssh_key(
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<Option<SshKeyDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    Ok(entry.ssh_key.as_ref().map(|k| SshKeyDto {
        key_type: ssh_key_type_to_str(&k.key_type).to_string(),
        public_key: k.public_key.clone(),
        comment: k.comment.clone(),
        fingerprint: k.fingerprint.clone(),
        add_to_agent: k.add_to_agent,
        agent_duration: k.agent_duration,
        confirm_before_use: k.confirm_before_use,
    }))
}

/// Set SSH key for an entry (replaces existing)
#[tauri::command]
pub fn set_entry_ssh_key(
    args: SetSshKeyArgs,
    state: State<'_, AppState>,
) -> Result<SshKeyDto, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&args.entry_uuid).map_err(|e| e.to_string())?;

    // Parse public key to get fingerprint
    let key_info =
        keepassex_core::ssh::parse_ssh_public_key(&args.public_key).map_err(|e| e.to_string())?;

    let key_type = parse_key_type(&args.key_type)?;

    let ssh_key = SshKeyEntry {
        key_type,
        private_key: keepassex_core::types::ProtectedString::new(args.private_key),
        public_key: args.public_key,
        comment: args.comment,
        fingerprint: key_info.fingerprint.clone(),
        add_to_agent: args.add_to_agent,
        agent_duration: args.agent_duration,
        confirm_before_use: args.confirm_before_use,
    };

    let dto = SshKeyDto {
        key_type: ssh_key_type_to_str(&ssh_key.key_type).to_string(),
        public_key: ssh_key.public_key.clone(),
        comment: ssh_key.comment.clone(),
        fingerprint: ssh_key.fingerprint.clone(),
        add_to_agent: ssh_key.add_to_agent,
        agent_duration: ssh_key.agent_duration,
        confirm_before_use: ssh_key.confirm_before_use,
    };

    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    entry.ssh_key = Some(ssh_key);
    entry.has_ssh_key = true;
    open_vault.vault.dirty = true;

    Ok(dto)
}

/// Remove SSH key from an entry
#[tauri::command]
pub fn remove_entry_ssh_key(entry_uuid: String, state: State<'_, AppState>) -> Result<(), String> {
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

    entry.ssh_key = None;
    entry.has_ssh_key = false;
    open_vault.vault.dirty = true;
    Ok(())
}

/// Get SSH private key for an entry (requires explicit request)
#[tauri::command]
pub fn get_entry_ssh_private_key(
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    entry
        .ssh_key
        .as_ref()
        .map(|k| k.private_key.get().to_string())
        .ok_or_else(|| "No SSH key configured".into())
}

/// Load SSH key into the agent
#[tauri::command]
pub async fn load_ssh_key_to_agent(
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    let ssh_key = entry.ssh_key.as_ref().ok_or("No SSH key configured")?;

    // Add to SSH agent state
    let mut agent = state.ssh_agent.write().unwrap();
    agent.add_key(ssh_key.clone());

    tracing::info!("SSH key loaded to agent: {}", ssh_key.fingerprint);
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn ssh_key_type_to_str(t: &SshKeyType) -> &'static str {
    match t {
        SshKeyType::Ed25519 => "ed25519",
        SshKeyType::Rsa2048 => "rsa2048",
        SshKeyType::Rsa4096 => "rsa4096",
        SshKeyType::EcdsaP256 => "ecdsa-p256",
        SshKeyType::EcdsaP384 => "ecdsa-p384",
    }
}

fn parse_key_type(s: &str) -> Result<SshKeyType, String> {
    match s.to_lowercase().as_str() {
        "ed25519" | "ssh-ed25519" => Ok(SshKeyType::Ed25519),
        "rsa2048" => Ok(SshKeyType::Rsa2048),
        "rsa4096" | "rsa" | "ssh-rsa" => Ok(SshKeyType::Rsa4096),
        "ecdsa-p256" | "ecdsa-sha2-nistp256" => Ok(SshKeyType::EcdsaP256),
        "ecdsa-p384" | "ecdsa-sha2-nistp384" => Ok(SshKeyType::EcdsaP384),
        _ => Err(format!("Unknown SSH key type: {}", s)),
    }
}
