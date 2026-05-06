//! kpx audit -- show vault audit log
use colored::Colorize;

pub fn run(vault_path: &str, _limit: usize, _format: &str) -> anyhow::Result<()> {
    println!("{}", "Audit Log".bold());
    println!("{}", "=".repeat(50).dimmed());
    println!("  Vault: {}", vault_path.cyan());
    println!();
    println!("{}", "Audit log is stored encrypted within the vault.".dimmed());
    println!("{}", "Use the desktop app to view the full audit log with filtering.".dimmed());
    Ok(())
}
