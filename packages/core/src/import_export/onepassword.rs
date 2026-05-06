//! 1Password 1PUX import

use crate::error::{KeePassExError, Result};
use crate::import_export::ImportResult;
use crate::vault::Vault;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct OnePuxExport {
    accounts: Vec<OnePuxAccount>,
}

#[derive(Debug, Deserialize)]
struct OnePuxAccount {
    vaults: Vec<OnePuxVault>,
}

#[derive(Debug, Deserialize)]
struct OnePuxVault {
    attrs: OnePuxVaultAttrs,
    items: Vec<OnePuxItem>,
}

#[derive(Debug, Deserialize)]
struct OnePuxVaultAttrs {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OnePuxItem {
    item: OnePuxItemData,
}

#[derive(Debug, Deserialize)]
struct OnePuxItemData {
    uuid: String,
    title: String,
    #[serde(rename = "categoryUuid")]
    category_uuid: String,
    #[serde(rename = "createdAt")]
    created_at: Option<i64>,
    #[serde(rename = "updatedAt")]
    updated_at: Option<i64>,
    #[serde(rename = "trashed")]
    trashed: Option<bool>,
    details: OnePuxDetails,
    overview: OnePuxOverview,
}

#[derive(Debug, Deserialize)]
struct OnePuxDetails {
    #[serde(rename = "loginFields")]
    login_fields: Option<Vec<OnePuxLoginField>>,
    sections: Option<Vec<OnePuxSection>>,
    #[serde(rename = "notesPlain")]
    notes_plain: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OnePuxLoginField {
    value: String,
    id: String,
    name: String,
    #[serde(rename = "fieldType")]
    field_type: String,
    designation: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OnePuxSection {
    title: String,
    fields: Option<Vec<OnePuxSectionField>>,
}

#[derive(Debug, Deserialize)]
struct OnePuxSectionField {
    title: String,
    id: String,
    value: OnePuxFieldValue,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OnePuxFieldValue {
    String(String),
    Totp { totp: String },
    Other(serde_json::Value),
}

#[derive(Debug, Deserialize)]
struct OnePuxOverview {
    url: Option<String>,
    urls: Option<Vec<OnePuxUrl>>,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OnePuxUrl {
    url: String,
}

pub fn import(vault: &mut Vault, data: &[u8], target_group: Uuid) -> Result<ImportResult> {
    let export: OnePuxExport = serde_json::from_slice(data)
        .map_err(|e| KeePassExError::Serialization(format!("Invalid 1Password 1PUX: {}", e)))?;

    let mut imported = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();
    let mut groups_created = 0;

    for account in &export.accounts {
        for vault_data in &account.vaults {
            // Create a group per vault
            let vault_group = match vault.create_group(&vault_data.attrs.name, target_group) {
                Ok(uuid) => {
                    groups_created += 1;
                    uuid
                }
                Err(_) => target_group,
            };

            for item_wrapper in &vault_data.items {
                let item = &item_wrapper.item;

                // Skip trashed items
                if item.trashed == Some(true) {
                    skipped += 1;
                    continue;
                }

                match vault.create_entry(vault_group) {
                    Ok(uuid) => {
                        if let Some(entry) = vault.get_entry_mut(&uuid) {
                            entry.title.set(&item.title);

                            // URL
                            let url = item
                                .overview
                                .url
                                .clone()
                                .or_else(|| {
                                    item.overview.urls.as_ref()?.first().map(|u| u.url.clone())
                                })
                                .unwrap_or_default();
                            entry.url = url;

                            // Tags
                            if let Some(tags) = &item.overview.tags {
                                entry.tags = tags.clone();
                            }

                            // Notes
                            if let Some(notes) = &item.details.notes_plain {
                                entry.notes.set(notes);
                            }

                            // Login fields
                            if let Some(fields) = &item.details.login_fields {
                                for field in fields {
                                    match field.designation.as_deref() {
                                        Some("username") => entry.username.set(&field.value),
                                        Some("password") => entry.password.set(&field.value),
                                        _ => {
                                            if !field.name.is_empty() && !field.value.is_empty() {
                                                let ps = crate::types::ProtectedString::new(
                                                    &field.value,
                                                );
                                                entry.custom_fields.insert(
                                                    field.name.clone(),
                                                    crate::types::CustomField {
                                                        key: field.name.clone(),
                                                        value: ps,
                                                        protected: false,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }

                            // Timestamps
                            if let Some(ts) = item.updated_at {
                                if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                                    entry.modified_at = dt;
                                }
                            }
                            if let Some(ts) = item.created_at {
                                if let Some(dt) = chrono::DateTime::from_timestamp(ts, 0) {
                                    entry.created_at = dt;
                                }
                            }
                        }
                        imported += 1;
                    }
                    Err(e) => {
                        warnings.push(format!("Item '{}': {}", item.title, e));
                        skipped += 1;
                    }
                }
            }
        }
    }

    Ok(ImportResult {
        entries_imported: imported,
        groups_created,
        entries_skipped: skipped,
        warnings,
    })
}
