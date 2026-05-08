//! SSH Agent Tauri commands

use crate::state::AppState;
use serde::Serialize;
use tauri::{Emitter, State};

#[derive(Debug, Serialize)]
pub struct SshKeyDto {
    pub fingerprint: String,
    pub comment: String,
    pub key_type: String,
    pub loaded_at: String,
}

#[tauri::command]
pub async fn start_ssh_agent(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    // Check if settings allow SSH agent
    if !state.settings.read().unwrap().ssh_agent_enabled {
        return Err("SSH Agent is disabled in settings".into());
    }

    let path = crate::ssh_agent_server::socket_path();

    // Notify frontend
    let _ = app.emit("ssh-agent-started", serde_json::json!({ "path": &path }));

    Ok(format!("SSH_AUTH_SOCK={}", path))
}

#[tauri::command]
pub fn stop_ssh_agent(state: State<'_, AppState>) -> Result<(), String> {
    state.ssh_agent.write().unwrap().remove_all();
    Ok(())
}

#[tauri::command]
pub fn add_ssh_key(entry_uuid: String, state: State<'_, AppState>) -> Result<(), String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let uuid = uuid::Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;
    let ssh_key = entry.ssh_key.as_ref().ok_or("Entry has no SSH key")?;

    state.ssh_agent.write().unwrap().add_key(ssh_key.clone());
    Ok(())
}

#[tauri::command]
pub fn list_ssh_keys(state: State<'_, AppState>) -> Vec<SshKeyDto> {
    state
        .ssh_agent
        .read()
        .unwrap()
        .list_keys()
        .into_iter()
        .map(|k| SshKeyDto {
            fingerprint: k.fingerprint.clone(),
            comment: k.comment.clone(),
            key_type: format!("{:?}", k.key_type),
            loaded_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect()
}
