//! KeePassEx Plugin System — WASM sandbox
//!
//! Plugins are WebAssembly modules that can:
//! - Add custom entry fields/validators
//! - Add custom import/export formats
//! - Add custom password generators
//! - Add custom health checks
//!
//! Plugins run in a sandboxed WASM environment with no direct access
//! to the vault data — they receive only what they need via a typed API.

use crate::error::{KeePassExError, Result};
use serde::{Deserialize, Serialize};

// ─── Plugin Manifest ──────────────────────────────────────────────────────────

/// Plugin manifest (plugin.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub capabilities: Vec<PluginCapability>,
    pub permissions: Vec<PluginPermission>,
    pub entry_point: String, // WASM file name
    pub min_keepassex_version: String,
}

/// What a plugin can do
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    /// Custom password generator
    PasswordGenerator,
    /// Custom import format
    Importer,
    /// Custom export format
    Exporter,
    /// Custom health check rule
    HealthCheck,
    /// Custom entry field validator
    FieldValidator,
    /// Custom icon set
    IconSet,
    /// UI extension (adds menu items)
    UiExtension,
}

/// What data a plugin can access
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PluginPermission {
    /// Read entry titles and URLs (no passwords)
    ReadEntryMetadata,
    /// Read entry passwords (requires explicit user approval)
    ReadPasswords,
    /// Create/modify entries
    WriteEntries,
    /// Network access (for breach checks, etc.)
    Network,
    /// File system access (for import/export)
    FileSystem,
}

// ─── Plugin Registry ──────────────────────────────────────────────────────────

/// Installed plugin record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub install_path: String,
    pub installed_at: chrono::DateTime<chrono::Utc>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

/// Plugin registry — manages installed plugins
pub struct PluginRegistry {
    plugins: Vec<InstalledPlugin>,
    plugins_dir: std::path::PathBuf,
}

impl PluginRegistry {
    pub fn new(plugins_dir: std::path::PathBuf) -> Self {
        Self {
            plugins: Vec::new(),
            plugins_dir,
        }
    }

    /// Load all installed plugins from disk
    pub fn load_all(&mut self) -> Result<usize> {
        if !self.plugins_dir.exists() {
            return Ok(0);
        }

        let mut count = 0;
        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(KeePassExError::Io)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("plugin.json");
                if manifest_path.exists() {
                    if let Ok(manifest_json) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&manifest_json) {
                            self.plugins.push(InstalledPlugin {
                                manifest,
                                enabled: true,
                                install_path: path.to_string_lossy().to_string(),
                                installed_at: chrono::Utc::now(),
                                last_used: None,
                            });
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    pub fn list(&self) -> &[InstalledPlugin] {
        &self.plugins
    }

    pub fn get(&self, id: &str) -> Option<&InstalledPlugin> {
        self.plugins.iter().find(|p| p.manifest.id == id)
    }

    pub fn enable(&mut self, id: &str) -> Result<()> {
        self.plugins.iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| KeePassExError::Other(format!("Plugin not found: {}", id)))?
            .enabled = true;
        Ok(())
    }

    pub fn disable(&mut self, id: &str) -> Result<()> {
        self.plugins.iter_mut()
            .find(|p| p.manifest.id == id)
            .ok_or_else(|| KeePassExError::Other(format!("Plugin not found: {}", id)))?
            .enabled = false;
        Ok(())
    }

    pub fn uninstall(&mut self, id: &str) -> Result<()> {
        let idx = self.plugins.iter().position(|p| p.manifest.id == id)
            .ok_or_else(|| KeePassExError::Other(format!("Plugin not found: {}", id)))?;

        let plugin = self.plugins.remove(idx);
        std::fs::remove_dir_all(&plugin.install_path)
            .map_err(KeePassExError::Io)?;

        Ok(())
    }
}

// ─── Plugin API (exposed to WASM) ─────────────────────────────────────────────

/// The API surface exposed to plugins via WASM imports
/// Plugins call these functions to interact with KeePassEx
pub trait PluginApi {
    /// Log a message (debug only)
    fn log(&self, level: LogLevel, message: &str);

    /// Generate a random password using core generator
    fn generate_password(&self, length: usize) -> String;

    /// Get the current vault name (no sensitive data)
    fn get_vault_name(&self) -> String;

    /// Show a notification to the user
    fn notify(&self, title: &str, message: &str);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

// ─── Plugin Host ──────────────────────────────────────────────────────────────

/// Hosts a WASM plugin in a sandboxed environment
/// In production: uses wasmtime or wasmer for execution
pub struct PluginHost {
    manifest: PluginManifest,
    // wasm_instance: wasmtime::Instance,  // Production
}

impl PluginHost {
    /// Load and instantiate a plugin from its WASM binary
    pub fn load(manifest: PluginManifest, _wasm_bytes: &[u8]) -> Result<Self> {
        // Production implementation:
        // 1. Validate WASM binary signature
        // 2. Create wasmtime Engine with resource limits
        // 3. Define host functions (PluginApi)
        // 4. Instantiate module with sandbox
        // 5. Call plugin's init() function

        tracing::info!("Loading plugin: {} v{}", manifest.name, manifest.version);

        Ok(Self { manifest })
    }

    /// Call a plugin function
    pub fn call(&self, function: &str, args: &[u8]) -> Result<Vec<u8>> {
        // Production: call WASM function via wasmtime
        tracing::debug!("Plugin {} calling: {}", self.manifest.id, function);
        Ok(Vec::new())
    }

    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

// ─── Built-in Plugin Examples ─────────────────────────────────────────────────

/// Example: Dashlane importer plugin manifest
pub fn dashlane_importer_manifest() -> PluginManifest {
    PluginManifest {
        id: "com.keepassex.importer.dashlane".to_string(),
        name: "Dashlane Importer".to_string(),
        version: "1.0.0".to_string(),
        description: "Import passwords from Dashlane CSV export".to_string(),
        author: "KeePassEx Team".to_string(),
        license: "GPL-3.0".to_string(),
        capabilities: vec![PluginCapability::Importer],
        permissions: vec![PluginPermission::FileSystem],
        entry_point: "dashlane_importer.wasm".to_string(),
        min_keepassex_version: "1.0.0".to_string(),
    }
}

/// Example: Enpass importer plugin manifest
pub fn enpass_importer_manifest() -> PluginManifest {
    PluginManifest {
        id: "com.keepassex.importer.enpass".to_string(),
        name: "Enpass Importer".to_string(),
        version: "1.0.0".to_string(),
        description: "Import passwords from Enpass JSON export".to_string(),
        author: "KeePassEx Team".to_string(),
        license: "GPL-3.0".to_string(),
        capabilities: vec![PluginCapability::Importer],
        permissions: vec![PluginPermission::FileSystem],
        entry_point: "enpass_importer.wasm".to_string(),
        min_keepassex_version: "1.0.0".to_string(),
    }
}
