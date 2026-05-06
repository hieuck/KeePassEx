//! Entry Tauri commands

use crate::state::AppState;
use keepassex_core::types::*;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct EntryDto {
    pub uuid: String,
    pub group_uuid: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub notes: String,
    pub icon_id: u32,
    pub tags: Vec<String>,
    pub has_password: bool,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub has_ssh_key: bool,
    pub has_attachments: bool,
    pub is_expired: bool,
    pub expiry: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub custom_fields: Vec<CustomFieldDto>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CustomFieldDto {
    pub key: String,
    pub value: String,
    pub protected: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateEntryArgs {
    pub group_uuid: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub icon_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEntryArgs {
    pub uuid: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
    pub tags: Vec<String>,
    pub icon_id: u32,
    pub expiry: Option<String>,
    pub custom_fields: Vec<CustomFieldDto>,
}

/// Get all entries in a group
#[tauri::command]
pub fn get_entries(
    group_uuid: Option<String>,
    state: State<'_, AppState>,
) -> Result<Vec<EntryDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let entries: Vec<EntryDto> = if let Some(uuid_str) = group_uuid {
        let uuid = Uuid::parse_str(&uuid_str).map_err(|e| e.to_string())?;
        open_vault
            .vault
            .get_group_entries_recursive(&uuid)
            .into_iter()
            .map(entry_to_dto)
            .collect()
    } else {
        open_vault.vault.all_entries().map(entry_to_dto).collect()
    };

    Ok(entries)
}

/// Get a single entry by UUID
#[tauri::command]
pub fn get_entry(
    uuid: String,
    include_password: bool,
    state: State<'_, AppState>,
) -> Result<EntryDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    let mut dto = entry_to_dto(entry);
    // Only include password if explicitly requested (security)
    // In production: require additional auth for password reveal

    Ok(dto)
}

/// Get entry password (requires explicit request)
#[tauri::command]
pub fn get_entry_password(uuid: String, state: State<'_, AppState>) -> Result<String, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    Ok(entry.password.get().to_string())
}

/// Create a new entry
#[tauri::command]
pub fn create_entry(args: CreateEntryArgs, state: State<'_, AppState>) -> Result<String, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let group_uuid = Uuid::parse_str(&args.group_uuid).map_err(|e| e.to_string())?;
    let entry_uuid = open_vault
        .vault
        .create_entry(group_uuid)
        .map_err(|e| e.to_string())?;

    // Update entry fields
    if let Some(entry) = open_vault.vault.get_entry_mut(&entry_uuid) {
        entry.title.set(args.title);
        entry.username.set(args.username);
        entry.password.set(args.password);
        entry.url = args.url;
        entry.notes.set(args.notes);
        entry.tags = args.tags;
        entry.icon_id = args.icon_id;
    }

    Ok(entry_uuid.to_string())
}

/// Update an existing entry
#[tauri::command]
pub fn update_entry(args: UpdateEntryArgs, state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&args.uuid).map_err(|e| e.to_string())?;

    let mut entry = open_vault
        .vault
        .get_entry(&uuid)
        .ok_or("Entry not found")?
        .clone();

    entry.title.set(args.title);
    entry.username.set(args.username);
    entry.password.set(args.password);
    entry.url = args.url;
    entry.notes.set(args.notes);
    entry.tags = args.tags;
    entry.icon_id = args.icon_id;
    entry.modified_at = chrono::Utc::now();

    if let Some(expiry_str) = args.expiry {
        entry.expiry = chrono::DateTime::parse_from_rfc3339(&expiry_str)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));
    } else {
        entry.expiry = None;
    }

    open_vault
        .vault
        .update_entry(entry)
        .map_err(|e| e.to_string())
}

/// Delete an entry
#[tauri::command]
pub fn delete_entry(
    uuid: String,
    permanent: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    open_vault
        .vault
        .delete_entry(&uuid, permanent)
        .map_err(|e| e.to_string())
}

/// Move entry to another group
#[tauri::command]
pub fn move_entry(
    uuid: String,
    new_group_uuid: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let group_uuid = Uuid::parse_str(&new_group_uuid).map_err(|e| e.to_string())?;

    open_vault
        .vault
        .move_entry(&uuid, group_uuid)
        .map_err(|e| e.to_string())
}

