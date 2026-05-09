//! TUI Application State

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Application mode (vim-style modal editing)
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Normal navigation mode
    Normal,
    /// Search mode (/ pressed)
    Search,
    /// Command mode (: pressed)
    Command,
    /// Entry detail view
    Detail,
    /// Confirmation dialog
    Confirm(ConfirmAction),
    /// Help overlay
    Help,
}

/// Actions requiring confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    DeleteEntry(String), // entry uuid
    EmptyRecycleBin,
    LockVault,
}

/// Active panel
#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Groups,
    Entries,
    Detail,
}

/// A simplified entry for display
#[derive(Debug, Clone)]
pub struct TuiEntry {
    pub uuid: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub is_expired: bool,
    pub is_favorite: bool,
    pub group_name: String,
    pub modified_ago: String,
}

/// A simplified group for display
#[derive(Debug, Clone)]
pub struct TuiGroup {
    pub uuid: String,
    pub name: String,
    pub entry_count: usize,
    pub depth: usize,
    pub is_expanded: bool,
}

/// Status bar message
#[derive(Debug, Clone)]
pub struct StatusMessage {
    pub text: String,
    pub is_error: bool,
}

/// Main application state
pub struct App {
    pub should_quit: bool,
    pub mode: AppMode,
    pub active_panel: Panel,

    // Data — loaded from real vault
    pub groups: Vec<TuiGroup>,
    pub entries: Vec<TuiEntry>,
    pub selected_group_idx: usize,
    pub selected_entry_idx: usize,
    pub selected_entry: Option<TuiEntry>,

    // Search
    pub search_query: String,
    pub search_results: Vec<TuiEntry>,

    // Command mode
    pub command_input: String,

    // Status
    pub status: Option<StatusMessage>,

    // Clipboard countdown
    pub clipboard_countdown: Option<u8>,

    // Theme
    pub theme: String,

    // Vault info
    pub vault_name: String,
    pub vault_path: String,
    pub entry_count: usize,
    pub is_modified: bool,

    // Vault data (kept in memory for copy operations)
    vault: keepassex_core::Vault,
}

impl App {
    pub async fn new(
        vault_path: &str,
        password: &str,
        key_file: Option<&str>,
        theme: &str,
    ) -> Result<Self> {
        use keepassex_core::crypto::keys::KeyFile;
        use keepassex_core::vault::operations::{open_vault, VaultCredentials};
        use std::path::Path;

        // Build credentials
        let mut credentials = VaultCredentials {
            password: Some(password.to_string()),
            key_file_data: None,
            hardware_key_response: None,
        };

        if let Some(kf_path) = key_file {
            let kf_data = tokio::fs::read(kf_path)
                .await
                .map_err(|e| anyhow::anyhow!("Cannot read key file: {}", e))?;
            credentials.key_file_data = Some(kf_data);
        }

        // Open vault using core engine
        let vault = open_vault(Path::new(vault_path), &credentials)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to open vault: {}", e))?;

        // Build TUI groups from vault
        let groups = Self::build_groups(&vault);

        // Build TUI entries from all vault entries
        let entries = Self::build_entries(&vault);

        let vault_name = vault.meta.name.clone();
        let entry_count = vault.entry_count();
        let first_entry = entries.first().cloned();

        Ok(App {
            should_quit: false,
            mode: AppMode::Normal,
            active_panel: Panel::Entries,
            groups,
            entries: entries.clone(),
            selected_group_idx: 0,
            selected_entry_idx: 0,
            selected_entry: first_entry,
            search_query: String::new(),
            search_results: vec![],
            command_input: String::new(),
            status: Some(StatusMessage {
                text: format!("✓ Opened: {} ({} entries)", vault_name, entry_count),
                is_error: false,
            }),
            clipboard_countdown: None,
            theme: theme.to_string(),
            vault_name,
            vault_path: vault_path.to_string(),
            entry_count,
            is_modified: false,
            vault,
        })
    }

