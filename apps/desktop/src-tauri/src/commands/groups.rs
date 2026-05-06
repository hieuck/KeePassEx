//! Group Tauri commands

use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct GroupDto {
    pub uuid: String,
    pub parent_uuid: Option<String>,
    pub name: String,
    pub notes: String,
    pub icon_id: u32,
    pub is_expanded: bool,
    pub entry_count: usize,
    pub child_group_count: usize,
}

#[tauri::command]
pub fn get_groups(state: State<'_, AppState>) -> Result<Vec<GroupDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let groups: Vec<GroupDto> = open_vault
        .vault
        .all_groups()
        .map(|g| GroupDto {
            uuid: g.uuid.to_string(),
            parent_uuid: g.parent_uuid.map(|u| u.to_string()),
            name: g.name.clone(),
            notes: g.notes.clone(),
            icon_id: g.icon_id,
            is_expanded: g.is_expanded,
            entry_count: open_vault.vault.get_group_entries(&g.uuid).len(),
            child_group_count: open_vault.vault.get_child_groups(&g.uuid).len(),
        })
        .collect();

    Ok(groups)
}

#[tauri::command]
pub fn create_group(
    name: String,
    parent_uuid: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let parent = Uuid::parse_str(&parent_uuid).map_err(|e| e.to_string())?;
    open_vault
        .vault
        .create_group(name, parent)
        .map(|u| u.to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_group(
    uuid: String,
    name: String,
    notes: String,
    icon_id: u32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    if let Some(group) = open_vault.vault.get_group_mut(&uuid) {
        group.name = name;
        group.notes = notes;
        group.icon_id = icon_id;
        group.modified_at = chrono::Utc::now();
        Ok(())
    } else {
        Err("Group not found".into())
    }
}

#[tauri::command]
pub fn delete_group(
    uuid: String,
    permanent: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    open_vault
        .vault
        .delete_group(&uuid, permanent)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn move_group(
    uuid: String,
    new_parent_uuid: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    let uuid = Uuid::parse_str(&uuid).map_err(|e| e.to_string())?;
    let parent = Uuid::parse_str(&new_parent_uuid).map_err(|e| e.to_string())?;
    open_vault
        .vault
        .move_group(&uuid, parent)
        .map_err(|e| e.to_string())
}
