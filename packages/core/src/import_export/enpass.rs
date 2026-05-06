//! Enpass JSON import

use crate::error::{KeePassExError, Result};
use crate::import_export::import_types::{
    ImportCustomField, ImportEntry, ImportGroup, ImportResult,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct EnpassExport {
    folders: Option<Vec<EnpassFolder>>,
    items: Option<Vec<EnpassItem>>,
}

#[derive(Debug, Deserialize)]
struct EnpassFolder {
    uuid: String,
    title: String,
}

#[derive(Debug, Deserialize)]
struct EnpassItem {
    uuid: Option<String>,
    title: String,
    category: Option<String>,
    note: Option<String>,
    folders: Option<Vec<String>>,
    fields: Option<Vec<EnpassField>>,
    archived: Option<bool>,
    trashed: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct EnpassField {
    label: String,
    value: String,
    #[serde(rename = "type")]
    field_type: Option<String>,
    sensitive: Option<bool>,
}

/// Import an Enpass JSON export.
pub fn import_enpass(json_data: &str) -> Result<ImportResult> {
    let export: EnpassExport = serde_json::from_str(json_data)
        .map_err(|e| KeePassExError::ImportParseFailed(format!("Invalid Enpass JSON: {e}")))?;

    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut groups: Vec<ImportGroup> = Vec::new();
    let mut warnings = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();
    let root_uuid = Uuid::new_v4().to_string();
    let mut folder_map: std::collections::HashMap<String, String> = Default::default();

    if let Some(folders) = export.folders {
        for folder in &folders {
            let group_uuid = Uuid::new_v4().to_string();
            folder_map.insert(folder.uuid.clone(), group_uuid.clone());
            groups.push(ImportGroup {
                uuid: group_uuid,
                parent_uuid: Some(root_uuid.clone()),
                name: folder.title.clone(),
                icon_id: 48,
                is_expanded: true,
                ..Default::default()
            });
        }
    }

    if let Some(items) = export.items {
        for item in items {
            if item.archived.unwrap_or(false) || item.trashed.unwrap_or(false) {
                continue;
            }

            let group_uuid = item
                .folders
                .as_ref()
                .and_then(|f| f.first())
                .and_then(|fid| folder_map.get(fid))
                .cloned()
                .unwrap_or_else(|| root_uuid.clone());

            let mut username = String::new();
            let mut password: Option<String> = None;
            let mut url = String::new();
            let mut otp_uri: Option<String> = None;
            let mut custom_fields: Vec<ImportCustomField> = Vec::new();

            if let Some(fields) = item.fields {
                for field in fields {
                    if field.value.is_empty() {
                        continue;
                    }
                    let ll = field.label.to_lowercase();
                    let sensitive = field.sensitive.unwrap_or(false);
                    match ll.as_str() {
                        "username" | "email" | "login" | "user name" => {
                            if username.is_empty() {
                                username = field.value.clone();
                            }
                        }
                        "password" | "passwd" => {
                            if password.is_none() {
                                password = Some(field.value.clone());
                            }
                        }
                        "url" | "website" | "web site" | "login url" => {
                            if url.is_empty() {
                                url = field.value.clone();
                            }
                        }
                        "totp" | "one-time password" | "otp" => {
                            otp_uri = Some(if field.value.starts_with("otpauth://") {
                                field.value.clone()
                            } else {
                                format!("otpauth://totp/{}?secret={}", item.title, field.value)
                            });
                        }
                        _ => {
                            custom_fields.push(ImportCustomField {
                                key: field.label.clone(),
                                value: field.value.clone(),
                                protected: sensitive,
                            });
                        }
                    }
                }
            }

            let has_otp = if let Some(otp) = otp_uri {
                custom_fields.push(ImportCustomField {
                    key: "otp".into(),
                    value: otp,
                    protected: true,
                });
                true
            } else {
                false
            };

            let icon_id = match item.category.as_deref().unwrap_or("") {
                "creditcard" | "finance" => 9,
                "note" => 22,
                "identity" => 2,
                "computer" | "server" => 5,
                "email" => 8,
                _ => 1,
            };

            entries.push(ImportEntry {
                uuid: item.uuid.unwrap_or_else(|| Uuid::new_v4().to_string()),
                group_uuid,
                title: item.title.clone(),
                username,
                password: password.clone(),
                url,
                notes: item.note.unwrap_or_default(),
                icon_id,
                has_password: password.is_some(),
                has_otp,
                custom_fields,
                created_at: now.clone(),
                modified_at: now.clone(),
                auto_type_enabled: true,
                quality_check: true,
                ..Default::default()
            });
        }
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

    const JSON: &str = r#"{"folders":[{"uuid":"f1","title":"Work"},{"uuid":"f2","title":"Finance"}],"items":[{"uuid":"i1","title":"GitHub","category":"login","folders":["f1"],"fields":[{"label":"Username","value":"user@example.com","type":"username","sensitive":false},{"label":"Password","value":"secret123","type":"password","sensitive":true},{"label":"URL","value":"https://github.com","type":"url","sensitive":false}]},{"uuid":"i2","title":"My Visa","category":"creditcard","folders":["f2"],"fields":[{"label":"Card Number","value":"4111111111111111","type":"ccnumber","sensitive":true},{"label":"CVV","value":"123","type":"cvc","sensitive":true}]},{"uuid":"i3","title":"Archived","category":"login","archived":true,"fields":[]}]}"#;

    #[test]
    fn test_import_credentials() {
        assert_eq!(import_enpass(JSON).unwrap().entries.len(), 2);
    }

    #[test]
    fn test_import_folders() {
        let r = import_enpass(JSON).unwrap();
        let names: Vec<&str> = r.groups.iter().map(|g| g.name.as_str()).collect();
        assert!(names.contains(&"Work") && names.contains(&"Finance"));
    }

    #[test]
    fn test_import_login_fields() {
        let r = import_enpass(JSON).unwrap();
        let gh = r.entries.iter().find(|e| e.title == "GitHub").unwrap();
        assert_eq!(gh.username, "user@example.com");
        assert!(gh.has_password);
        assert_eq!(gh.url, "https://github.com");
    }

    #[test]
    fn test_import_credit_card() {
        let r = import_enpass(JSON).unwrap();
        let card = r.entries.iter().find(|e| e.title == "My Visa").unwrap();
        assert_eq!(card.icon_id, 9);
        assert!(card
            .custom_fields
            .iter()
            .any(|f| f.key == "Card Number" && f.protected));
    }

    #[test]
    fn test_archived_excluded() {
        assert!(!import_enpass(JSON)
            .unwrap()
            .entries
            .iter()
            .any(|e| e.title == "Archived"));
    }

    #[test]
    fn test_invalid_json() {
        assert!(import_enpass("not json").is_err());
    }

    #[test]
    fn test_empty_export() {
        assert_eq!(import_enpass("{}").unwrap().entries.len(), 0);
    }
}
