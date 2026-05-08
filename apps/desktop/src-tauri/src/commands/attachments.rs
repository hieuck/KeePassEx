//! Attachment Tauri commands

use crate::state::AppState;
use tauri::State;
use uuid::Uuid;

/// Read a file as bytes (for adding attachments)
#[tauri::command]
pub async fn read_file_bytes(path: String) -> Result<Vec<u8>, String> {
    tokio::fs::read(&path)
        .await
        .map_err(|e| format!("Cannot read file: {}", e))
}

/// Save an attachment from an entry to disk
#[tauri::command]
pub async fn save_attachment(
    entry_uuid: String,
    attachment_name: String,
    output_path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Collect data BEFORE any await — avoids Send issues with RwLock
    let data = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
        if open_vault.locked {
            return Err("Vault is locked".into());
        }
        let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
        let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;
        let key = format!("_attachment_{}", attachment_name);
        let data_b64 = entry
            .custom_fields
            .get(&key)
            .map(|f| f.value.get().to_string())
            .ok_or_else(|| format!("Attachment '{}' not found", attachment_name))?;
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data_b64)
            .map_err(|e| format!("Cannot decode attachment: {}", e))?
    };
    // State lock released — safe to await

    tokio::fs::write(&output_path, &data)
        .await
        .map_err(|e| format!("Cannot write file: {}", e))
}

/// Add an attachment to an entry (sync — no I/O needed)
#[tauri::command]
pub fn add_attachment(
    entry_uuid: String,
    attachment_name: String,
    data: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;
    if open_vault.locked {
        return Err("Vault is locked".into());
    }
    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let data_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
    if let Some(entry) = open_vault.vault.get_entry_mut(&uuid) {
        let key = format!("_attachment_{}", attachment_name);
        entry.custom_fields.insert(
            key.clone(),
            keepassex_core::types::CustomField {
                key,
                value: keepassex_core::types::ProtectedString::new(data_b64),
                protected: false,
            },
        );
        Ok(())
    } else {
        Err("Entry not found".into())
    }
}

/// Remove an attachment from an entry
#[tauri::command]
pub fn remove_attachment(
    entry_uuid: String,
    attachment_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;
    let uuid = Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    if let Some(entry) = open_vault.vault.get_entry_mut(&uuid) {
        entry
            .custom_fields
            .remove(&format!("_attachment_{}", attachment_name));
        Ok(())
    } else {
        Err("Entry not found".into())
    }
}
