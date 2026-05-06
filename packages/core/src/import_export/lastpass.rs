//! LastPass CSV import
//! Format: url,username,password,totp,extra,name,grouping,fav

use crate::error::{KeePassExError, Result};
use crate::import_export::ImportResult;
use crate::vault::Vault;
use uuid::Uuid;

pub fn import(vault: &mut Vault, data: &[u8], target_group: Uuid) -> Result<ImportResult> {
    let text = std::str::from_utf8(data)
        .map_err(|_| KeePassExError::Other("Invalid UTF-8 in LastPass CSV".into()))?;

    let mut imported = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();
    let mut group_map: std::collections::HashMap<String, Uuid> = std::collections::HashMap::new();

    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue; // Skip header
        }

        let fields = super::csv::parse_csv_line_pub(line);
        if fields.len() < 6 {
            skipped += 1;
            continue;
        }

        let url = fields[0].trim().to_string();
        let username = fields[1].trim().to_string();
        let password = fields[2].trim().to_string();
        let totp = fields.get(3).copied().unwrap_or("").trim().to_string();
        let notes = fields.get(4).copied().unwrap_or("").trim().to_string();
        let name = fields.get(5).copied().unwrap_or("").trim().to_string();
        let grouping = fields.get(6).copied().unwrap_or("").trim().to_string();

        // Create group if needed
        let group_uuid = if !grouping.is_empty() {
            if let Some(&uuid) = group_map.get(&grouping) {
                uuid
            } else {
                match vault.create_group(&grouping, target_group) {
                    Ok(uuid) => {
                        group_map.insert(grouping.clone(), uuid);
                        uuid
                    }
                    Err(_) => target_group,
                }
            }
        } else {
            target_group
        };

        let title = if name.is_empty() {
            url.clone()
        } else {
            name.clone()
        };

        match vault.create_entry(group_uuid) {
            Ok(uuid) => {
                if let Some(entry) = vault.get_entry_mut(&uuid) {
                    entry.title.set(&title);
                    entry.username.set(&username);
                    entry.password.set(&password);
                    entry.url = url;
                    entry.notes.set(&notes);

                    // TOTP
                    if !totp.is_empty() {
                        if let Ok(otp_config) = crate::otp::parse_otp_uri(&totp).or_else(|_| {
                            crate::otp::parse_otp_uri(&format!(
                                "otpauth://totp/{}?secret={}",
                                title, totp
                            ))
                        }) {
                            entry.otp = Some(otp_config);
                        }
                    }
                }
                imported += 1;
            }
            Err(e) => {
                warnings.push(format!("Row {}: {}", i + 1, e));
                skipped += 1;
            }
        }
    }

    Ok(ImportResult {
        entries_imported: imported,
        groups_created: group_map.len(),
        entries_skipped: skipped,
        warnings,
    })
}
