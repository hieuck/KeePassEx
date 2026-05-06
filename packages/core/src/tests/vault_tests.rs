//! Vault CRUD tests

use crate::vault::Vault;
use crate::types::SearchQuery;

#[test]
fn test_create_vault() {
    let vault = Vault::new("Test Vault");
    assert_eq!(vault.meta.name, "Test Vault");
    assert_eq!(vault.entry_count(), 0);
    assert_eq!(vault.group_count(), 1); // root group
}

#[test]
fn test_create_and_get_entry() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    assert_eq!(vault.entry_count(), 1);

    let entry = vault.get_entry(&uuid).unwrap();
    assert_eq!(entry.uuid, uuid);
    assert_eq!(entry.group_uuid, root);
}

#[test]
fn test_update_entry() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let uuid = vault.create_entry(root).unwrap();

    let mut entry = vault.get_entry(&uuid).unwrap().clone();
    entry.title.set("My Entry");
    entry.username.set("user@example.com");
    entry.password.set("SecureP@ss123!");
    entry.url = "https://example.com".to_string();

    vault.update_entry(entry).unwrap();

    let updated = vault.get_entry(&uuid).unwrap();
    assert_eq!(updated.title.get(), "My Entry");
    assert_eq!(updated.username.get(), "user@example.com");
    assert_eq!(updated.url, "https://example.com");
}

#[test]
fn test_delete_entry_to_recycle_bin() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let uuid = vault.create_entry(root).unwrap();

    vault.delete_entry(&uuid, false).unwrap();

    // Entry should be in recycle bin, not deleted
    assert!(vault.get_entry(&uuid).is_some());
    assert_eq!(vault.entry_count(), 1);
}

#[test]
fn test_delete_entry_permanently() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let uuid = vault.create_entry(root).unwrap();

    vault.delete_entry(&uuid, true).unwrap();

    assert!(vault.get_entry(&uuid).is_none());
    assert_eq!(vault.entry_count(), 0);
}

#[test]
fn test_create_group() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let group_uuid = vault.create_group("Work", root).unwrap();
    assert_eq!(vault.group_count(), 2); // root + Work

    let group = vault.get_group(&group_uuid).unwrap();
    assert_eq!(group.name, "Work");
    assert_eq!(group.parent_uuid, Some(root));
}

#[test]
fn test_nested_groups() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let work = vault.create_group("Work", root).unwrap();
    let dev = vault.create_group("Development", work).unwrap();
    let prod = vault.create_group("Production", dev).unwrap();

    assert_eq!(vault.group_count(), 4);

    let prod_group = vault.get_group(&prod).unwrap();
    assert_eq!(prod_group.parent_uuid, Some(dev));
}

#[test]
fn test_cannot_delete_root_group() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let result = vault.delete_group(&root, true);
    assert!(result.is_err());
}

#[test]
fn test_move_entry() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let work = vault.create_group("Work", root).unwrap();

    let entry_uuid = vault.create_entry(root).unwrap();
    vault.move_entry(&entry_uuid, work).unwrap();

    let entry = vault.get_entry(&entry_uuid).unwrap();
    assert_eq!(entry.group_uuid, work);
}

#[test]
fn test_duplicate_entry() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let uuid = vault.create_entry(root).unwrap();

    {
        let entry = vault.get_entry_mut(&uuid).unwrap();
        entry.title.set("Original");
        entry.password.set("secret123");
    }

    let dup_uuid = vault.duplicate_entry(&uuid).unwrap();
    assert_ne!(uuid, dup_uuid);

    let dup = vault.get_entry(&dup_uuid).unwrap();
    assert!(dup.title.get().contains("copy"));
    assert_eq!(dup.password.get(), "secret123");
}

#[test]
fn test_search_by_title() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid1 = vault.create_entry(root).unwrap();
    vault.get_entry_mut(&uuid1).unwrap().title.set("GitHub");

    let uuid2 = vault.create_entry(root).unwrap();
    vault.get_entry_mut(&uuid2).unwrap().title.set("GitLab");

    let uuid3 = vault.create_entry(root).unwrap();
    vault.get_entry_mut(&uuid3).unwrap().title.set("Amazon");

    let query = SearchQuery::new("git");
    let results = vault.search(&query);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_case_insensitive() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    vault.get_entry_mut(&uuid).unwrap().title.set("GITHUB");

    let query = SearchQuery::new("github");
    let results = vault.search(&query);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_empty_recycle_bin() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    // Add entries and delete them to recycle bin
    for _ in 0..3 {
        let uuid = vault.create_entry(root).unwrap();
        vault.delete_entry(&uuid, false).unwrap();
    }

    let deleted = vault.empty_recycle_bin().unwrap();
    assert_eq!(deleted, 3);
    assert_eq!(vault.entry_count(), 0);
}

#[test]
fn test_entry_history() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let uuid = vault.create_entry(root).unwrap();

    // Update entry multiple times
    for i in 0..3 {
        let mut entry = vault.get_entry(&uuid).unwrap().clone();
        entry.password.set(format!("password{}", i));
        vault.update_entry(entry).unwrap();
    }

    let entry = vault.get_entry(&uuid).unwrap();
    assert_eq!(entry.history.len(), 3);
}

#[test]
fn test_vault_dirty_flag() {
    let mut vault = Vault::new("Test");
    assert!(!vault.dirty);

    let root = vault.root_group_uuid;
    vault.create_entry(root).unwrap();
    assert!(vault.dirty);
}
