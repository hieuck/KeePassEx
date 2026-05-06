//! Dashlane JSON import
//!
//! Supports Dashlane's JSON export format (exported from Dashlane app).
//! Dashlane exports a JSON file with categories: credentials, securenotes,
//! payments, personalinfo, ids.

use crate::error::{KeePassExError, Result};
use crate::types::{Entry, Group};
use serde::Deserialize;
use uuid::Uuid;

// ─── Dashlane JSON structures ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DashlaneExport {
    credentials: Option<Vec<DashlaneCredential>>,
    #[serde(rename = "securenotes")]
    secure_notes: Option<Vec<DashlaneSecureNote>>,
    payments: Option<Vec<DashlanePayment>>,
    #[serde(rename = "personalinfo")]
    personal_info: Option<Vec<DashlanePersonalInfo>>,
    ids: Option<Vec<DashlaneId>>,
}

#[derive(Debug, Deserialize)]
struct DashlaneCredential {
    title: Option<String>,
    username: Option<String>,
    password: Option<String>,
    url: Option<String>,
    note: Option<String>,
    category: Option<String>,
    #[serde(rename = "otpSecret")]
    otp_secret: Option<String>,
    #[serde(rename = "secondaryLogin")]
    secondary_login: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashlaneSecureNote {
    title: Option<String>,
    content: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashlanePayment {
    name: Option<String>,
    #[serde(rename = "cardNumber")]
    card_number: Option<String>,
    #[serde(rename = "cardName")]
    card_name: Option<String>,
    #[serde(rename = "expireMonth")]
    expire_month: Option<String>,
    #[serde(rename = "expireYear")]
    expire_year: Option<String>,
    #[serde(rename = "securityCode")]
    security_code: Option<String>,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashlanePersonalInfo {
    title: Option<String>,
    #[serde(rename = "firstName")]
    first_name: Option<String>,
    #[serde(rename = "lastName")]
    last_name: Option<String>,
    email: Option<String>,
    phone: Option<String>,
    address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DashlaneId {
    #[serde(rename = "type")]
    id_type: Option<String>,
    number: Option<String>,
    name: Option<String>,
    #[serde(rename = "issueDate")]
    issue_date: Option<String>,
    #[serde(rename = "expirationDate")]
    expiration_date: Option<String>,
    country: Option<String>,
}

// ─── Import result ────────────────────────────────────────────────────────────

pub use crate::import_export::import_types::ImportResult as DashlaneImportResult;

// ─── Importer ─────────────────────────────────────────────────────────────────

/// Import a Dashlane JSON export file.
pub fn import_dashlane(json_data: &str) -> Result<DashlaneImportResult> {
    use crate::import_export::import_types::{ImportCustomField, ImportEntry, ImportGroup};

    let export: DashlaneExport = serde_json::from_str(json_data)
        .map_err(|e| KeePassExError::ImportParseFailed(format!("Invalid Dashlane JSON: {e}")))?;

    let mut entries: Vec<ImportEntry> = Vec::new();
    let mut groups: Vec<ImportGroup> = Vec::new();
    let mut warnings = Vec::new();

    let root_uuid = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    // ── Credentials ──────────────────────────────────────────────────────────
    let mut category_groups: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    if let Some(credentials) = export.credentials {
        for cred in credentials {
            let title = cred.title.unwrap_or_else(|| {
                cred.url
                    .as_deref()
                    .and_then(|u| {
                        u.split("://")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .map(|h| h.trim_start_matches("www.").to_string())
                    })
                    .unwrap_or_else(|| "Untitled".to_string())
            });

            let category = cred.category.unwrap_or_else(|| "Credentials".to_string());
            let group_uuid = category_groups
                .entry(category.clone())
                .or_insert_with(|| {
                    let uuid = Uuid::new_v4().to_string();
                    groups.push(ImportGroup {
                        uuid: uuid.clone(),
                        parent_uuid: Some(root_uuid.clone()),
                        name: category.clone(),
                        icon_id: 1,
                        is_expanded: true,
                        ..Default::default()
                    });
                    uuid
                })
                .clone();

            let mut custom_fields = Vec::new();

            if let Some(secondary) = cred.secondary_login {
                if !secondary.is_empty() {
                    custom_fields.push(ImportCustomField {
                        key: "Secondary Login".to_string(),
                        value: secondary,
                        protected: false,
                    });
                }
            }

            let has_otp = if let Some(otp) = cred.otp_secret {
                if !otp.is_empty() {
                    custom_fields.push(ImportCustomField {
                        key: "otp".to_string(),
                        value: format!("otpauth://totp/{}?secret={}", title, otp),
                        protected: true,
                    });
                    true
                } else {
                    false
                }
            } else {
                false
            };

            let has_password = cred
                .password
                .as_deref()
                .map(|p| !p.is_empty())
                .unwrap_or(false);

            entries.push(ImportEntry {
                uuid: Uuid::new_v4().to_string(),
                group_uuid,
                title,
                username: cred.username.unwrap_or_default(),
                password: cred.password,
                url: cred.url.unwrap_or_default(),
                notes: cred.note.unwrap_or_default(),
                icon_id: 1,
                has_password,
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

    // ── Secure Notes ─────────────────────────────────────────────────────────
    if let Some(notes) = export.secure_notes {
        if !notes.is_empty() {
            let notes_group_uuid = Uuid::new_v4().to_string();
            groups.push(ImportGroup {
                uuid: notes_group_uuid.clone(),
                parent_uuid: Some(root_uuid.clone()),
                name: "Secure Notes".to_string(),
                icon_id: 22,
                is_expanded: true,
                ..Default::default()
            });
            for note in notes {
                entries.push(ImportEntry {
                    uuid: Uuid::new_v4().to_string(),
                    group_uuid: notes_group_uuid.clone(),
                    title: note.title.unwrap_or_else(|| "Secure Note".to_string()),
                    notes: note.content.unwrap_or_default(),
                    icon_id: 22,
                    created_at: now.clone(),
                    modified_at: now.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // ── Payments ─────────────────────────────────────────────────────────────
    if let Some(payments) = export.payments {
        if !payments.is_empty() {
            let payments_group_uuid = Uuid::new_v4().to_string();
            groups.push(ImportGroup {
                uuid: payments_group_uuid.clone(),
                parent_uuid: Some(root_uuid.clone()),
                name: "Payment Cards".to_string(),
                icon_id: 9,
                is_expanded: true,
                ..Default::default()
            });
            for payment in payments {
                let mut custom_fields = Vec::new();
                if let Some(cn) = payment.card_number {
                    custom_fields.push(ImportCustomField {
                        key: "Card Number".into(),
                        value: cn,
                        protected: true,
                    });
                }
                if let Some(holder) = payment.card_name {
                    custom_fields.push(ImportCustomField {
                        key: "Cardholder".into(),
                        value: holder,
                        protected: false,
                    });
                }
                if let Some(cvv) = payment.security_code {
                    custom_fields.push(ImportCustomField {
                        key: "CVV".into(),
                        value: cvv,
                        protected: true,
                    });
                }
                let expiry_str = match (payment.expire_month, payment.expire_year) {
                    (Some(m), Some(y)) => Some(format!("{m}/{y}")),
                    _ => None,
                };
                if let Some(exp) = expiry_str {
                    custom_fields.push(ImportCustomField {
                        key: "Expiry".into(),
                        value: exp,
                        protected: false,
                    });
                }
                entries.push(ImportEntry {
                    uuid: Uuid::new_v4().to_string(),
                    group_uuid: payments_group_uuid.clone(),
                    title: payment.name.unwrap_or_else(|| "Payment Card".to_string()),
                    notes: payment.note.unwrap_or_default(),
                    icon_id: 9,
                    custom_fields,
                    created_at: now.clone(),
                    modified_at: now.clone(),
                    ..Default::default()
                });
            }
        }
    }

    // ── IDs ──────────────────────────────────────────────────────────────────
    if let Some(ids) = export.ids {
        if !ids.is_empty() {
            let ids_group_uuid = Uuid::new_v4().to_string();
            groups.push(ImportGroup {
                uuid: ids_group_uuid.clone(),
                parent_uuid: Some(root_uuid.clone()),
                name: "IDs & Documents".to_string(),
                icon_id: 2,
                is_expanded: true,
                ..Default::default()
            });
            for id in ids {
                let mut custom_fields = Vec::new();
                if let Some(num) = id.number {
                    custom_fields.push(ImportCustomField {
                        key: "Number".into(),
                        value: num,
                        protected: true,
                    });
                }
                if let Some(country) = id.country {
                    custom_fields.push(ImportCustomField {
                        key: "Country".into(),
                        value: country,
                        protected: false,
                    });
                }
                if let Some(issue) = id.issue_date {
                    custom_fields.push(ImportCustomField {
                        key: "Issue Date".into(),
                        value: issue,
                        protected: false,
                    });
                }
                entries.push(ImportEntry {
                    uuid: Uuid::new_v4().to_string(),
                    group_uuid: ids_group_uuid.clone(),
                    title: id
                        .name
                        .or(id.id_type)
                        .unwrap_or_else(|| "ID Document".to_string()),
                    icon_id: 2,
                    custom_fields,
                    expiry: id.expiration_date,
                    created_at: now.clone(),
                    modified_at: now.clone(),
                    ..Default::default()
                });
            }
        }
    }

    Ok(DashlaneImportResult {
        entries,
        groups,
        warnings,
    })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"{
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
                "category": "Personal"
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
    fn test_import_credentials() {
        let result = import_dashlane(SAMPLE_JSON).unwrap();
        assert_eq!(result.entries.iter().filter(|e| e.icon_id == 1).count(), 2);
    }

    #[test]
    fn test_import_secure_notes() {
        let result = import_dashlane(SAMPLE_JSON).unwrap();
        let notes: Vec<_> = result.entries.iter().filter(|e| e.icon_id == 22).collect();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].title, "WiFi Password");
    }

    #[test]
    fn test_import_groups_created() {
        let result = import_dashlane(SAMPLE_JSON).unwrap();
        // Should have Work, Personal, Secure Notes groups
        assert!(result.groups.len() >= 2);
        let group_names: Vec<&str> = result.groups.iter().map(|g| g.name.as_str()).collect();
        assert!(group_names.contains(&"Work"));
        assert!(group_names.contains(&"Personal"));
    }

    #[test]
    fn test_import_invalid_json() {
        let result = import_dashlane("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_import_empty_export() {
        let result = import_dashlane("{}").unwrap();
        assert_eq!(result.entries.len(), 0);
        assert_eq!(result.groups.len(), 0);
    }

    #[test]
    fn test_import_with_otp() {
        let json = r#"{
            "credentials": [{
                "title": "GitHub",
                "username": "user",
                "password": "pass",
                "url": "https://github.com",
                "otpSecret": "JBSWY3DPEHPK3PXP"
            }]
        }"#;
        let result = import_dashlane(json).unwrap();
        assert_eq!(result.entries.len(), 1);
        assert!(result.entries[0].has_otp);
    }
}
