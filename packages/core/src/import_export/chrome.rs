//! Chrome/Firefox/Edge CSV import
//! Chrome format: name,url,username,password
//! Firefox format: "widechar_url","httpRealm","formActionOrigin","guid","timeCreated","timeLastUsed","timePasswordChanged","username","password"

use crate::error::{KeePassExError, Result};
use crate::vault::Vault;
use crate::import_export::ImportResult;
use uuid::Uuid;

pub fn import(vault: &mut Vault, data: &[u8], target_group: Uuid) -> Result<ImportResult> {
    let text = std::str::from_utf8(data)
        .map_err(|_| KeePassExError::Other("Invalid UTF-8 in CSV".into()))?;

    let mut lines = text.lines();
    let header = lines.next().unwrap_or("").to_lowercase();

    // Detect Chrome vs Firefox format
    let is_firefox = header.contains("widechar_url") || header.contains("httpRealm");

    let mut imported = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();

    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue; // Skip header
        }

        let fields = super::csv::parse_csv_line_pub(line);

        let (name, url, username, password) = if is_firefox {
            // Firefox: url, httpRealm, formActionOrigin, guid, timeCreated, timeLastUsed, timePasswordChanged, username, password
            let url = fields.get(0).copied().unwrap_or("").trim().to_string();
            let username = fields.get(7).copied().unwrap_or("").trim().to_string();
            let password = fields.get(8).copied().unwrap_or("").trim().to_string();
            let name = extract_domain(&url);
            (name, url, username, password)
        } else {
            // Chrome: name, url, username, password
            let name = fields.get(0).copied().unwrap_or("").trim().to_string();
            let url = fields.get(1).copied().unwrap_or("").trim().to_string();
            let username = fields.get(2).copied().unwrap_or("").trim().to_string();
            let password = fields.get(3).copied().unwrap_or("").trim().to_string();
            (name, url, username, password)
        };

        if username.is_empty() && password.is_empty() {
            skipped += 1;
            continue;
        }

        match vault.create_entry(target_group) {
            Ok(uuid) => {
                if let Some(entry) = vault.get_entry_mut(&uuid) {
                    entry.title.set(if name.is_empty() { &url } else { &name });
                    entry.username.set(&username);
                    entry.password.set(&password);
                    entry.url = url;
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
        groups_created: 0,
        entries_skipped: skipped,
        warnings,
    })
}

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}
