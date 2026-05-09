//! Clipboard commands with auto-clear

use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn copy_to_clipboard(
    text: String,
    clear_after_seconds: Option<u32>,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    app.clipboard()
        .write_text(text)
        .map_err(|e| e.to_string())?;

    // Schedule auto-clear
    let delay = clear_after_seconds
        .or_else(|| state.settings.read().unwrap().clipboard_clear_seconds)
        .unwrap_or(10);

    if delay > 0 {
        let app_clone = app.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay as u64)).await;
            let _ = app_clone.clipboard().write_text("".to_string());
        });
    }

    Ok(())
}

#[tauri::command]
pub fn clear_clipboard(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard()
        .write_text("".to_string())
        .map_err(|e| e.to_string())
}

/// Read text from clipboard (for QuickEntryCreator URL detection)
#[tauri::command]
pub fn read_clipboard_text(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().read_text().map_err(|e| e.to_string())
}