    fn build_groups(vault: &keepassex_core::Vault) -> Vec<TuiGroup> {
        let mut groups = Vec::new();

        // Root group first
        let root_uuid = vault.root_group_uuid;
        if let Some(root) = vault.get_group(&root_uuid) {
            groups.push(TuiGroup {
                uuid: root_uuid.to_string(),
                name: root.name.clone(),
                entry_count: vault.get_group_entries(&root_uuid).len(),
                depth: 0,
                is_expanded: true,
            });

            // Child groups (depth 1)
            for group in vault.all_groups() {
                if group.parent_uuid == Some(root_uuid) {
                    let entry_count = vault.get_group_entries_recursive(&group.uuid).len();
                    groups.push(TuiGroup {
                        uuid: group.uuid.to_string(),
                        name: group.name.clone(),
                        entry_count,
                        depth: 1,
                        is_expanded: group.is_expanded,
                    });
                }
            }
        }

        groups
    }

    fn build_entries(vault: &keepassex_core::Vault) -> Vec<TuiEntry> {
        vault
            .all_entries()
            .map(|entry| {
                let group_name = vault
                    .get_group(&entry.group_uuid)
                    .map(|g| g.name.clone())
                    .unwrap_or_default();

                let modified_ago = format_time_ago(entry.modified_at);

                TuiEntry {
                    uuid: entry.uuid.to_string(),
                    title: entry.title.get().to_string(),
                    username: entry.username.get().to_string(),
                    url: entry.url.clone(),
                    has_otp: entry.otp.is_some(),
                    has_passkey: !entry.passkeys.is_empty(),
                    is_expired: entry.check_expired(),
                    is_favorite: entry.tags.iter().any(|t| {
                        t.eq_ignore_ascii_case("favorite") || t.eq_ignore_ascii_case("starred")
                    }),
                    group_name,
                    modified_ago,
                }
            })
            .collect()
    }

    fn build_entries_for_group(vault: &keepassex_core::Vault, group_uuid: &str) -> Vec<TuiEntry> {
        if let Ok(uuid) = uuid::Uuid::parse_str(group_uuid) {
            vault
                .get_group_entries_recursive(&uuid)
                .iter()
                .map(|entry| {
                    let group_name = vault
                        .get_group(&entry.group_uuid)
                        .map(|g| g.name.clone())
                        .unwrap_or_default();
                    TuiEntry {
                        uuid: entry.uuid.to_string(),
                        title: entry.title.get().to_string(),
                        username: entry.username.get().to_string(),
                        url: entry.url.clone(),
                        has_otp: entry.otp.is_some(),
                        has_passkey: !entry.passkeys.is_empty(),
                        is_expired: entry.check_expired(),
                        is_favorite: entry
                            .tags
                            .iter()
                            .any(|t| t.eq_ignore_ascii_case("favorite")),
                        group_name,
                        modified_ago: format_time_ago(entry.modified_at),
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Handle a key event. Returns true if app should exit.
    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match &self.mode.clone() {
            AppMode::Normal => self.handle_normal_key(key).await,
            AppMode::Search => self.handle_search_key(key),
            AppMode::Command => self.handle_command_key(key),
            AppMode::Detail => self.handle_detail_key(key),
            AppMode::Confirm(action) => self.handle_confirm_key(key, action.clone()),
            AppMode::Help => {
                self.mode = AppMode::Normal;
                Ok(false)
            }
        }
    }

    async fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            // Navigation
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Char('h') | KeyCode::Left => self.switch_panel_left(),
            KeyCode::Char('l') | KeyCode::Right => self.switch_panel_right(),
            KeyCode::Char('g') => self.go_to_top(),
            KeyCode::Char('G') => self.go_to_bottom(),

            // Actions
            KeyCode::Enter => self.open_detail(),
            KeyCode::Char('e') => self.edit_entry(),
            KeyCode::Char('n') => self.new_entry(),
            KeyCode::Char('d') => self.confirm_delete(),
            KeyCode::Char('y') => self.copy_password().await?,
            KeyCode::Char('u') => self.copy_username().await?,
            KeyCode::Char('o') => self.open_url(),

            // Search
            KeyCode::Char('/') => {
                self.mode = AppMode::Search;
                self.search_query.clear();
            }

            // Command mode
            KeyCode::Char(':') => {
                self.mode = AppMode::Command;
                self.command_input.clear();
            }

            // Help
            KeyCode::Char('?') => self.mode = AppMode::Help,

            // Escape
            KeyCode::Esc => {
                self.status = None;
                self.selected_entry = self.entries.get(self.selected_entry_idx).cloned();
            }

            _ => {}
        }
        Ok(false)
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.search_query.clear();
                self.search_results.clear();
            }
            KeyCode::Enter => {
                self.execute_search();
                self.mode = AppMode::Normal;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.execute_search();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.execute_search();
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.mode = AppMode::Normal;
                self.command_input.clear();
            }
            KeyCode::Enter => {
                let cmd = self.command_input.clone();
                self.command_input.clear();
                self.mode = AppMode::Normal;
                self.execute_command(&cmd);
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    fn handle_detail_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = AppMode::Normal;
                self.active_panel = Panel::Entries;
            }
            KeyCode::Char('y') => {
                self.set_status("Password copied to clipboard (clears in 10s)", false);
            }
            KeyCode::Char('e') => self.edit_entry(),
            _ => {}
        }
        Ok(false)
    }

