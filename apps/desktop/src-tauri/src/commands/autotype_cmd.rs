//! Auto-Type Tauri command
//!
//! Waits briefly for the user to switch focus, then types credentials
//! into the active window using the platform keyboard simulation API.

use crate::autotype::{AutoTypeContext, AutoTypeEngine};
use crate::state::AppState;
use tauri::State;

/// Auto-type credentials for an entry into the currently focused window.
///
/// Default sequence: `{USERNAME}{TAB}{PASSWORD}{ENTER}`
/// Custom sequence can be stored in entry custom field `Auto-Type-Sequence`.
#[tauri::command]
pub async fn auto_type_entry(entry_uuid: String, state: State<'_, AppState>) -> Result<(), String> {
    // Collect entry data under lock, then release before sleeping
    let (title, username, password, url, sequence) = {
        let vault_lock = state.vault.read().unwrap();
        let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

        if open_vault.locked {
            return Err("Vault is locked".into());
        }

        let uuid = uuid::Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
        let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

        // Check for custom auto-type sequence in custom fields
        let sequence = entry
            .custom_fields
            .get("Auto-Type-Sequence")
            .map(|f| f.value.get().to_string())
            .unwrap_or_else(|| "{USERNAME}{TAB}{PASSWORD}{ENTER}".to_string());

        (
            entry.title.get().to_string(),
            entry.username.get().to_string(),
            entry.password.get().to_string(),
            entry.url.clone(),
            sequence,
        )
    };

    // Give user 500ms to switch to the target window
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let context = AutoTypeContext {
        title,
        username,
        password,
        url,
        totp: None,
        custom_fields: std::collections::HashMap::new(),
    };

    AutoTypeEngine::execute(&sequence, &context).map_err(|e| e.to_string())
}
