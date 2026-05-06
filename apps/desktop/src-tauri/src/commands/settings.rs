//! Settings Tauri commands — with disk persistence

use crate::state::{AppSettings, AppState};
use tauri::State;

/// Settings file path relative to app data dir
const SETTINGS_FILE: &str = "keepassex-settings.json";

/// Get current settings
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> AppSettings {
    state.settings.read().unwrap().clone()
}

/// Save settings to memory and persist to disk
#[tauri::command]
pub async fn save_settings(
    settings: AppSettings,
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Update in-memory state
    *state.settings.write().unwrap() = settings.clone();

    // Persist to disk in app data directory
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    tokio::fs::create_dir_all(&app_dir)
        .await
        .map_err(|e| e.to_string())?;

    let settings_path = app_dir.join(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(&settings)
        .map_err(|e| e.to_string())?;

    tokio::fs::write(&settings_path, json)
        .await
        .map_err(|e| e.to_string())?;

    tracing::debug!("Settings saved to {:?}", settings_path);
    Ok(())
}

/// Load settings from disk (called on startup)
pub async fn load_settings_from_disk(app: &tauri::AppHandle) -> Option<AppSettings> {
    let app_dir = app.path().app_data_dir().ok()?;
    let settings_path = app_dir.join(SETTINGS_FILE);

    let json = tokio::fs::read_to_string(&settings_path).await.ok()?;
    serde_json::from_str(&json).ok()
}
