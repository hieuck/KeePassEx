//! Plugin management Tauri commands
//!
//! KeePassEx exclusive: WASM plugin sandbox — no competitor has this.
//! Plugins extend KeePassEx without modifying core code.

use crate::state::AppState;
use keepassex_core::plugin::{InstalledPlugin, PluginCapability, PluginManifest, PluginRegistry};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Clone)]
pub struct PluginDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<String>,
    pub enabled: bool,
    pub installed_at: String,
}

fn plugin_to_dto(p: &InstalledPlugin) -> PluginDto {
    PluginDto {
        id: p.manifest.id.clone(),
        name: p.manifest.name.clone(),
        version: p.manifest.version.clone(),
        description: p.manifest.description.clone(),
        author: p.manifest.author.clone(),
        capabilities: p
            .manifest
            .capabilities
            .iter()
            .map(|c| format!("{:?}", c).to_lowercase())
            .collect(),
        enabled: p.enabled,
        installed_at: p.installed_at.to_rfc3339(),
    }
}

fn get_plugins_dir(app: &tauri::AppHandle) -> std::path::PathBuf {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("plugins")
}

/// List all installed plugins
#[tauri::command]
pub fn list_plugins(app: tauri::AppHandle) -> Result<Vec<PluginDto>, String> {
    let plugins_dir = get_plugins_dir(&app);
    let mut registry = PluginRegistry::new(plugins_dir);
    registry.load_all().map_err(|e| e.to_string())?;
    Ok(registry.list().iter().map(plugin_to_dto).collect())
}

/// Enable or disable a plugin
#[tauri::command]
pub fn toggle_plugin(
    plugin_id: String,
    enabled: bool,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let plugins_dir = get_plugins_dir(&app);
    let mut registry = PluginRegistry::new(plugins_dir);
    registry.load_all().map_err(|e| e.to_string())?;

    if enabled {
        registry.enable(&plugin_id).map_err(|e| e.to_string())?;
    } else {
        registry.disable(&plugin_id).map_err(|e| e.to_string())?;
    }

    // Persist enabled state to a config file
    let config_path = get_plugins_dir(&app).join("plugins.json");
    let enabled_ids: Vec<String> = registry
        .list()
        .iter()
        .filter(|p| p.enabled)
        .map(|p| p.manifest.id.clone())
        .collect();

    std::fs::create_dir_all(get_plugins_dir(&app)).ok();
    std::fs::write(
        &config_path,
        serde_json::to_string_pretty(&enabled_ids).unwrap_or_default(),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Uninstall a plugin
#[tauri::command]
pub fn uninstall_plugin(plugin_id: String, app: tauri::AppHandle) -> Result<(), String> {
    let plugins_dir = get_plugins_dir(&app);
    let mut registry = PluginRegistry::new(plugins_dir);
    registry.load_all().map_err(|e| e.to_string())?;
    registry.uninstall(&plugin_id).map_err(|e| e.to_string())?;
    tracing::info!("Plugin uninstalled: {}", plugin_id);
    Ok(())
}

/// Install a plugin from a file path (.kpxplugin or .wasm)
#[tauri::command]
pub async fn install_plugin_from_file(
    file_path: String,
    app: tauri::AppHandle,
) -> Result<PluginDto, String> {
    let plugins_dir = get_plugins_dir(&app);
    std::fs::create_dir_all(&plugins_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&file_path);
    let file_name = source
        .file_name()
        .ok_or("Invalid file path")?
        .to_string_lossy()
        .to_string();

    // Extract plugin ID from filename (e.g., "com.example.plugin.kpxplugin")
    let plugin_id = file_name
        .trim_end_matches(".kpxplugin")
        .trim_end_matches(".wasm")
        .trim_end_matches(".zip")
        .to_string();

    // Create plugin directory
    let plugin_dir = plugins_dir.join(&plugin_id);
    std::fs::create_dir_all(&plugin_dir).map_err(|e| e.to_string())?;

    // Copy plugin file
    let dest = plugin_dir.join(&file_name);
    tokio::fs::copy(&file_path, &dest)
        .await
        .map_err(|e| e.to_string())?;

    // Create a basic manifest if none exists
    let manifest_path = plugin_dir.join("plugin.json");
    if !manifest_path.exists() {
        let manifest = PluginManifest {
            id: plugin_id.clone(),
            name: plugin_id.clone(),
            version: "1.0.0".to_string(),
            description: format!("Plugin: {}", plugin_id),
            author: "Unknown".to_string(),
            license: "Unknown".to_string(),
            capabilities: vec![],
            permissions: vec![],
            entry_point: file_name.clone(),
            min_keepassex_version: "0.1.0".to_string(),
        };
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap_or_default(),
        )
        .map_err(|e| e.to_string())?;
    }

    tracing::info!("Plugin installed: {}", plugin_id);

    Ok(PluginDto {
        id: plugin_id.clone(),
        name: plugin_id,
        version: "1.0.0".to_string(),
        description: format!("Installed from {}", file_name),
        author: "Unknown".to_string(),
        capabilities: vec![],
        enabled: true,
        installed_at: chrono::Utc::now().to_rfc3339(),
    })
}
