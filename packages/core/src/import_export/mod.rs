//! Import/Export module
//!
//! Supports importing from: Bitwarden, LastPass, 1Password, Chrome, Firefox,
//! Dashlane, NordPass, Enpass, RoboForm, KeePass 1.x XML, Generic CSV
//! Supports exporting to: KDBX 4.x, CSV (unencrypted warning), JSON

pub mod bitwarden;
pub mod chrome;
pub mod csv;
pub mod dashlane;
pub mod enpass;
pub mod import_types;
pub mod keepass1;
pub mod lastpass;
pub mod nordpass;
pub mod onepassword;
pub mod roboform;

pub use import_types::{ImportCustomField, ImportEntry, ImportGroup, ImportResult as ImportBatch};

use crate::error::{KeePassExError, Result};
use crate::vault::Vault;
use chrono::Utc;
use uuid::Uuid;

// ─── Import Formats ───────────────────────────────────────────────────────────

/// Supported import formats
#[derive(Debug, Clone, PartialEq)]
pub enum ImportFormat {
    KdbxAny,
    BitwardenJson,
    LastPassCsv,
    OnePasswordOpux,
    ChromeCsv,
    FirefoxCsv,
    DashlaneCsv,
    DashlaneJson,
    NordPassCsv,
    EnpassJson,
    RoboFormHtml,
    KeePass1Xml,
    GenericCsv,
}

/// Supported export formats
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Kdbx4,
    CsvUnencrypted,
    JsonUnencrypted,
}

/// Import result summary
#[derive(Debug, Clone)]
pub struct ImportResult {
    pub entries_imported: usize,
    pub groups_created: usize,
    pub entries_skipped: usize,
    pub warnings: Vec<String>,
}

/// Export result
#[derive(Debug, Clone)]
pub struct ExportResult {
    pub entries_exported: usize,
    pub format: ExportFormat,
    pub file_size_bytes: usize,
}

// ─── Format Detection ─────────────────────────────────────────────────────────

/// Detect import format from file content
pub fn detect_format(data: &[u8]) -> Option<ImportFormat> {
    // KDBX signature
    if data.len() >= 8 {
        let sig1 = u32::from_le_bytes(data[0..4].try_into().unwrap_or([0; 4]));
        let sig2 = u32::from_le_bytes(data[4..8].try_into().unwrap_or([0; 4]));
        if sig1 == 0x9AA2D903 && sig2 == 0xB54BFB67 {
            return Some(ImportFormat::KdbxAny);
        }
    }

    let Ok(text) = std::str::from_utf8(data) else {
        return None;
    };
    let trimmed = text.trim_start();

    // XML / HTML
    if trimmed.starts_with("<?xml") || trimmed.starts_with("<pwlist") {
        if text.contains("<pwentry>") || text.contains("<pwlist>") {
            return Some(ImportFormat::KeePass1Xml);
        }
    }
    if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
        if text.contains("<table") {
            return Some(ImportFormat::RoboFormHtml);
        }
    }

    // JSON
    if trimmed.starts_with('{') {
        if text.contains("\"encrypted\"") && text.contains("\"items\"") {
            return Some(ImportFormat::BitwardenJson);
        }
        if text.contains("\"1pux\"") || text.contains("\"accounts\"") {
            return Some(ImportFormat::OnePasswordOpux);
        }
        if text.contains("\"credentials\"") && text.contains("\"securenotes\"") {
            return Some(ImportFormat::DashlaneJson);
        }
        if text.contains("\"folders\"") && text.contains("\"items\"") {
            return Some(ImportFormat::EnpassJson);
        }
    }

    // CSV
    if trimmed.starts_with("name,url,username,password,note,cardholder,cardnumber") {
        return Some(ImportFormat::NordPassCsv);
    }
    if trimmed.starts_with("name,url,username,password")
        || trimmed.starts_with("Name,Url,Username,Password")
    {
        return Some(ImportFormat::ChromeCsv);
    }
    if trimmed.starts_with("url,username,password,totp,extra,name,grouping,fav") {
        return Some(ImportFormat::LastPassCsv);
    }
    if trimmed.starts_with("\"widechar_url\"") || trimmed.starts_with("widechar_url") {
        return Some(ImportFormat::FirefoxCsv);
    }

    None
}

// ─── Import ───────────────────────────────────────────────────────────────────

