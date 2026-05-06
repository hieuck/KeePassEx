//! Import/Export tests

use crate::vault::Vault;
use crate::import_export::{import_into_vault, export_vault, ImportFormat, ExportFormat, detect_format};

#[test]
fn test_detect_bitwarden_format() {
    let data = br#"{"encrypted":false,"items":[{"id":"1","name":"Test","type":1,"login":{"username":"user","password":"pass","uris":[{"uri":"https://example.com"}]}}]}"#;
    let format = detect_format(data);
    assert_eq!(format, Some(ImportFormat::BitwardenJson));
}

#[test]
fn test_detect_chrome_csv() {
    let data = b"name,url,username,password\nGitHub,https://github.com,user,pass";
    let format = detect_format(data);
    assert_eq!(format, Some(ImportFormat::ChromeCsv));
}

#[test]
fn test_detect_lastpass_csv() {
    let data = b"url,username,password,totp,extra,name,grouping,fav\nhttps://example.com,user,pass,,,Example,,0";
    let format = detect_format(data);
    assert_eq!(format, Some(ImportFormat::LastPassCsv));
}

#[test]
fn test_import_bitwarden_json() {
    let data = br#"{
        "encrypted": false,
        "folders": [
            {"id": "folder1", "name": "Work"}
        ],
        "items": [
            {
                "id": "item1",
                "name": "GitHub",
                "type": 1,
                "folderId": "folder1",
                "login": {
                    "username": "user@example.com",
                    "password": "SecureP@ss123!",
                    "uris": [{"uri": "https://github.com"}]
                },
                "notes": "My GitHub account"
            },
            {
                "id": "item2",
                "name": "Note",
                "type": 2,
                "login": null
            }
        ]
    }"#;

    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let result = import_into_vault(&mut vault, data, ImportFormat::BitwardenJson, Some(root)).unwrap();

    assert_eq!(result.entries_imported, 1); // Only login items
    assert_eq!(result.entries_skipped, 1); // Note skipped
    assert_eq!(result.groups_created, 1); // "Work" folder

    // Verify entry data
    let entries: Vec<_> = vault.all_entries().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title.get(), "GitHub");
    assert_eq!(entries[0].username.get(), "user@example.com");
    assert_eq!(entries[0].url, "https://github.com");
    assert_eq!(entries[0].notes.get(), "My GitHub account");
}

#[test]
fn test_import_chrome_csv() {
    let data = b"name,url,username,password\nGitHub,https://github.com,user@example.com,SecurePass123!\nGmail,https://gmail.com,user@gmail.com,GmailPass456!";

    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let result = import_into_vault(&mut vault, data, ImportFormat::ChromeCsv, Some(root)).unwrap();

    assert_eq!(result.entries_imported, 2);
    assert_eq!(vault.entry_count(), 2);
}

#[test]
fn test_import_generic_csv() {
    let data = b"Title,Username,Password,URL,Notes\nGitHub,user,pass,https://github.com,My account\nGmail,user2,pass2,https://gmail.com,";

    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;
    let result = import_into_vault(&mut vault, data, ImportFormat::GenericCsv, Some(root)).unwrap();

    assert_eq!(result.entries_imported, 2);
}

#[test]
fn test_export_csv() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    let entry = vault.get_entry_mut(&uuid).unwrap();
    entry.title.set("GitHub");
    entry.username.set("user@example.com");
    entry.password.set("SecurePass123!");
    entry.url = "https://github.com".to_string();

    let csv = export_vault(&vault, ExportFormat::CsvUnencrypted).unwrap();
    let csv_str = String::from_utf8(csv).unwrap();

    assert!(csv_str.contains("GitHub"));
    assert!(csv_str.contains("user@example.com"));
    assert!(csv_str.contains("SecurePass123!"));
    assert!(csv_str.contains("https://github.com"));
}

#[test]
fn test_export_json() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    let entry = vault.get_entry_mut(&uuid).unwrap();
    entry.title.set("Test Entry");
    entry.username.set("testuser");

    let json_bytes = export_vault(&vault, ExportFormat::JsonUnencrypted).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&json_bytes).unwrap();

    assert_eq!(json["generator"], "KeePassEx");
    assert!(json["entries"].is_array());
    assert_eq!(json["entries"][0]["title"], "Test Entry");
}

#[test]
fn test_csv_escape_special_chars() {
    let mut vault = Vault::new("Test");
    let root = vault.root_group_uuid;

    let uuid = vault.create_entry(root).unwrap();
    let entry = vault.get_entry_mut(&uuid).unwrap();
    entry.title.set("Entry with, comma");
    entry.notes.set("Notes with \"quotes\"");

    let csv = export_vault(&vault, ExportFormat::CsvUnencrypted).unwrap();
    let csv_str = String::from_utf8(csv).unwrap();

    // Should be properly escaped
    assert!(csv_str.contains("\"Entry with, comma\""));
}
