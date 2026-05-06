//! Decoy vault tests — uses the public generate_decoy_vault() API

use crate::decoy_vault::{generate_decoy_vault, DecoyVaultConfig};

#[test]
fn test_decoy_config_default_disabled() {
    let config = DecoyVaultConfig::default();
    assert!(!config.enabled);
    assert!(config.decoy_path.is_empty());
    assert!(config.mirror_vault_name);
}

#[test]
fn test_decoy_config_serialization() {
    let config = DecoyVaultConfig {
        enabled: true,
        decoy_path: "/tmp/decoy.kdbx".to_string(),
        mirror_vault_name: false,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DecoyVaultConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.decoy_path, config.decoy_path);
    assert_eq!(deserialized.mirror_vault_name, config.mirror_vault_name);
}

#[test]
fn test_generate_decoy_vault_has_entries() {
    let vault = generate_decoy_vault("My Vault");
    assert_eq!(vault.meta.name, "My Vault");
    assert!(
        vault.entry_count() >= 5,
        "Decoy vault should have at least 5 entries"
    );
}

#[test]
fn test_generate_decoy_vault_entries_have_passwords() {
    let vault = generate_decoy_vault("Test");
    let entries: Vec<_> = vault.all_entries().collect();
    let with_password = entries
        .iter()
        .filter(|e| !e.password.get().is_empty())
        .count();
    assert!(with_password > 0, "Decoy entries should have passwords");
}

#[test]
fn test_generate_decoy_vault_entries_have_urls() {
    let vault = generate_decoy_vault("Test");
    let entries: Vec<_> = vault.all_entries().collect();
    let with_url = entries.iter().filter(|e| !e.url.is_empty()).count();
    assert!(with_url > 0, "Decoy entries should have URLs");
}

#[test]
fn test_generate_decoy_vault_entries_have_https_urls() {
    let vault = generate_decoy_vault("Test");
    for entry in vault.all_entries() {
        if !entry.url.is_empty() {
            assert!(
                entry.url.starts_with("https://"),
                "Entry '{}' URL should be HTTPS, got: {}",
                entry.title.get(),
                entry.url
            );
        }
    }
}

#[test]
fn test_generate_decoy_vault_entries_have_titles() {
    let vault = generate_decoy_vault("Test");
    for entry in vault.all_entries() {
        assert!(
            !entry.title.get().is_empty(),
            "All decoy entries should have titles"
        );
    }
}

#[test]
fn test_decoy_vault_name_preserved() {
    let vault = generate_decoy_vault("Alice's Vault");
    assert_eq!(vault.meta.name, "Alice's Vault");
}
