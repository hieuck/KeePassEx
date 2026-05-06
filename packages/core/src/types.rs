//! Core domain types for KeePassEx

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use zeroize::Zeroize;

// ─── Entry ───────────────────────────────────────────────────────────────────

/// A single password entry in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub uuid: Uuid,
    pub group_uuid: Uuid,
    pub title: ProtectedString,
    pub username: ProtectedString,
    pub password: ProtectedString,
    pub url: String,
    pub notes: ProtectedString,
    pub icon_id: u32,
    pub custom_icon_uuid: Option<Uuid>,
    pub tags: Vec<String>,
    pub custom_fields: HashMap<String, CustomField>,
    pub attachments: Vec<AttachmentRef>,
    pub otp: Option<OtpConfig>,
    pub passkeys: Vec<PasskeyEntry>,
    pub ssh_key: Option<SshKeyEntry>,
    pub expiry: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub history: Vec<HistoryEntry>,
    pub auto_type: AutoTypeConfig,
    pub foreground_color: Option<String>,
    pub background_color: Option<String>,
    pub override_url: Option<String>,
    pub quality_check: bool,
    // ─── Computed / convenience fields ───────────────────────────────────────
    /// Additional URLs for this entry (beyond the primary url)
    #[serde(default)]
    pub additional_urls: Vec<String>,
    /// Auto-type enabled flag (mirrors auto_type.enabled)
    #[serde(default)]
    pub auto_type_enabled: Option<bool>,
    /// Auto-type obfuscation (mirrors auto_type.obfuscation)
    #[serde(default)]
    pub auto_type_obfuscation: Option<u32>,
    /// Default auto-type sequence override
    #[serde(default)]
    pub auto_type_sequence: Option<String>,
    /// How many times this entry has been accessed
    #[serde(default)]
    pub usage_count: u32,
    // ─── Cached computed fields (set by importers/vault operations) ──────────
    /// Cached: does this entry have a non-empty password?
    #[serde(default)]
    pub has_password: bool,
    /// Cached: does this entry have OTP configured?
    #[serde(default)]
    pub has_otp: bool,
    /// Cached: does this entry have a passkey?
    #[serde(default)]
    pub has_passkey: bool,
    /// Cached: does this entry have an SSH key?
    #[serde(default)]
    pub has_ssh_key: bool,
    /// Cached: does this entry have attachments?
    #[serde(default)]
    pub has_attachments: bool,
    /// Cached: is this entry expired?
    #[serde(default)]
    pub is_expired: bool,
}

impl Entry {
    pub fn new(group_uuid: Uuid) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            group_uuid,
            title: ProtectedString::new(""),
            username: ProtectedString::new(""),
            password: ProtectedString::new(""),
            url: String::new(),
            notes: ProtectedString::new(""),
            icon_id: 0,
            custom_icon_uuid: None,
            tags: Vec::new(),
            custom_fields: HashMap::new(),
            attachments: Vec::new(),
            otp: None,
            passkeys: Vec::new(),
            ssh_key: None,
            expiry: None,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            history: Vec::new(),
            auto_type: AutoTypeConfig::default(),
            foreground_color: None,
            background_color: None,
            override_url: None,
            quality_check: true,
            additional_urls: Vec::new(),
            auto_type_enabled: None,
            auto_type_obfuscation: None,
            auto_type_sequence: None,
            usage_count: 0,
            has_password: false,
            has_otp: false,
            has_passkey: false,
            has_ssh_key: false,
            has_attachments: false,
            is_expired: false,
        }
    }

    /// Check if entry is expired (computed from expiry date)
    pub fn check_expired(&self) -> bool {
        self.expiry.map(|e| e < Utc::now()).unwrap_or(false)
    }

    /// Check if entry expires within `days` days
    pub fn expires_within_days(&self, days: i64) -> bool {
        self.expiry
            .map(|e| {
                let diff = e - Utc::now();
                diff.num_days() <= days && diff.num_days() >= 0
            })
            .unwrap_or(false)
    }

    /// Convenience: does this entry have a non-empty password?
    pub fn has_password_value(&self) -> bool {
        !self.password.get().is_empty()
    }

    /// Convenience: does this entry have OTP configured?
    pub fn has_otp_config(&self) -> bool {
        self.otp.is_some()
    }

    /// Convenience: does this entry have a passkey?
    pub fn has_passkey_entry(&self) -> bool {
        !self.passkeys.is_empty()
    }

    /// Convenience: does this entry have an SSH key?
    pub fn has_ssh_key_entry(&self) -> bool {
        self.ssh_key.is_some()
    }

    /// Convenience: does this entry have attachments?
    pub fn has_attachment_files(&self) -> bool {
        !self.attachments.is_empty()
    }
}

