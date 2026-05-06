//! NordPass CSV import

use crate::error::{KeePassExError, Result};
use crate::import_export::import_types::{
    ImportCustomField, ImportEntry, ImportGroup, ImportResult,
};
use uuid::Uuid;

/// Import a NordPass CSV export.
pub fn import_nordpass(csv_data: &str) -> Result<ImportResult> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_data.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| KeePassExError::ImportParseFailed(format!("Invalid NordPass CSV: {e}")))?
        .clone();

    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut groups: Vec<ImportGroup> = Vec::new();
    let mut warnings = Vec::new();
    let mut folder_groups: std::collections::HashMap<String, String> = Default::default();
    let now = chrono::Utc::now().to_rfc3339();
    let root_uuid = Uuid::new_v4().to_string();

    let col =
        |name: &str| -> Option<usize> { headers.iter().position(|h| h.eq_ignore_ascii_case(name)) };

    let idx_name = col("name");
    let idx_url = col("url");
    let idx_username = col("username");
    let idx_password = col("password");
    let idx_note = col("note");
    let idx_folder = col("folder");
    let idx_cardholder = col("cardholder");
    let idx_cardnumber = col("cardnumber");
    let idx_cvc = col("cvc");
    let idx_expiry = col("expirydate");

    for (row_idx, result) in reader.records().enumerate() {
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                warnings.push(format!("Row {}: {e}", row_idx + 2));
                continue;
            }
        };

        let get =
            |idx: Option<usize>| -> &str { idx.and_then(|i| record.get(i)).unwrap_or("").trim() };

        let title = get(idx_name);
        let url = get(idx_url);
        let username = get(idx_username);
        let password = get(idx_password);
        let note = get(idx_note);
        let folder = get(idx_folder);

        if title.is_empty() && username.is_empty() && url.is_empty() {
            continue;
        }

        let group_uuid = if !folder.is_empty() {
            folder_groups
                .entry(folder.to_string())
                .or_insert_with(|| {
                    let uuid = Uuid::new_v4().to_string();
                    groups.push(ImportGroup {
                        uuid: uuid.clone(),
                        parent_uuid: Some(root_uuid.clone()),
                        name: folder.to_string(),
                        icon_id: 48,
                        is_expanded: true,
                        ..Default::default()
                    });
                    uuid
                })
                .clone()
        } else {
            root_uuid.clone()
        };

        let cardholder = get(idx_cardholder);
        let cardnumber = get(idx_cardnumber);
        let cvc = get(idx_cvc);
        let expiry = get(idx_expiry);

        let mut custom_fields = Vec::new();
        if !cardnumber.is_empty() {
            custom_fields.push(ImportCustomField {
                key: "Card Number".into(),
                value: cardnumber.into(),
                protected: true,
            });
        }
        if !cardholder.is_empty() {
            custom_fields.push(ImportCustomField {
                key: "Cardholder".into(),
                value: cardholder.into(),
                protected: false,
            });
        }
        if !cvc.is_empty() {
            custom_fields.push(ImportCustomField {
                key: "CVC".into(),
                value: cvc.into(),
                protected: true,
            });
        }
        if !expiry.is_empty() {
            custom_fields.push(ImportCustomField {
                key: "Expiry Date".into(),
                value: expiry.into(),
                protected: false,
            });
        }

        let is_card = !cardnumber.is_empty();
        entries.push(ImportEntry {
            uuid: Uuid::new_v4().to_string(),
            group_uuid,
            title: if title.is_empty() {
                "Untitled".into()
            } else {
                title.into()
            },
            username: username.into(),
            password: if password.is_empty() {
                None
            } else {
                Some(password.into())
            },
            url: url.into(),
            notes: note.into(),
            icon_id: if is_card { 9 } else { 1 },
            has_password: !password.is_empty(),
            custom_fields,
            created_at: now.clone(),
            modified_at: now.clone(),
            auto_type_enabled: !is_card,
            quality_check: !is_card,
            ..Default::default()
        });
    }

    Ok(ImportResult {
        entries,
        groups,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "name,url,username,password,note,cardholder,cardnumber,cvc,expirydate,zipcode,folder,full_name,phone_number,email,address1,address2,city,country,state\nGitHub,https://github.com,user@example.com,secret123,,,,,,,Work,,,,,,,,,\nGmail,https://mail.google.com,user@gmail.com,gmailpass,,,,,,,Personal,,,,,,,,,\nMy Visa,,,,,,4111111111111111,123,12/2027,,Finance,,,,,,,,,\n";

    #[test]
    fn test_import_credentials() {
        let r = import_nordpass(SAMPLE).unwrap();
        assert_eq!(r.entries.iter().filter(|e| e.has_password).count(), 2);
    }

    #[test]
    fn test_import_card() {
        let r = import_nordpass(SAMPLE).unwrap();
        let cards: Vec<_> = r.entries.iter().filter(|e| e.icon_id == 9).collect();
        assert_eq!(cards.len(), 1);
        assert!(cards[0]
            .custom_fields
            .iter()
            .any(|f| f.key == "Card Number" && f.protected));
        assert!(cards[0]
            .custom_fields
            .iter()
            .any(|f| f.key == "CVC" && f.protected));
    }

    #[test]
    fn test_import_folders() {
        let r = import_nordpass(SAMPLE).unwrap();
        let names: Vec<&str> = r.groups.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"Work"));
        assert!(names.contains(&"Personal"));
        assert!(names.contains(&"Finance"));
    }

    #[test]
    fn test_import_empty() {
        let r = import_nordpass("name,url,username,password,note,cardholder,cardnumber,cvc,expirydate,zipcode,folder,full_name,phone_number,email,address1,address2,city,country,state\n").unwrap();
        assert_eq!(r.entries.len(), 0);
    }

    #[test]
    fn test_import_notes() {
        let csv = "name,url,username,password,note,cardholder,cardnumber,cvc,expirydate,zipcode,folder,full_name,phone_number,email,address1,address2,city,country,state\nGitHub,https://github.com,user@example.com,secret123,Work account,,,,,,,,,,,,,,,\n";
        let r = import_nordpass(csv).unwrap();
        let gh = r.entries.iter().find(|e| e.title == "GitHub").unwrap();
        assert_eq!(gh.notes, "Work account");
    }
}
