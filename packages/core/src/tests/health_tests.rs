//! Health audit tests

use crate::vault::Vault;
use crate::health::audit_vault;

fn make_vault_with_entries() -> Vault {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    // Strong unique password
    let uuid1 = vault.create_entry(root).unwrap();
    let entry1 = vault.get_entry_mut(&uuid1).unwrap();
    entry1.title.set("GitHub");
    entry1.password.set("Xk9#mP2$vL7@nQ4!zR8^");
    entry1.url = "https://github.com".to_string();

    // Weak password
    let uuid2 = vault.create_entry(root).unwrap();
    let entry2 = vault.get_entry_mut(&uuid2).unwrap();
    entry2.title.set("Old Site");
    entry2.password.set("abc");

    // Reused password (same as uuid1)
    let uuid3 = vault.create_entry(root).unwrap();
    let entry3 = vault.get_entry_mut(&uuid3).unwrap();
    entry3.title.set("GitLab");
    entry3.password.set("Xk9#mP2$vL7@nQ4!zR8^"); // same as GitHub

    vault
}

#[test]
fn test_audit_detects_weak_passwords() {
    let vault = make_vault_with_entries();
    let report = audit_vault(&vault);

    assert!(report.weak_passwords.iter().any(|w| w.entry_title == "Old Site"));
}

#[test]
fn test_audit_detects_reused_passwords() {
    let vault = make_vault_with_entries();
    let report = audit_vault(&vault);

    assert!(!report.reused_passwords.is_empty());
    let reused_group = &report.reused_passwords[0];
    assert_eq!(reused_group.entries.len(), 2);
}

#[test]
fn test_audit_score_perfect_vault() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    // Add only strong, unique passwords
    for i in 0..5 {
        let uuid = vault.create_entry(root).unwrap();
        let entry = vault.get_entry_mut(&uuid).unwrap();
        entry.title.set(format!("Entry {}", i));
        entry.password.set(format!("Xk9#mP2$vL7@nQ4!zR8^{}", i));
        entry.url = format!("https://site{}.com", i);
    }

    let report = audit_vault(&vault);
    assert!(report.score >= 90);
    assert!(report.weak_passwords.is_empty());
    assert!(report.reused_passwords.is_empty());
}

#[test]
fn test_audit_detects_expired_entries() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    let entry = vault.get_entry_mut(&uuid).unwrap();
    entry.title.set("Expired Entry");
    entry.password.set("StrongP@ss123!");
    // Set expiry in the past
    entry.expiry = Some(chrono::Utc::now() - chrono::Duration::days(30));

    let report = audit_vault(&vault);
    assert_eq!(report.expired_entries.len(), 1);
    assert_eq!(report.expired_entries[0].entry_title, "Expired Entry");
}

#[test]
fn test_audit_empty_vault() {
    let vault = Vault::new("Empty");
    let report = audit_vault(&vault);

    assert_eq!(report.total_entries, 0);
    assert_eq!(report.score, 100);
    assert!(report.weak_passwords.is_empty());
}