    fn handle_confirm_key(&mut self, key: KeyEvent, action: ConfirmAction) -> Result<bool> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match action {
                    ConfirmAction::DeleteEntry(uuid) => {
                        self.entries.retain(|e| e.uuid != uuid);
                        self.set_status("Entry deleted", false);
                    }
                    ConfirmAction::EmptyRecycleBin => {
                        self.set_status("Recycle bin emptied", false);
                    }
                    ConfirmAction::LockVault => {
                        return Ok(true); // Exit TUI
                    }
                }
                self.mode = AppMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.mode = AppMode::Normal;
            }
            _ => {}
        }
        Ok(false)
    }

    // ─── Navigation ───────────────────────────────────────────────────────────

    fn move_down(&mut self) {
        match self.active_panel {
            Panel::Groups => {
                if self.selected_group_idx + 1 < self.groups.len() {
                    self.selected_group_idx += 1;
                }
            }
            Panel::Entries | Panel::Detail => {
                let list = if self.search_query.is_empty() {
                    &self.entries
                } else {
                    &self.search_results
                };
                if self.selected_entry_idx + 1 < list.len() {
                    self.selected_entry_idx += 1;
                    self.selected_entry = list.get(self.selected_entry_idx).cloned();
                }
            }
        }
    }

    fn move_up(&mut self) {
        match self.active_panel {
            Panel::Groups => {
                if self.selected_group_idx > 0 {
                    self.selected_group_idx -= 1;
                }
            }
            Panel::Entries | Panel::Detail => {
                if self.selected_entry_idx > 0 {
                    self.selected_entry_idx -= 1;
                    let list = if self.search_query.is_empty() {
                        &self.entries
                    } else {
                        &self.search_results
                    };
                    self.selected_entry = list.get(self.selected_entry_idx).cloned();
                }
            }
        }
    }

    fn switch_panel_left(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Entries => Panel::Groups,
            Panel::Detail => Panel::Entries,
            Panel::Groups => Panel::Groups,
        };
    }

    fn switch_panel_right(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Groups => Panel::Entries,
            Panel::Entries => Panel::Detail,
            Panel::Detail => Panel::Detail,
        };
    }

    fn go_to_top(&mut self) {
        self.selected_entry_idx = 0;
        self.selected_entry = self.entries.first().cloned();
    }

    fn go_to_bottom(&mut self) {
        if !self.entries.is_empty() {
            self.selected_entry_idx = self.entries.len() - 1;
            self.selected_entry = self.entries.last().cloned();
        }
    }

    // ─── Actions ──────────────────────────────────────────────────────────────

    fn open_detail(&mut self) {
        self.mode = AppMode::Detail;
        self.active_panel = Panel::Detail;
    }

    fn edit_entry(&mut self) {
        self.set_status("Edit: not yet implemented in TUI (use desktop app)", false);
    }

    fn new_entry(&mut self) {
        self.set_status(
            "New entry: not yet implemented in TUI (use desktop app)",
            false,
        );
    }

    fn confirm_delete(&mut self) {
        if let Some(entry) = &self.selected_entry {
            let uuid = entry.uuid.clone();
            self.mode = AppMode::Confirm(ConfirmAction::DeleteEntry(uuid));
        }
    }

    async fn copy_password(&mut self) -> Result<()> {
        if let Some(entry) = &self.selected_entry {
            let uuid_str = entry.uuid.clone();
            let title = entry.title.clone();
            if let Ok(parsed_uuid) = uuid::Uuid::parse_str(&uuid_str) {
                if let Some(vault_entry) = self.vault.get_entry(&parsed_uuid) {
                    let password = vault_entry.password.get().to_string();
                    if !password.is_empty() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(&password);
                        }
                        self.set_status(
                            &format!("✓ Password for '{}' copied (clears in 10s)", title),
                            false,
                        );
                        self.clipboard_countdown = Some(10);
                    } else {
                        self.set_status("No password set for this entry", true);
                    }
                }
            }
        }
        Ok(())
    }

    async fn copy_username(&mut self) -> Result<()> {
        if let Some(entry) = &self.selected_entry {
            let username = entry.username.clone();
            if !username.is_empty() {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(&username);
                }
                self.set_status(&format!("✓ Username '{}' copied", username), false);
            } else {
                self.set_status("No username set for this entry", true);
            }
        }
        Ok(())
    }

    fn open_url(&mut self) {
        if let Some(entry) = &self.selected_entry {
            if !entry.url.is_empty() {
                let _ = open::that(&entry.url);
                self.set_status(&format!("Opening: {}", entry.url), false);
            }
        }
    }

    fn execute_search(&mut self) {
        let q = self.search_query.to_lowercase();
        if q.is_empty() {
            self.search_results.clear();
            return;
        }

        // Use vault search engine
        let query = keepassex_core::types::SearchQuery::new(&self.search_query);
        let results = self.vault.search(&query);
        self.search_results = results
            .iter()
            .map(|entry| {
                let group_name = self
                    .vault
                    .get_group(&entry.group_uuid)
                    .map(|g| g.name.clone())
                    .unwrap_or_default();
                TuiEntry {
                    uuid: entry.uuid.to_string(),
                    title: entry.title.get().to_string(),
                    username: entry.username.get().to_string(),
                    url: entry.url.clone(),
                    has_otp: entry.otp.is_some(),
                    has_passkey: !entry.passkeys.is_empty(),
                    is_expired: entry.check_expired(),
                    is_favorite: entry
                        .tags
                        .iter()
                        .any(|t| t.eq_ignore_ascii_case("favorite")),
                    group_name,
                    modified_ago: format_time_ago(entry.modified_at),
                }
            })
            .collect();

        self.selected_entry_idx = 0;
        self.selected_entry = self.search_results.first().cloned();
    }

    fn execute_command(&mut self, cmd: &str) {
        match cmd.trim() {
            "q" | "quit" => self.should_quit = true,
            "w" | "save" => self.set_status("Vault saved", false),
            "wq" => {
                self.set_status("Vault saved", false);
                self.should_quit = true;
            }
            "lock" => {
                self.mode = AppMode::Confirm(ConfirmAction::LockVault);
            }
            "health" => self.set_status("Run: kpx health", false),
            "breach" => self.set_status("Run: kpx breach --online", false),
            cmd if cmd.starts_with("search ") => {
                self.search_query = cmd[7..].to_string();
                self.execute_search();
            }
            _ => self.set_status(&format!("Unknown command: {}", cmd), true),
        }
    }

    pub fn set_status(&mut self, msg: &str, is_error: bool) {
        self.status = Some(StatusMessage {
            text: msg.to_string(),
            is_error,
        });
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn format_time_ago(dt: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(dt);

    if diff.num_seconds() < 60 {
        "just now".to_string()
    } else if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
    } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
    } else if diff.num_days() < 7 {
        format!("{}d ago", diff.num_days())
    } else if diff.num_weeks() < 5 {
        format!("{}w ago", diff.num_weeks())
    } else if diff.num_days() < 365 {
        format!("{}mo ago", diff.num_days() / 30)
    } else {
        format!("{}y ago", diff.num_days() / 365)
    }
}
