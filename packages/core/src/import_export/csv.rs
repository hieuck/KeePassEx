//! Generic CSV import/export

use crate::error::{KeePassExError, Result};
use crate::vault::Vault;
use crate::types::ProtectedString;
use crate::import_export::ImportResult;
use uuid::Uuid;
use chrono::Utc;

/// Import from generic CSV (title, username, password, url, notes)
pub fn import_generic(vault: &mut Vault, data: &[u8], target_group: Uuid) -> Result<ImportResult> {
    let text = std::str::from_utf8(data)
        .map_err(|_| KeePassExError::Other("Invalid UTF-8 in CSV".into()))?;

    let mut reader = csv_reader(text);
    let mut imported = 0;
    let mut skipped = 0;
    let mut warnings = Vec::new();

    // Try to detect header
    let mut lines = text.lines().peekable();
    let header = lines.peek().map(|l| l.to_lowercase());
    let has_header = header.as_deref().map(|h|
        h.contains("title") || h.contains("name") || h.contains("username")
    ).unwrap_or(false);

    for (i, line) in text.lines().enumerate() {
        if i == 0 && has_header {
            continue; // Skip header
        }

        let fields: Vec<&str> = parse_csv_line(line);
        if fields.is_empty() || fields.iter().all(|f| f.is_empty()) {
            continue;
        }

        let title = fields.get(0).copied().unwrap_or("").trim().to_string();
        let username = fields.get(1).copied().unwrap_or("").trim().to_string();
        let password = fields.get(2).copied().unwrap_or("").trim().to_string();
        let url = fields.get(3).copied().unwrap_or("").trim().to_string();
        let notes = fields.get(4).copied().unwrap_or("").trim().to_string();

        if title.is_empty() && username.is_empty() && password.is_empty() {
            skipped += 1;
            continue;
        }

        match vault.create_entry(target_group) {
            Ok(uuid) => {
                if let Some(entry) = vault.get_entry_mut(&uuid) {
                    entry.title.set(if title.is_empty() { &url } else { &title });
                    entry.username.set(&username);
                    entry.password.set(&password);
                    entry.url = url;
                    entry.notes.set(&notes);
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

/// Export vault to CSV (unencrypted — warn user)
pub fn export_csv(vault: &Vault) -> Result<Vec<u8>> {
    let mut output = String::new();

    // Header
    output.push_str("Title,Username,Password,URL,Notes,Tags,Modified\n");

    for entry in vault.all_entries() {
        let fields = [
            csv_escape(entry.title.get()),
            csv_escape(entry.username.get()),
            csv_escape(entry.password.get()),
            csv_escape(&entry.url),
            csv_escape(entry.notes.get()),
            csv_escape(&entry.tags.join(";")),
            csv_escape(&entry.modified_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        ];
        output.push_str(&fields.join(","));
        output.push('\n');
    }

    Ok(output.into_bytes())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Public version for use by other importers
pub fn parse_csv_line_pub(line: &str) -> Vec<&str> {
    parse_csv_line(line)
}

fn parse_csv_line(line: &str) -> Vec<&str> {
    // Simple CSV parser — handles quoted fields
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let chars: Vec<char> = line.chars().collect();
    let bytes = line.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                in_quotes = !in_quotes;
                i += 1;
            }
            b',' if !in_quotes => {
                fields.push(line[start..i].trim_matches('"'));
                start = i + 1;
                i += 1;
            }
            _ => i += 1,
        }
    }
    fields.push(line[start..].trim_matches('"'));
    fields
}

fn csv_reader(_text: &str) {} // placeholder
