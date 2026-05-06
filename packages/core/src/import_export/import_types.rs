//! Import-specific entry/group types used by all importers.
//!
//! These types use String fields (not Uuid/ProtectedString) for ease of
//! construction in importers. They are converted to canonical types via
//! `into_entry()` / `into_group()` before being added to the vault.

use crate::types::{
    AutoTypeConfig, AutoTypeObfuscation, CustomField, Entry, Group, ProtectedString,
};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

/// Importer-friendly entry (all String fields, Vec for custom_fields)
#[derive(Debug, Clone, Default)]
pub struct ImportEntry {
    pub uuid: String,
    pub group_uuid: String,
    pub title: String,
    pub username: String,
    pub password: Option<String>,
    pub url: String,
    pub notes: String,
    pub icon_id: u32,
    pub tags: Vec<String>,
    pub custom_fields: Vec<ImportCustomField>,
    pub has_password: bool,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub has_ssh_key: bool,
    pub has_attachments: bool,
    pub is_expired: bool,
    pub expiry: Option<String>,
    pub created_at: String,
    pub modified_at: String,
    pub usage_count: u32,
    pub auto_type_enabled: bool,
    pub auto_type_sequence: Option<String>,
    pub quality_check: bool,
}

/// Importer-friendly custom field
#[derive(Debug, Clone, Default)]
pub struct ImportCustomField {
    pub key: String,
    pub value: String,
    pub protected: bool,
}

/// Importer-friendly group
#[derive(Debug, Clone, Default)]
pub struct ImportGroup {
    pub uuid: String,
    pub parent_uuid: Option<String>,
    pub name: String,
    pub notes: String,
    pub icon_id: u32,
    pub is_expanded: bool,
    pub entry_count: usize,
    pub child_group_count: usize,
    pub enable_auto_type: Option<bool>,
    pub enable_searching: Option<bool>,
    pub default_auto_type_sequence: Option<String>,
    pub last_top_visible_entry: Option<String>,
}

/// Result of an import operation
#[derive(Debug, Default)]
pub struct ImportResult {
    pub entries: Vec<ImportEntry>,
    pub groups: Vec<ImportGroup>,
    pub warnings: Vec<String>,
}

impl ImportEntry {
    /// Convert to canonical `Entry` type
    pub fn into_entry(self) -> Entry {
        let now = Utc::now();

        // Parse UUID or generate new one
        let uuid = Uuid::parse_str(&self.uuid).unwrap_or_else(|_| Uuid::new_v4());
        let group_uuid = Uuid::parse_str(&self.group_uuid).unwrap_or_else(|_| Uuid::new_v4());

        // Parse timestamps
        let created_at = chrono::DateTime::parse_from_rfc3339(&self.created_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(now);
        let modified_at = chrono::DateTime::parse_from_rfc3339(&self.modified_at)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or(now);

        // Convert custom fields Vec → HashMap
        let custom_fields: HashMap<String, CustomField> = self
            .custom_fields
            .into_iter()
            .map(|f| {
                let key = f.key.clone();
                let cf = CustomField {
                    key: f.key,
                    value: ProtectedString::new(f.value),
                    protected: f.protected,
                };
                (key, cf)
            })
            .collect();

        Entry {
            uuid,
            group_uuid,
            title: ProtectedString::new(self.title),
            username: ProtectedString::new(self.username),
            password: ProtectedString::new(self.password.unwrap_or_default()),
            url: self.url,
            notes: ProtectedString::new(self.notes),
            icon_id: self.icon_id,
            custom_icon_uuid: None,
            tags: self.tags,
            custom_fields,
            attachments: Vec::new(),
            otp: None,
            passkeys: Vec::new(),
            ssh_key: None,
            expiry: self.expiry.and_then(|e| {
                chrono::DateTime::parse_from_rfc3339(&e)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            }),
            created_at,
            modified_at,
            accessed_at: now,
            history: Vec::new(),
            auto_type: AutoTypeConfig {
                enabled: self.auto_type_enabled,
                obfuscation: AutoTypeObfuscation::None,
                default_sequence: self.auto_type_sequence,
                associations: Vec::new(),
            },
            foreground_color: None,
            background_color: None,
            override_url: None,
            quality_check: self.quality_check,
            additional_urls: Vec::new(),
            auto_type_enabled: Some(self.auto_type_enabled),
            auto_type_obfuscation: None,
            auto_type_sequence: None,
            usage_count: self.usage_count,
            has_password: self.has_password,
            has_otp: self.has_otp,
            has_passkey: self.has_passkey,
            has_ssh_key: self.has_ssh_key,
            has_attachments: self.has_attachments,
            is_expired: self.is_expired,
        }
    }
}

impl ImportGroup {
    /// Convert to canonical `Group` type
    pub fn into_group(self) -> Group {
        let now = Utc::now();
        let uuid = Uuid::parse_str(&self.uuid).unwrap_or_else(|_| Uuid::new_v4());
        let parent_uuid = self
            .parent_uuid
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok());
        let last_top = self
            .last_top_visible_entry
            .as_deref()
            .and_then(|s| Uuid::parse_str(s).ok());

        Group {
            uuid,
            parent_uuid,
            name: self.name,
            notes: self.notes,
            icon_id: self.icon_id,
            custom_icon_uuid: None,
            is_expanded: self.is_expanded,
            auto_type_enabled: self.enable_auto_type,
            search_enabled: self.enable_searching,
            last_top_visible_entry: last_top,
            created_at: now,
            modified_at: now,
            accessed_at: now,
            tags: Vec::new(),
            enable_auto_type: self.enable_auto_type,
            enable_searching: self.enable_searching,
            default_auto_type_sequence: self.default_auto_type_sequence,
            entry_count: self.entry_count,
            child_group_count: self.child_group_count,
        }
    }
}
