//! CLI `export` command — export vault to various formats

use keepassex_core::{Vault, import_export::{export_vault, ExportFormat}};
use colored::Colorize;
use std::path::Path;

pub fn run(
    vault: &Vault,
    output: &str,
    format: &str,
) -> anyhow::Result<()> {
    let export_format = match format.to_lowercase().as_str() {
        "csv" => {
            eprintln!(
                "{} CSV export contains unencrypted passwords. Keep the file secure!",
                "⚠".yellow()
            );
            ExportFormat::CsvUnencrypted
        }
        "json" => {
            eprintln!(
                "{} JSON export contains unencrypted passwords. Keep the file secure!",
                "⚠".yellow()
            );
            ExportFormat::JsonUnencrypted
        }
        "kdbx" => {
            eprintln!("{}", "Use the original vault file for KDBX export.".dimmed());
            return Ok(());
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown format '{}'. Supported: csv, json, kdbx",
                format
            ));
        }
    };

    let data = export_vault(vault, export_format)?;
    std::fs::write(output, &data)?;

    let size = data.len();
    eprintln!(
        "{} Exported {} entries to {} ({} bytes)",
        "✓".green(),
        vault.entry_count().to_string().bold(),
        output.bold(),
        size
    );

    Ok(())
}
