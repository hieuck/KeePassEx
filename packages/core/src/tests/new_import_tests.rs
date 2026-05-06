//! Tests for new importers: NordPass, Enpass, Dashlane, RoboForm, KeePass 1.x
//! These importers return ImportResult (ImportEntry/ImportGroup) — not canonical Entry/Group.

use crate::import_export::dashlane::import_dashlane;
use crate::import_export::enpass::import_enpass;
use crate::import_export::keepass1::import_keepass1_xml;
use crate::import_export::nordpass::import_nordpass;
use crate::import_export::roboform::import_roboform;

// ─── NordPass ─────────────────────────────────────────────────────────────────

const NORDPASS_CSV: &str = "name,url,username,password,note,cardholder,cardnumber,cvc,expirydate,zipcode,folder,full_name,phone_number,email,address1,address2,city,country,state\nGitHub,https://github.com,user@example.com,secret123,Work account,,,,,,Work,,,,,,,,\nGmail,https://mail.google.com,user@gmail.com,gmailpass,,,,,,,Personal,,,,,,,,\nMy Visa,,,,,,4111111111111111,123,12/2027,,Finance,,,,,,,,\n";

#[test]
fn test_nordpass_import_credentials() {
    let result = import_nordpass(NORDPASS_CSV).unwrap();
    let logins: Vec<_> = result.entries.iter().filter(|e| e.has_password).collect();
    assert_eq!(logins.len(), 2);
}

#[test]
fn test_nordpass_import_card() {
    let result = import_nordpass(NORDPASS_CSV).unwrap();
    let cards: Vec<_> = result.entries.iter().filter(|e| e.icon_id == 9).collect();
    assert_eq!(cards.len(), 1);
    let card = &cards[0];
    // ImportEntry.custom_fields is Vec<ImportCustomField>
    assert!(card
        .custom_fields
        .iter()
        .any(|f| f.key == "Card Number" && f.protected));
    assert!(card
        .custom_fields
        .iter()
        .any(|f| f.key == "CVC" && f.protected));
}

#[test]
fn test_nordpass_import_folders() {
    let result = import_nordpass(NORDPASS_CSV).unwrap();
    let names: Vec<&str> = result.groups.iter().map(|g| g.name.as_str()).collect();
    assert!(names.contains(&"Work"));
    assert!(names.contains(&"Personal"));
    assert!(names.contains(&"Finance"));
}

#[test]
fn test_nordpass_import_notes() {
    let result = import_nordpass(NORDPASS_CSV).unwrap();
    let github = result.entries.iter().find(|e| e.title == "GitHub").unwrap();
    assert_eq!(github.notes, "Work account");
}

#[test]
fn test_nordpass_empty_csv() {
    let csv = "name,url,username,password,note,cardholder,cardnumber,cvc,expirydate,zipcode,folder,full_name,phone_number,email,address1,address2,city,country,state\n";
    let result = import_nordpass(csv).unwrap();
    assert_eq!(result.entries.len(), 0);
}

// ─── Enpass ───────────────────────────────────────────────────────────────────

const ENPASS_JSON: &str = r#"{
    "folders": [
        {"uuid": "f1", "title": "Work"},
        {"uuid": "f2", "title": "Finance"}
    ],
    "items": [
        {
            "uuid": "i1",
            "title": "GitHub",
            "category": "login",
            "folders": ["f1"],
            "fields": [
                {"label": "Username", "value": "user@example.com", "type": "username", "sensitive": false},
                {"label": "Password", "value": "secret123", "type": "password", "sensitive": true},
                {"label": "URL", "value": "https://github.com", "type": "url", "sensitive": false}
            ]
        },
        {
            "uuid": "i2",
            "title": "My Visa",
            "category": "creditcard",
            "folders": ["f2"],
            "fields": [
                {"label": "Card Number", "value": "4111111111111111", "type": "ccnumber", "sensitive": true},
                {"label": "CVV", "value": "123", "type": "cvc", "sensitive": true}
            ]
        },
        {
            "uuid": "i3",
            "title": "Archived",
            "category": "login",
            "archived": true,
            "fields": []
        }
    ]
}"#;

#[test]
fn test_enpass_import_credentials() {
    let result = import_enpass(ENPASS_JSON).unwrap();
    assert_eq!(result.entries.len(), 2); // archived excluded
}