// ─── Group ───────────────────────────────────────────────────────────────────

/// A group (folder) containing entries and sub-groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub name: String,
    pub notes: String,
    pub icon_id: u32,
    pub custom_icon_uuid: Option<Uuid>,
    pub is_expanded: bool,
    pub auto_type_enabled: Option<bool>,
    pub search_enabled: Option<bool>,
    pub last_top_visible_entry: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub tags: Vec<String>,
    // ─── Extended fields used by importers ───────────────────────────────────
    /// Whether auto-type is enabled for this group (KDBX field)
    #[serde(default)]
    pub enable_auto_type: Option<bool>,
    /// Whether searching is enabled for this group (KDBX field)
    #[serde(default)]
    pub enable_searching: Option<bool>,
    /// Default auto-type sequence for this group
    #[serde(default)]
    pub default_auto_type_sequence: Option<String>,
    /// Number of direct child entries (computed, not stored)
    #[serde(default)]
    pub entry_count: usize,
    /// Number of direct child groups (computed, not stored)
    #[serde(default)]
    pub child_group_count: usize,
}

impl Group {
    pub fn new(name: impl Into<String>, parent_uuid: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4(),
            parent_uuid,
            name: name.into(),
            notes: String::new(),
            icon_id: 48,
            custom_icon_uuid: None,
            is_expanded: true,
            auto_type_enabled: None,
            search_enabled: None,
            last_top_visible_entry: None,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            tags: Vec::new(),
            enable_auto_type: None,
            enable_searching: None,
            default_auto_type_sequence: None,
            entry_count: 0,
            child_group_count: 0,
        }
    }
}

// ─── Protected String ─────────────────────────────────────────────────────────

/// A string that is zeroed from memory when dropped
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedString {
    #[serde(skip)]
    value: String,
    /// Serialized as encrypted blob in KDBX
    pub protected: bool,
}

impl ProtectedString {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            protected: true,
        }
    }

    pub fn get(&self) -> &str {
        &self.value
    }

    pub fn set(&mut self, value: impl Into<String>) {
        self.value.zeroize();
        self.value = value.into();
    }
}

impl Drop for ProtectedString {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

// Allow comparing ProtectedString with &str and String directly
impl PartialEq<str> for ProtectedString {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<&str> for ProtectedString {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl PartialEq<String> for ProtectedString {
    fn eq(&self, other: &String) -> bool {
        self.value == *other
    }
}

impl PartialEq for ProtectedString {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl std::fmt::Display for ProtectedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never display the actual value — show placeholder
        write!(f, "***")
    }
}

impl From<String> for ProtectedString {
    fn from(s: String) -> Self {
        ProtectedString::new(s)
    }
}

impl From<&str> for ProtectedString {
    fn from(s: &str) -> Self {
        ProtectedString::new(s)
    }
}

impl From<Option<String>> for ProtectedString {
    fn from(opt: Option<String>) -> Self {
        ProtectedString::new(opt.unwrap_or_default())
    }
}

// ─── Custom Field ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub key: String,
    pub value: ProtectedString,
    /// Whether this field is protected (encrypted in KDBX inner stream)
    #[serde(default)]
    pub protected: bool,
}

impl CustomField {
    pub fn new(key: impl Into<String>, value: impl Into<String>, protected: bool) -> Self {
        Self {
            key: key.into(),
            value: ProtectedString::new(value),
            protected,
        }
    }

    pub fn plain(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(key, value, false)
    }

    pub fn secret(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(key, value, true)
    }
}

// ─── Attachment ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentRef {
    pub name: String,
    pub data_ref: Uuid, // references binary pool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub uuid: Uuid,
    pub name: String,
    pub data: Vec<u8>,
    pub mime_type: Option<String>,
}

// ─── OTP ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtpConfig {
    pub secret: ProtectedString,
    pub algorithm: OtpAlgorithm,
    pub digits: u8,
    pub period: u64,  // TOTP period in seconds
    pub counter: u64, // HOTP counter
    pub otp_type: OtpType,
    pub issuer: Option<String>,
    pub account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OtpType {
    Totp,
    Hotp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OtpAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

// ─── Passkey ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasskeyEntry {
    pub credential_id: Vec<u8>,
    pub rp_id: String,
    pub rp_name: String,
    pub user_id: Vec<u8>,
    pub user_name: String,
    pub user_display_name: String,
    pub private_key: ProtectedString, // PKCS#8 PEM, protected
    pub sign_count: u64,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub backup_eligible: bool,
    pub backup_state: bool,
}

