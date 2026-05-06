//! Import/Export Tauri commands

use crate::state::AppState;
use keepassex_core::import_export::{
    import_into_vault, export_vault, detect_format,
    ImportFormat, ExportFormat,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct ImportArgs {
    pub file_path: String,
    pub format: Option<String>, // None = auto-detect
    pub target_group_uuid: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ImportResultDto {
    pub entries_imported: usize,
    pub groups_created: usize,
    pub entries_skipped: usize,
    pub warnings: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExportArgs {
    pub file_path: String,
    pub format: String, // "csv" | "json"
}

/// Import entries from an external file
#[tauri::command]
pub async fn import_vault(
    args: ImportArgs,
    state: State<'_, AppState>,
) -> Result<ImportResultDto, String> {
    let data = tokio::fs::read(&args.file_path)
        .await
        .map_err(|e| format!("Cannot read file: {}", e))?;

    let format = if let Some(fmt) = &args.format {
        match fmt.to_lowercase().as_str() {
            "bitwarden" => ImportFormat::BitwardenJson,
            "lastpass" => ImportFormat::LastPassCsv,
            "chrome" => ImportFormat::ChromeCsv,
            "firefox" => ImportFormat::FirefoxCsv,
            "1password" | "opux" => ImportFormat::OnePasswordOpux,
            "csv" => ImportFormat::GenericCsv,
            _ => return Err(format!("Unknown format: {}", fmt)),
        }
    } else {
        detect_format(&data).ok_or("Could not detect file format")?
    };

    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let target = args.target_group_uuid
        .as_deref()
        .and_then(|s| uuid::Uuid::parse_str(s).ok());

    let result = import_into_vault(&mut open_vault.vault, &data, format, target)
        .map_err(|e| e.to_string())?;

    Ok(ImportResultDto {
        entries_imported: result.entries_imported,
        groups_created: result.groups_created,
        entries_skipped: result.entries_skipped,
        warnings: result.warnings,
    })
}

/// Export vault to CSV or JSON
#[tauri::command]
pub async fn export_vault_cmd(
    args: ExportArgs,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let format = match args.format.to_lowercase().as_str() {
        "csv" => ExportFormat::CsvUnencrypted,
        "json" => ExportFormat::JsonUnencrypted,
        _ => return Err(format!("Unknown export format: {}", args.format)),
    };

    let data = export_vault(&open_vault.vault, format).map_err(|e| e.to_string())?;
    let size = data.len();

    tokio::fs::write(&args.file_path, &data)
        .await
        .map_err(|e| format!("Cannot write file: {}", e))?;

    Ok(size)
}

/// Detect format of an import file
#[tauri::command]
pub async fn detect_import_format(file_path: String) -> Result<String, String> {
    let data = tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Cannot read file: {}", e))?;

    match detect_format(&data) {
        Some(fmt) => Ok(format!("{:?}", fmt)),
        None => Err("Could not detect format".into()),
    }
}