#[test]
fn test_enpass_import_folders() {
    let result = import_enpass(ENPASS_JSON).unwrap();
    let names: Vec<&str> = result.groups.iter().map(|g| g.name.as_str()).collect();
    assert!(names.contains(&"Work"));
    assert!(names.contains(&"Finance"));
}

#[test]
fn test_enpass_import_login_fields() {
    let result = import_enpass(ENPASS_JSON).unwrap();
    let github = result.entries.iter().find(|e| e.title == "GitHub").unwrap();
    assert_eq!(github.username, "user@example.com");
    assert!(github.has_password);
    assert_eq!(github.url, "https://github.com");
}

#[test]
fn test_enpass_import_credit_card() {
    let result = import_enpass(ENPASS_JSON).unwrap();
    let card = result
        .entries
        .iter()
        .find(|e| e.title == "My Visa")
        .unwrap();
    assert_eq!(card.icon_id, 9);
    assert!(card
        .custom_fields
        .iter()
        .any(|f| f.key == "Card Number" && f.protected));
}

#[test]
fn test_enpass_archived_excluded() {
    let result = import_enpass(ENPASS_JSON).unwrap();
    assert!(!result.entries.iter().any(|e| e.title == "Archived"));
}

#[test]
fn test_enpass_invalid_json() {
    assert!(import_enpass("not json").is_err());
}

#[test]
fn test_enpass_empty_export() {
    let result = import_enpass("{}").unwrap();
    assert_eq!(result.entries.len(), 0);
}

// ─── Dashlane ─────────────────────────────────────────────────────────────────

const DASHLANE_JSON: &str = r#"{
    "credentials": [
        {
            "title": "GitHub",
            "username": "user@example.com",
            "password": "secret123",
            "url": "https://github.com",
            "note": "Work account",
            "category": "Work"
        },
        {
            "title": "Gmail",
            "username": "user@gmail.com",
            "password": "gmailpass",
            "url": "https://mail.google.com",
            "category": "Personal",
            "otpSecret": "JBSWY3DPEHPK3PXP"
        }
    ],
    "securenotes": [
        {
            "title": "WiFi Password",
            "content": "HomeNetwork: password123"
        }
    ]
}"#;

#[test]
fn test_dashlane_import_credentials() {
    let result = import_dashlane(DASHLANE_JSON).unwrap();
    let logins: Vec<_> = result.entries.iter().filter(|e| e.has_password).collect();
    assert_eq!(logins.len(), 2);
}

#[test]
fn test_dashlane_import_secure_notes() {
    let result = import_dashlane(DASHLANE_JSON).unwrap();
    let notes: Vec<_> = result.entries.iter().filter(|e| e.icon_id == 22).collect();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "WiFi Password");
}

#[test]
fn test_dashlane_import_otp() {
    let result = import_dashlane(DASHLANE_JSON).unwrap();
    let gmail = result.entries.iter().find(|e| e.title == "Gmail").unwrap();
    assert!(gmail.has_otp);
    assert!(gmail
        .custom_fields
        .iter()
        .any(|f| f.key == "otp" && f.value.starts_with("otpauth://")));
}

#[test]
fn test_dashlane_import_groups() {
    let result = import_dashlane(DASHLANE_JSON).unwrap();
    let names: Vec<&str> = result.groups.iter().map(|g| g.name.as_str()).collect();
    assert!(names.contains(&"Work"));
    assert!(names.contains(&"Personal"));
}

#[test]
fn test_dashlane_invalid_json() {
    assert!(import_dashlane("not json").is_err());
}

#[test]
fn test_dashlane_empty_export() {
    let result = import_dashlane("{}").unwrap();
    assert_eq!(result.entries.len(), 0);
}

// ─── RoboForm ─────────────────────────────────────────────────────────────────

const ROBOFORM_HTML: &str = r#"<!DOCTYPE html>
<html>
<body>
<table>
<tr><th>Name</th><th>URL</th><th>Login</th><th>Password</th><th>Note</th><th>Folder</th></tr>
<tr><td>GitHub</td><td>https://github.com</td><td>user@example.com</td><td>secret123</td><td>Work account</td><td>Work</td></tr>
<tr><td>Gmail</td><td>https://mail.google.com</td><td>user@gmail.com</td><td>gmailpass</td><td></td><td>Personal</td></tr>
<tr><td>Amazon</td><td>https://amazon.com</td><td>shopper</td><td>shoppass</td><td></td><td>Shopping</td></tr>
</table>
</body>
</html>"#;

