//! Decoy Vault — a fake vault revealed under duress
//!
//! When a user is forced to reveal their vault password, they can provide
//! the decoy password instead. The decoy vault opens normally and contains
//! plausible-looking fake entries, making it indistinguishable from a real vault.
//!
//! # Security model
//! - The decoy vault is a separate KDBX file stored alongside the real vault
//! - The decoy password is completely different from the real password
//! - No metadata links the two vaults
//! - The decoy vault has realistic-looking entries (email, banking, social media)
//! - Accessing the decoy vault does NOT trigger any alert (silent)
//!
//! # Implementation
//! The platform layer checks the provided password against both the real and
//! decoy vault credentials. If the decoy password matches, the decoy vault
//! is opened transparently.

use crate::error::Result;
use crate::types::{Entry, Group};
use crate::vault::Vault;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Configuration for the decoy vault feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoyVaultConfig {
    /// Whether the decoy vault is enabled
    pub enabled: bool,
    /// Path to the decoy vault file (relative to real vault or absolute)
    pub decoy_path: String,
    /// Whether to show the same vault name as the real vault
    pub mirror_vault_name: bool,
}

impl Default for DecoyVaultConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            decoy_path: String::new(),
            mirror_vault_name: true,
        }
    }
}

/// A fake entry for the decoy vault
#[derive(Debug, Clone)]
pub struct DecoyEntry {
    pub title: String,
    pub username: String,
    pub url: String,
    pub notes: String,
}

// ─── Decoy vault generation ───────────────────────────────────────────────────

/// Generate a realistic-looking decoy vault with plausible fake entries.
pub fn generate_decoy_vault(vault_name: &str) -> Vault {
    let mut vault = Vault::new(vault_name);
    let root = vault.root_group_uuid;

    for entry in default_fake_entries() {
        if let Ok(uuid) = vault.create_entry(root) {
            if let Some(e) = vault.get_entry_mut(&uuid) {
                e.title.set(entry.title);
                e.username.set(entry.username);
                e.password.set(generate_fake_password());
                e.url = entry.url;
                e.notes.set(entry.notes);
                e.has_password = true;
            }
        }
    }

    vault
}

/// Default set of plausible fake entries
fn default_fake_entries() -> Vec<DecoyEntry> {
    vec![
        DecoyEntry {
            title: "Gmail".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://mail.google.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "Facebook".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://www.facebook.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "Amazon".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://www.amazon.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "Bank of America".to_string(),
            username: "johndoe1985".to_string(),
            url: "https://www.bankofamerica.com".to_string(),
            notes: "Account: ****4521".to_string(),
        },
        DecoyEntry {
            title: "Netflix".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://www.netflix.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "LinkedIn".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://www.linkedin.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "GitHub".to_string(),
            username: "johndoe".to_string(),
            url: "https://github.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "Twitter / X".to_string(),
            username: "@johndoe".to_string(),
            url: "https://twitter.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "PayPal".to_string(),
            username: "john.doe@gmail.com".to_string(),
            url: "https://www.paypal.com".to_string(),
            notes: String::new(),
        },
        DecoyEntry {
            title: "Apple ID".to_string(),
            username: "john.doe@icloud.com".to_string(),
            url: "https://appleid.apple.com".to_string(),
            notes: String::new(),
        },
    ]
}

/// Generate a plausible-looking fake password (not cryptographically meaningful)
fn generate_fake_password() -> String {
    // A realistic-looking password that won't be used for anything real
    "Tr0ub4dor&3".to_string()
}

// ─── Decoy vault detection ────────────────────────────────────────────────────

/// Check if a given password matches the decoy vault.
///
/// This is called by the platform layer before opening the real vault.
/// Returns `true` if the decoy vault should be opened instead.
pub async fn is_decoy_password(decoy_path: &std::path::Path, password: &str) -> bool {
    use crate::vault::operations::{open_vault, VaultCredentials};
    let creds = VaultCredentials::password_only(password);
    open_vault(decoy_path, &creds).await.is_ok()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_decoy_vault() {
        let vault = generate_decoy_vault("My Vault");
        assert_eq!(vault.meta.name, "My Vault");
        // Should have at least 5 fake entries
        assert!(vault.entry_count() >= 5);
    }

    #[test]
    fn test_default_fake_entries_count() {
        let entries = default_fake_entries();
        assert_eq!(entries.len(), 10);
    }

    #[test]
    fn test_all_fake_entries_have_url() {
        let entries = default_fake_entries();
        for entry in &entries {
            assert!(!entry.url.is_empty(), "Entry '{}' has no URL", entry.title);
            assert!(
                entry.url.starts_with("https://"),
                "Entry '{}' URL should be HTTPS",
                entry.title
            );
        }
    }

    #[test]
    fn test_decoy_config_default() {
        let config = DecoyVaultConfig::default();
        assert!(!config.enabled);
        assert!(config.decoy_path.is_empty());
        assert!(config.mirror_vault_name);
    }
}