/// Duplicate an entry
#[tauri::command]
pub fn duplicate_entry(uuid: String, state: State<'_, AppState>) -> Result<String, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    open_vault
        .vault
        .duplicate_entry(&uuid)
        .map(|u| u.to_string())
        .map_err(|e| e.to_string())
}

/// Search entries
#[tauri::command]
pub fn search_entries(query: String, state: State<'_, AppState>) -> Result<Vec<EntryDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let search_query = SearchQuery::new(query);
    let results = open_vault.vault.search(&search_query);

    Ok(results.into_iter().map(entry_to_dto).collect())
}

// ─── Entry History Commands ───────────────────────────────────────────────────

/// DTO for a single history snapshot
#[derive(Debug, Serialize, Clone)]
pub struct EntryHistoryDto {
    /// Synthetic UUID derived from the snapshot's modified_at timestamp
    pub uuid: String,
    pub modified_at: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub notes: String,
    pub has_password: bool,
}

/// Get the history snapshots for an entry (newest first)
#[tauri::command]
pub fn get_entry_history(
    uuid: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntryHistoryDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    let mut history: Vec<EntryHistoryDto> = entry
        .history
        .iter()
        .enumerate()
        .map(|(i, h)| EntryHistoryDto {
            // Use index + timestamp as a stable synthetic ID
            uuid: format!("{}-history-{}", uuid, i),
            modified_at: h.modified_at.to_rfc3339(),
            title: h.entry_snapshot.title.get().to_string(),
            username: h.entry_snapshot.username.get().to_string(),
            url: h.entry_snapshot.url.clone(),
            notes: h.entry_snapshot.notes.get().to_string(),
            has_password: !h.entry_snapshot.password.get().is_empty(),
        })
        .collect();

    // Newest first
    history.reverse();
    Ok(history)
}

/// Restore an entry to a previous history snapshot.
/// The current state is pushed into history before restoring.
#[tauri::command]
pub fn restore_entry_from_history(
    entry_uuid: String,
    history_uuid: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;

    // Parse the synthetic history index from the UUID suffix (format: "<uuid>-history-<index>")
    let index: usize = history_uuid
        .rsplit('-')
        .next()
        .and_then(|s| s.parse().ok())
        .ok_or("Invalid history UUID format")?;

    let entry = open_vault
        .vault
        .get_entry(&uuid)
        .ok_or("Entry not found")?
        .clone();

    // Bounds check (history is stored oldest-first)
    if index >= entry.history.len() {
        return Err(format!(
            "History index {} out of range (len={})",
            index,
            entry.history.len()
        ));
    }

    // Clone the snapshot we want to restore
    let snapshot = *entry.history[index].entry_snapshot.clone();

    // Build a new entry from the snapshot, preserving the current UUID and group
    let mut restored = snapshot;
    restored.uuid = uuid;
    restored.group_uuid = entry.group_uuid;
    restored.modified_at = chrono::Utc::now();

    open_vault
        .vault
        .update_entry(restored)
        .map_err(|e| e.to_string())
}

/// Clear all history snapshots for an entry
#[tauri::command]
pub fn clear_entry_history(uuid: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    entry.history.clear();
    open_vault.vault.dirty = true;
    Ok(())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn entry_to_dto(entry: &keepassex_core::types::Entry) -> EntryDto {
    EntryDto {
        uuid: entry.uuid.to_string(),
        group_uuid: entry.group_uuid.to_string(),
        title: entry.title.get().to_string(),
        username: entry.username.get().to_string(),
        url: entry.url.clone(),
        notes: entry.notes.get().to_string(),
        icon_id: entry.icon_id,
        tags: entry.tags.clone(),
        has_password: !entry.password.get().is_empty(),
        has_otp: entry.otp.is_some(),
        has_passkey: !entry.passkeys.is_empty(),
        has_ssh_key: entry.ssh_key.is_some(),
        has_attachments: !entry.attachments.is_empty(),
        is_expired: entry.is_expired(),
        expiry: entry.expiry.map(|e| e.to_rfc3339()),
        created_at: entry.created_at.to_rfc3339(),
        modified_at: entry.modified_at.to_rfc3339(),
        custom_fields: entry
            .custom_fields
            .values()
            .map(|f| CustomFieldDto {
                key: f.key.clone(),
                value: f.value.get().to_string(),
                protected: f.value.protected,
            })
            .collect(),
    }
}