// ─── SSH Key ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyEntry {
    pub key_type: SshKeyType,
    pub private_key: ProtectedString, // OpenSSH format, protected
    pub public_key: String,
    pub comment: String,
    pub fingerprint: String,
    pub add_to_agent: bool,
    pub agent_duration: Option<u64>, // seconds, None = forever
    pub confirm_before_use: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SshKeyType {
    Ed25519,
    Rsa2048,
    Rsa4096,
    EcdsaP256,
    EcdsaP384,
}

// ─── AutoType ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoTypeConfig {
    pub enabled: bool,
    pub obfuscation: AutoTypeObfuscation,
    pub default_sequence: Option<String>,
    pub associations: Vec<AutoTypeAssociation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum AutoTypeObfuscation {
    #[default]
    None,
    UseClipboard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTypeAssociation {
    pub window: String,
    pub sequence: Option<String>,
}

// ─── History ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub modified_at: DateTime<Utc>,
    pub entry_snapshot: Box<Entry>,
}

// ─── Vault Metadata ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMeta {
    pub name: String,
    pub description: String,
    pub default_username: String,
    pub maintenance_history_days: u32,
    pub max_history_items: u32,
    pub max_history_size: u64,
    pub recycle_bin_enabled: bool,
    pub recycle_bin_uuid: Option<Uuid>,
    pub entry_templates_group: Option<Uuid>,
    pub last_selected_group: Option<Uuid>,
    pub last_top_visible_group: Option<Uuid>,
    pub history_max_items: i32,
    pub history_max_size: i64,
    pub custom_data: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    /// Emergency access grants (stored in vault custom data)
    #[serde(default)]
    pub emergency_access: crate::emergency_access::EmergencyAccessManager,
}

impl Default for VaultMeta {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: "KeePassEx Vault".to_string(),
            description: String::new(),
            default_username: String::new(),
            maintenance_history_days: 365,
            max_history_items: 10,
            max_history_size: 6 * 1024 * 1024, // 6 MB
            recycle_bin_enabled: true,
            recycle_bin_uuid: None,
            entry_templates_group: None,
            last_selected_group: None,
            last_top_visible_group: None,
            history_max_items: 10,
            history_max_size: 6 * 1024 * 1024,
            custom_data: HashMap::new(),
            created_at: now,
            modified_at: now,
            emergency_access: crate::emergency_access::EmergencyAccessManager::new(),
        }
    }
}

// ─── Search ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub text: String,
    pub search_title: bool,
    pub search_username: bool,
    pub search_password: bool,
    pub search_url: bool,
    pub search_notes: bool,
    pub search_tags: bool,
    pub search_custom_fields: bool,
    pub case_sensitive: bool,
    pub regex: bool,
    pub exclude_expired: bool,
    pub group_uuid: Option<Uuid>,
    pub recursive: bool,
}

impl SearchQuery {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            search_title: true,
            search_username: true,
            search_url: true,
            search_tags: true,
            recursive: true,
            ..Default::default()
        }
    }
}

// ─── Password Generator Config ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordGeneratorConfig {
    pub mode: GeneratorMode,
    pub length: usize,
    pub use_uppercase: bool,
    pub use_lowercase: bool,
    pub use_digits: bool,
    pub use_symbols: bool,
    pub custom_symbols: Option<String>,
    pub exclude_ambiguous: bool,
    pub exclude_chars: String,
    pub min_uppercase: usize,
    pub min_lowercase: usize,
    pub min_digits: usize,
    pub min_symbols: usize,
    // Passphrase options
    pub word_count: usize,
    pub word_separator: String,
    pub capitalize_words: bool,
    pub include_number: bool,
    pub wordlist: WordList,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeneratorMode {
    Random,
    Passphrase,
    Pronounceable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WordList {
    Eff,
    Bip39,
    Custom(String),
}

impl Default for PasswordGeneratorConfig {
    fn default() -> Self {
        Self {
            mode: GeneratorMode::Random,
            length: 20,
            use_uppercase: true,
            use_lowercase: true,
            use_digits: true,
            use_symbols: true,
            custom_symbols: None,
            exclude_ambiguous: false,
            exclude_chars: String::new(),
            min_uppercase: 1,
            min_lowercase: 1,
            min_digits: 1,
            min_symbols: 1,
            word_count: 6,
            word_separator: "-".to_string(),
            capitalize_words: false,
            include_number: false,
            wordlist: WordList::Eff,
        }
    }
}
