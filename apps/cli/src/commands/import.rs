//! CLI `import` command — import entries from various formats

use keepassex_core::{
    Vault,
    import_export::{import_into_vault, detect_format, ImportFormat},
    vault::operations::{save_vault, VaultCredentials},
};
use colored::Colorize;
use std::path::Path;

pub async fn run(
    vault: &mut Vault,
    vault_path: &str,
    password: &str,
    input: &str,
    format: &str,
) -> anyhow::Result<()> {
    let data = std::fs::read(input)?;

    let import_format = if format == "auto" {
        detect_format(&data).ok_or_else(|| {
            anyhow::anyhow!(
                "Could not detect format. Specify with --format: bitwarden, lastpass, chrome, firefox, 1password, csv"
            )
        })?
    } else {
        match format.to_lowercase().as_str() {
            "bitwarden" | "bitwarden-json" => ImportFormat::BitwardenJson,
            "lastpass" | "lastpass-csv" => ImportFormat::LastPassCsv,
            "chrome" | "chrome-csv" => ImportFormat::ChromeCsv,
            "firefox" | "firefox-csv" => ImportFormat::FirefoxCsv,
            "1password" | "1password-opux" | "opux" => ImportFormat::OnePasswordOpux,
            "csv" | "generic-csv" => ImportFormat::GenericCsv,
            _ => return Err(anyhow::anyhow!("Unknown format: {}", format)),
        }
    };

    let format_name = format!("{:?}", import_format);
    eprintln!("Importing from {} ({})...", input.bold(), format_name.dimmed());

    let result = import_into_vault(vault, &data, import_format, None)?;

    // Save vault
    let credentials = VaultCredentials::password_only(password);
    save_vault(vault, Path::new(vault_path), &credentials).await?;

    // Report
    eprintln!(
        "{} Import complete:",
        "✓".green()
    );
    eprintln!(
        "  {} entries imported",
        result.entries_imported.to_string().bold().green()
    );
    if result.groups_created > 0 {
        eprintln!(
            "  {} groups created",
            result.groups_created.to_string().bold()
        );
    }
    if result.entries_skipped > 0 {
        eprintln!(
            "  {} entries skipped",
            result.entries_skipped.to_string().yellow()
        );
    }
    for warning in &result.warnings {
        eprintln!("  {} {}", "⚠".yellow(), warning.dimmed());
    }

    Ok(())
}
