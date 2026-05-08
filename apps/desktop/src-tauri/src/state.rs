//! Application state management

use keepassex_core::Vault;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Global application state
pub struct AppState {
    pub vault: Arc<RwLock<Option<OpenVault>>>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub ssh_agent: Arc<RwLock<keepassex_core::ssh::SshAgent>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            vault: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            ssh_agent: Arc::new(RwLock::new(keepassex_core::ssh::SshAgent::new())),
        }
    }
}

/// An open vault with its file path and credentials context
pub struct OpenVault {
    pub vault: Vault,
    pub path: std::path::PathBuf,
    /// Encrypted master key for re-save (never stored in plaintext long-term)
    pub master_key_hash: Vec<u8>,
    pub locked: bool,
}

/// Application settings (persisted to disk)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub theme: String,
    pub lock_on_minimize: bool,
    pub lock_on_screen_lock: bool,
    pub lock_after_idle_minutes: Option<u32>,
    pub clipboard_clear_seconds: Option<u32>,
    pub show_password_in_list: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub check_for_updates: bool,
    pub browser_integration: bool,
    pub ssh_agent_enabled: bool,
    pub default_auto_type_sequence: String,
    pub recent_vaults: Vec<RecentVault>,
    /// Scheduled backup configuration
    pub backup_config: Option<keepassex_core::scheduled_backup::BackupConfig>,
    /// IDs of enabled password policies
    pub enabled_policy_ids: Option<Vec<String>>,
    /// Password policies (serialized)
    pub password_policies: Option<String>,
    /// Sync configuration (provider, credentials, etc.)
    pub sync_config: Option<keepassex_core::sync::SyncConfig>,
    /// Last sync timestamp (RFC3339)
    pub last_sync_at: Option<String>,
    /// Last sync status (success, error, no_changes, conflict)
    pub last_sync_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentVault {
    pub path: String,
    pub name: String,
    pub last_opened: chrono::DateTime<chrono::Utc>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            lock_on_minimize: false,
            lock_on_screen_lock: true,
            lock_after_idle_minutes: Some(5),
            clipboard_clear_seconds: Some(10),
            show_password_in_list: false,
            minimize_to_tray: true,
            start_minimized: false,
            check_for_updates: true,
            browser_integration: true,
            ssh_agent_enabled: false,
            default_auto_type_sequence: "{USERNAME}{TAB}{PASSWORD}{ENTER}".to_string(),
            recent_vaults: Vec::new(),
            backup_config: None,
            enabled_policy_ids: None,
            password_policies: None,
            sync_config: None,
            last_sync_at: None,
            last_sync_status: None,
        }
    }
}
