//! Emergency Access Tauri commands
//!
//! KeePassEx exclusive: no competitor has emergency access built into KDBX vault.
//! Trusted contacts can request access after a configurable waiting period.

use crate::state::AppState;
use keepassex_core::emergency_access::{
    EmergencyAccess, EmergencyAccessLevel, EmergencyAccessManager,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct EmergencyGrantDto {
    pub id: String,
    pub grantee_name: String,
    pub grantee_email: String,
    pub access_level: String,
    pub wait_time_days: u32,
    pub status: String,
    pub days_remaining: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct AddEmergencyGrantArgs {
    pub name: String,
    pub email: String,
    pub access_level: String, // "view" | "takeover"
    pub wait_days: u32,
}

fn grant_to_dto(g: &EmergencyAccess) -> EmergencyGrantDto {
    EmergencyGrantDto {
        id: g.id.to_string(),
        grantee_name: g.grantee_name.clone(),
        grantee_email: g.grantee_email.clone(),
        access_level: match g.access_level {
            EmergencyAccessLevel::View => "view".to_string(),
            EmergencyAccessLevel::Takeover => "takeover".to_string(),
        },
        wait_time_days: g.wait_time_days,
        status: format!("{:?}", g.status).to_lowercase(),
        days_remaining: g.days_remaining(),
        created_at: g.created_at.to_rfc3339(),
    }
}

/// Get all emergency access grants
#[tauri::command]
pub fn get_emergency_grants(state: State<'_, AppState>) -> Result<Vec<EmergencyGrantDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    Ok(open_vault
        .vault
        .meta
        .emergency_access
        .grants
        .iter()
        .map(grant_to_dto)
        .collect())
}

/// Add a new emergency access grant
#[tauri::command]
pub fn add_emergency_grant(
    args: AddEmergencyGrantArgs,
    state: State<'_, AppState>,
) -> Result<EmergencyGrantDto, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let access_level = match args.access_level.as_str() {
        "takeover" => EmergencyAccessLevel::Takeover,
        _ => EmergencyAccessLevel::View,
    };

    // Generate a placeholder public key (production: grantee provides their X25519 public key)
    let mut public_key = vec![0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut public_key);

    let grant = EmergencyAccess::new(
        args.email.clone(),
        args.name,
        args.email,
        public_key,
        access_level,
        args.wait_days,
    );

    let dto = grant_to_dto(&grant);
    open_vault.vault.meta.emergency_access.add_grant(grant);
    open_vault.vault.dirty = true;

    tracing::info!("Emergency access grant added");
    Ok(dto)
}

/// Revoke an emergency access grant
#[tauri::command]
pub fn revoke_emergency_grant(grant_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let id = Uuid::parse_str(&grant_id).map_err(|e| e.to_string())?;

    let grant = open_vault
        .vault
        .meta
        .emergency_access
        .get_grant_mut(&id)
        .ok_or("Grant not found")?;

    grant.revoke();
    open_vault.vault.dirty = true;
    Ok(())
}

/// Approve an emergency access request (owner approves, starts wait period)
#[tauri::command]
pub fn approve_emergency_request(
    grant_id: String,
    state: State<'_, AppState>,
) -> Result<EmergencyGrantDto, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let id = Uuid::parse_str(&grant_id).map_err(|e| e.to_string())?;

    let grant = open_vault
        .vault
        .meta
        .emergency_access
        .get_grant_mut(&id)
        .ok_or("Grant not found")?;

    grant.approve_request().map_err(|e| e.to_string())?;
    open_vault.vault.dirty = true;

    let dto = grant_to_dto(grant);
    Ok(dto)
}

/// Delete an emergency access grant permanently
#[tauri::command]
pub fn delete_emergency_grant(grant_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let id = Uuid::parse_str(&grant_id).map_err(|e| e.to_string())?;
    open_vault.vault.meta.emergency_access.remove_grant(&id);
    open_vault.vault.dirty = true;
    Ok(())
}
