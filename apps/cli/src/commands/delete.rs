//! CLI `delete` command — delete an entry

use keepassex_core::{Vault, vault::operations::{save_vault, VaultCredentials}};
use colored::Colorize;
use uuid::Uuid;
use std::path::Path;

pub async fn run(
    vault: &mut Vault,
    vault_path: &str,
    password: &str,
    uuid_str: &str,
    permanent: bool,
    force: bool,
) -> anyhow::Result<()> {
    let uuid = resolve_uuid(vault, uuid_str)?;

    let title = vault
        .get_entry(&uuid)
        .map(|e| e.title.get().to_string())
        .unwrap_or_else(|| uuid_str.to_string());

    // Confirm unless --force
    if !force {
        let action = if permanent { "permanently delete" } else { "move to recycle bin" };
        eprint!(
            "{} '{}' ({})? [y/N]: ",
            format!("Are you sure you want to {} entry", action).yellow(),
            title.bold(),
            &uuid_str[..8.min(uuid_str.len())]
        );

        use std::io::BufRead;
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let answer = line.trim().to_lowercase();

        if answer != "y" && answer != "yes" {
            eprintln!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    vault.delete_entry(&uuid, permanent)?;

    let credentials = VaultCredentials::password_only(password);
    save_vault(vault, Path::new(vault_path), &credentials).await?;

    let action = if permanent { "permanently deleted" } else { "moved to recycle bin" };
    eprintln!("{} '{}' {}", "✓".green(), title.bold(), action);

    Ok(())
}

fn resolve_uuid(vault: &Vault, uuid_str: &str) -> anyhow::Result<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
        return Ok(uuid);
    }
    vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_str))
        .map(|e| e.uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_str))
}