#[test]
fn test_roboform_import_entries() {
    let result = import_roboform(ROBOFORM_HTML).unwrap();
    assert_eq!(result.entries.len(), 3);
}

#[test]
fn test_roboform_import_fields() {
    let result = import_roboform(ROBOFORM_HTML).unwrap();
    let github = result.entries.iter().find(|e| e.title == "GitHub").unwrap();
    assert_eq!(github.username, "user@example.com");
    assert!(github.has_password);
    assert_eq!(github.url, "https://github.com");
    assert_eq!(github.notes, "Work account");
}

#[test]
fn test_roboform_import_folders() {
    let result = import_roboform(ROBOFORM_HTML).unwrap();
    let names: Vec<&str> = result.groups.iter().map(|g| g.name.as_str()).collect();
    assert!(names.contains(&"Work"));
    assert!(names.contains(&"Personal"));
    assert!(names.contains(&"Shopping"));
}

#[test]
fn test_roboform_empty_html() {
    let result = import_roboform("<html><body></body></html>").unwrap();
    assert_eq!(result.entries.len(), 0);
}

#[test]
fn test_roboform_html_entities() {
    let html = r#"<table>
<tr><th>Name</th><th>URL</th><th>Login</th><th>Password</th><th>Note</th><th>Folder</th></tr>
<tr><td>AT&amp;T</td><td>https://att.com</td><td>user</td><td>pass&amp;word</td><td></td><td></td></tr>
</table>"#;
    let result = import_roboform(html).unwrap();
    assert_eq!(result.entries.len(), 1);
    assert_eq!(result.entries[0].title, "AT&T");
}

// ─── KeePass 1.x ──────────────────────────────────────────────────────────────

const KEEPASS1_XML: &str = r#"<?xml version="1.0"?><pwlist>
<pwentry><group>Internet</group><title>GitHub</title><username>user@example.com</username><url>https://github.com</url><password>secret123</password><notes>Work account</notes><expire>Never</expire></pwentry>
<pwentry><group>Email</group><title>Gmail</title><username>user@gmail.com</username><url>https://mail.google.com</url><password>gmailpass</password><notes></notes><expire>Never</expire></pwentry>
<pwentry><group>Internet</group><title>Amazon</title><username>shopper</username><url>https://amazon.com</url><password>shoppass</password><notes></notes><expire>Never</expire></pwentry>
</pwlist>"#;

#[test]
fn test_keepass1_import_entries() {
    let result = import_keepass1_xml(KEEPASS1_XML).unwrap();
    assert_eq!(result.entries.len(), 3);
}

#[test]
fn test_keepass1_import_fields() {
    let result = import_keepass1_xml(KEEPASS1_XML).unwrap();
    let github = result.entries.iter().find(|e| e.title == "GitHub").unwrap();
    assert_eq!(github.username, "user@example.com");
    assert!(github.has_password);
    assert_eq!(github.url, "https://github.com");
    assert_eq!(github.notes, "Work account");
}

#[test]
fn test_keepass1_group_deduplication() {
    let result = import_keepass1_xml(KEEPASS1_XML).unwrap();
    assert_eq!(
        result
            .groups
            .iter()
            .filter(|g| g.name == "Internet")
            .count(),
        1
    );
}

#[test]
fn test_keepass1_empty_xml() {
    let result = import_keepass1_xml("<pwlist></pwlist>").unwrap();
    assert_eq!(result.entries.len(), 0);
}

#[test]
fn test_keepass1_never_expiry() {
    let result = import_keepass1_xml(KEEPASS1_XML).unwrap();
    let github = result.entries.iter().find(|e| e.title == "GitHub").unwrap();
    assert!(github.expiry.is_none());
    assert!(!github.is_expired);
}

#[test]
fn test_keepass1_xml_entities() {
    let xml = r#"<pwlist><pwentry><group>Test</group><title>AT&amp;T</title><username>u</username><url>https://att.com</url><password>p</password><notes></notes><expire>Never</expire></pwentry></pwlist>"#;
    let result = import_keepass1_xml(xml).unwrap();
    assert_eq!(result.entries[0].title, "AT&T");
}
