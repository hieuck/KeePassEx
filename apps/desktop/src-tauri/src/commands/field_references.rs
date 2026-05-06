//! Tauri commands for entry field references

use crate::state::AppState;
use keepassex_core::field_references::{
    build_ref, has_references, resolve_entry_references, resolve_references, ResolvedEntry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ResolvedEntryDto {
    pub uuid: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,
}

impl From<ResolvedEntry> for ResolvedEntryDto {
    fn from(r: ResolvedEntry) -> Self {
        Self {
            uuid: r.uuid.to_string(),
            title: r.title,
            username: r.username,
            password: r.password,
            url: r.url,
            notes: r.notes,
        }
    }
}

/// Resolve all field references in a single entry.
/// Returns the entry with all {REF:...} placeholders replaced by their values.
#[tauri::command]
pub fn resolve_entry_refs(
    entry_uuid: String,
    state: State<'_, AppState>,
) -> Result<ResolvedEntryDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault
        .vault
        .get_entry(&uuid)
        .ok_or_else(|| format!("Entry not found: {}", entry_uuid))?;

    // Build entry map for reference resolution
    let entry_map: HashMap<Uuid, &keepassex_core::Entry> = open_vault
        .vault
        .all_entries()
        .map(|e| (e.uuid, e))
        .collect();

    let resolved = resolve_entry_references(entry, &entry_map);
    Ok(ResolvedEntryDto::from(resolved))
}

/// Resolve a single field reference string against the vault.
/// e.g. "{REF:P@I:550e8400-...}" → "actual_password"
#[tauri::command]
pub fn resolve_ref_string(value: String, state: State<'_, AppState>) -> Result<String, String> {
    if !has_references(&value) {
        return Ok(value);
    }

    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let entry_map: HashMap<Uuid, &keepassex_core::Entry> = open_vault
        .vault
        .all_entries()
        .map(|e| (e.uuid, e))
        .collect();

    resolve_references(&value, &entry_map, 0).map_err(|e| e.to_string())
}

/// Build a field reference token for insertion into an entry field.
/// field: 'T' | 'U' | 'P' | 'A' | 'N'
/// target_uuid: UUID of the entry to reference
#[tauri::command]
pub fn build_field_ref(field: char, target_uuid: String) -> Result<String, String> {
    let uuid = Uuid::parse_str(&target_uuid).map_err(|e| e.to_string())?;
    Ok(build_ref(field, &uuid))
}

/// Check if a string contains any field references.
#[tauri::command]
pub fn check_has_refs(value: String) -> bool {
    has_references(&value)
}
