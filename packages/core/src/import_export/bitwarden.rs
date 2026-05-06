//! Bitwarden JSON import

use crate::error::{KeePassExError, Result};
use crate::import_export::ImportResult;
use crate::vault::Vault;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct BitwardenExport {
    items: Vec<BitwardenItem>,
    folders: Option<Vec<BitwardenFolder>>,
}

#[derive(Debug, Deserialize)]
struct BitwardenFolder {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct BitwardenItem {
    id: String,
    name: String,
    #[serde(rename = "type")]
    item_type: u8, // 1=login, 2=note, 3=card, 4=identity
    #[serde(rename = "folderId")]
    folder_id: Option<String>,
    notes: Option<String>,
    login: Option<BitwardenLogin>,
    fields: Option<Vec<BitwardenField>>,
    #[serde(rename = "revisionDate")]
    revision_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitwardenLogin {
    username: Option<String>,
    password: Option<String>,
    uris: Option<Vec<BitwardenUri>>,
    totp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitwardenUri {
    uri: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BitwardenField {
    name: Option<String>,
    value: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<u8>, // 0=text, 1=hidden, 2=boolean
}

pub fn import(vault: &mut Vault, data: &[u8], target_group: Uuid) -> Result<ImportResult> {
    let export: BitwardenExport = serde_json::from_slice(data)
        .map_err(|e| KeePassExError::Serialization(format!("Invalid Bitwarden JSON: {}", e)))?;

    let mut imported = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();
    let mut folder_map: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

    // Create groups for folders
    if let Some(folders) = &export.folders {
        for folder in folders {
            match vault.create_group(&folder.name, target_group) {
                Ok(uuid) => {
                    folder_map.insert(folder.id.clone(), uuid);
                }
                Err(e) => warnings.push(format!("Folder '{}': {}", folder.name, e)),
            }
        }
    }

    for item in &export.items {
        // Only import login items for now
        if item.item_type != 1 {
            skipped += 1;
            continue;
        }

        let group = item
            .folder_id
            .as_ref()
            .and_then(|fid| folder_map.get(fid))
            .copied()
            .unwrap_or(target_group);

        match vault.create_entry(group) {
            Ok(uuid) => {
                if let Some(entry) = vault.get_entry_mut(&uuid) {
                    entry.title.set(&item.name);

                    if let Some(login) = &item.login {
                        entry.username.set(login.username.as_deref().unwrap_or(""));
                        entry.password.set(login.password.as_deref().unwrap_or(""));

                        // First URI as URL
                        if let Some(uris) = &login.uris {
                            if let Some(first) = uris.first() {
                                entry.url = first.uri.clone().unwrap_or_default();
                            }
                        }

                        // TOTP
                        if let Some(totp_secret) = &login.totp {
                            if !totp_secret.is_empty() {
                                use crate::otp;
                                if let Ok(otp_config) =
                                    otp::parse_otp_uri(totp_secret).or_else(|_| {
                                        // Plain secret — wrap in otpauth URI
                                        otp::parse_otp_uri(&format!(
                                            "otpauth://totp/{}?secret={}",
                                            urlencoding::encode(&item.name),
                                            totp_secret
                                        ))
                                    })
                                {
                                    entry.otp = Some(otp_config);
                                }
                            }
                        }
                    }

                    entry.notes.set(item.notes.as_deref().unwrap_or(""));

                    // Custom fields
                    if let Some(fields) = &item.fields {
                        for field in fields {
                            if let (Some(key), Some(value)) = (&field.name, &field.value) {
                                let protected = field.field_type == Some(1);
                                let mut ps = crate::types::ProtectedString::new(value.as_str());
                                ps.protected = protected;
                                entry.custom_fields.insert(
                                    key.clone(),
                                    crate::types::CustomField {
                                        key: key.clone(),
                                        value: ps,
                                        protected,
                                    },
                                );
                            }
                        }
                    }

                    // Parse revision date
                    if let Some(date_str) = &item.revision_date {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
                            entry.modified_at = dt.with_timezone(&chrono::Utc);
                        }
                    }
                }
                imported += 1;
            }
            Err(e) => {
                warnings.push(format!("Item '{}': {}", item.name, e));
                skipped += 1;
            }
        }
    }

    Ok(ImportResult {
        entries_imported: imported,
        groups_created: folder_map.len(),
        entries_skipped: skipped,
        warnings,
    })
}
