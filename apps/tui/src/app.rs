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

    // Data
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
}

impl App {
    pub async fn new(vault_path: &str, password: &str, theme: &str) -> Result<Self> {
        // In production: open vault and load entries
        // For now: create with mock data to demonstrate TUI structure
        let mock_entries = vec![
            TuiEntry {
                uuid: "uuid-1".into(),
                title: "GitHub".into(),
                username: "user@example.com".into(),
                url: "https://github.com".into(),
                has_otp: true,
                has_passkey: false,
                is_expired: false,
                is_favorite: true,
                group_name: "Development".into(),
                modified_ago: "2 days ago".into(),
            },
            TuiEntry {
                uuid: "uuid-2".into(),
                title: "Gmail".into(),
                username: "user@gmail.com".into(),
                url: "https://gmail.com".into(),
                has_otp: true,
                has_passkey: true,
                is_expired: false,
                is_favorite: false,
                group_name: "Email".into(),
                modified_ago: "1 week ago".into(),
            },
            TuiEntry {
                uuid: "uuid-3".into(),
                title: "Bank Account".into(),
                username: "user123".into(),
                url: "https://bank.example.com".into(),
                has_otp: false,
                has_passkey: false,
                is_expired: true,
                is_favorite: false,
                group_name: "Banking".into(),
                modified_ago: "3 months ago".into(),
            },
        ];

        let mock_groups = vec![
            TuiGroup {
                uuid: "g-root".into(),
                name: "All Entries".into(),
                entry_count: 3,
                depth: 0,
                is_expanded: true,
            },
            TuiGroup {
                uuid: "g-dev".into(),
                name: "Development".into(),
                entry_count: 1,
                depth: 1,
                is_expanded: true,
            },
            TuiGroup {
                uuid: "g-email".into(),
                name: "Email".into(),
                entry_count: 1,
                depth: 1,
                is_expanded: false,
            },
            TuiGroup {
                uuid: "g-bank".into(),
                name: "Banking".into(),
                entry_count: 1,
                depth: 1,
                is_expanded: false,
            },
        ];

        Ok(App {
            should_quit: false,
            mode: AppMode::Normal,
            active_panel: Panel::Entries,
            groups: mock_groups,
            entries: mock_entries.clone(),
            selected_group_idx: 0,
            selected_entry_idx: 0,
            selected_entry: mock_entries.first().cloned(),
            search_query: String::new(),
            search_results: vec![],
            command_input: String::new(),
            status: Some(StatusMessage {
                text: format!("Opened: {}", vault_path),
                is_error: false,
            }),
            clipboard_countdown: None,
            theme: theme.to_string(),
            vault_name: "KeePassEx Vault".into(),
            vault_path: vault_path.to_string(),
            entry_count: 3,
            is_modified: false,
        })
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
        self.set_status("New entry: not yet implemented in TUI (use desktop app)", false);
    }

    fn confirm_delete(&mut self) {
        if let Some(entry) = &self.selected_entry {
            let uuid = entry.uuid.clone();
            self.mode = AppMode::Confirm(ConfirmAction::DeleteEntry(uuid));
        }
    }

    async fn copy_password(&mut self) -> Result<()> {
        if let Some(entry) = &self.selected_entry {
            // In production: copy actual password from vault
            self.set_status(
                &format!("Password for '{}' copied (clears in 10s)", entry.title),
                false,
            );
            self.clipboard_countdown = Some(10);
        }
        Ok(())
    }

    async fn copy_username(&mut self) -> Result<()> {
        if let Some(entry) = &self.selected_entry {
            self.set_status(
                &format!("Username '{}' copied", entry.username),
                false,
            );
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
        self.search_results = self
            .entries
            .iter()
            .filter(|e| {
                e.title.to_lowercase().contains(&q)
                    || e.username.to_lowercase().contains(&q)
                    || e.url.to_lowercase().contains(&q)
                    || e.group_name.to_lowercase().contains(&q)
            })
            .cloned()
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
