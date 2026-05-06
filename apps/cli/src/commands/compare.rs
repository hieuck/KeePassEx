//! kpx compare -- diff two KDBX vaults and optionally merge
use keepassex_core::vault_compare::{compare_vaults, WhichVault};
use keepassex_core::vault::operations::{open_vault, VaultCredentials};
use colored::Colorize;

pub async fn run(
    vault1_path: &str,
    vault2_path: &str,
    password1: &str,
    password2: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let creds1 = VaultCredentials::password_only(password1);
    let creds2 = VaultCredentials::password_only(password2.unwrap_or(password1));

    let vault1 = open_vault(std::path::Path::new(vault1_path), &creds1).await
        .map_err(|e| anyhow::anyhow!("Failed to open first vault: {}", e))?;
    let vault2 = open_vault(std::path::Path::new(vault2_path), &creds2).await
        .map_err(|e| anyhow::anyhow!("Failed to open second vault: {}", e))?;

    let diff = compare_vaults(&vault1, &vault2);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&diff)?);
        return Ok(());
    }

    println!("{}", "Vault Comparison".bold());
    println!("{}", "=".repeat(50).dimmed());
    println!("  First:  {} ({} entries)", vault1_path.cyan(), diff.first_total);
    println!("  Second: {} ({} entries)", vault2_path.cyan(), diff.second_total);
    println!();

    if diff.is_identical() {
        println!("{}", "Vaults are identical!".green().bold());
        return Ok(());
    }

    println!("{}", format!("Found {} difference(s):", diff.diff_count()).yellow());
    println!();

    for e in &diff.only_in_first {
        println!("  {} {} ({})", "-".red(), e.title.bold(), e.username.dimmed());
    }
    for e in &diff.only_in_second {
        println!("  {} {} ({})", "+".green(), e.title.bold(), e.username.dimmed());
    }
    for m in &diff.modified {
        let newer = match &m.newer_in {
            WhichVault::First  => "newer in first",
            WhichVault::Second => "newer in second",
            WhichVault::Same   => "same timestamp",
        };
        println!("  {} {} [{}] changed: {}",
            "~".yellow(), m.title.bold(), newer.yellow(),
            m.changed_fields.join(", ").dimmed());
    }

    if diff.identical_count > 0 {
        println!();
        println!("  {} identical entries", diff.identical_count.to_string().dimmed());
    }

    Ok(())
}