/// Import entries into a vault from raw data
pub fn import_into_vault(
    vault: &mut Vault,
    data: &[u8],
    format: ImportFormat,
    target_group_uuid: Option<Uuid>,
) -> Result<ImportResult> {
    let target = target_group_uuid.unwrap_or(vault.root_group_uuid);

    match format {
        ImportFormat::BitwardenJson => bitwarden::import(vault, data, target),
        ImportFormat::LastPassCsv => lastpass::import(vault, data, target),
        ImportFormat::ChromeCsv | ImportFormat::FirefoxCsv => chrome::import(vault, data, target),
        ImportFormat::GenericCsv => csv::import_generic(vault, data, target),
        ImportFormat::OnePasswordOpux => onepassword::import(vault, data, target),
        ImportFormat::KdbxAny => Err(KeePassExError::Other(
            "Use open_vault() to import KDBX files".into(),
        )),
        ImportFormat::DashlaneJson | ImportFormat::DashlaneCsv => {
            let text = std::str::from_utf8(data)
                .map_err(|e| KeePassExError::ImportParseFailed(e.to_string()))?;
            let result = dashlane::import_dashlane(text)?;
            add_import_batch_to_vault(vault, result, target)
        }
        ImportFormat::NordPassCsv => {
            let text = std::str::from_utf8(data)
                .map_err(|e| KeePassExError::ImportParseFailed(e.to_string()))?;
            let result = nordpass::import_nordpass(text)?;
            add_import_batch_to_vault(vault, result, target)
        }
        ImportFormat::EnpassJson => {
            let text = std::str::from_utf8(data)
                .map_err(|e| KeePassExError::ImportParseFailed(e.to_string()))?;
            let result = enpass::import_enpass(text)?;
            add_import_batch_to_vault(vault, result, target)
        }
        ImportFormat::RoboFormHtml => {
            let text = std::str::from_utf8(data)
                .map_err(|e| KeePassExError::ImportParseFailed(e.to_string()))?;
            let result = roboform::import_roboform(text)?;
            add_import_batch_to_vault(vault, result, target)
        }
        ImportFormat::KeePass1Xml => {
            let text = std::str::from_utf8(data)
                .map_err(|e| KeePassExError::ImportParseFailed(e.to_string()))?;
            let result = keepass1::import_keepass1_xml(text)?;
            add_import_batch_to_vault(vault, result, target)
        }
    }
}

/// Convert an `ImportBatch` (ImportEntry/ImportGroup) into vault entries
fn add_import_batch_to_vault(
    vault: &mut Vault,
    batch: import_types::ImportResult,
    _target: Uuid,
) -> Result<ImportResult> {
    let groups_created = batch.groups.len();
    let entries_imported = batch.entries.len();
    let warnings = batch.warnings;

    // Convert and add groups
    for ig in batch.groups {
        let group = ig.into_group();
        vault.insert_group(group);
    }

    // Convert and add entries
    for ie in batch.entries {
        let entry = ie.into_entry();
        vault.insert_entry(entry);
    }

    Ok(ImportResult {
        entries_imported,
        groups_created,
        entries_skipped: 0,
        warnings,
    })
}

// ─── Export ───────────────────────────────────────────────────────────────────

/// Export vault to bytes
pub fn export_vault(vault: &Vault, format: ExportFormat) -> Result<Vec<u8>> {
    match format {
        ExportFormat::CsvUnencrypted => csv::export_csv(vault),
        ExportFormat::JsonUnencrypted => export_json(vault),
        ExportFormat::Kdbx4 => Err(KeePassExError::Other(
            "Use save_vault() to export as KDBX".into(),
        )),
    }
}

fn export_json(vault: &Vault) -> Result<Vec<u8>> {
    let entries: Vec<serde_json::Value> = vault
        .all_entries()
        .map(|e| {
            serde_json::json!({
                "uuid": e.uuid.to_string(),
                "title": e.title.get(),
                "username": e.username.get(),
                "url": e.url,
                "notes": e.notes.get(),
                "tags": e.tags,
                "created": e.created_at.to_rfc3339(),
                "modified": e.modified_at.to_rfc3339(),
            })
        })
        .collect();

    let json = serde_json::json!({
        "generator": "KeePassEx",
        "version": "1.0",
        "exported_at": Utc::now().to_rfc3339(),
        "entries": entries,
    });

    serde_json::to_vec_pretty(&json).map_err(|e| KeePassExError::Serialization(e.to_string()))
}
