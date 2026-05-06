//! CLI `edit` command — edit an existing entry

use keepassex_core::{Vault, vault::operations::{save_vault, VaultCredentials}};
use colored::Colorize;
use uuid::Uuid;
use std::path::Path;

pub async fn run(
    vault: &mut Vault,
    vault_path: &str,
    password: &str,
    uuid_str: &str,
) -> anyhow::Result<()> {
    // Find entry
    let uuid = resolve_uuid(vault, uuid_str)?;

    let entry = vault
        .get_entry(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_str))?
        .clone();

    eprintln!("{}", format!("Editing: {}", entry.title.get()).bold());
    eprintln!("{}", "Press Enter to keep current value".dimmed());

    // Interactive editing
    let new_title = prompt_with_default("Title", entry.title.get())?;
    let new_username = prompt_with_default("Username", entry.username.get())?;

    let new_password = {
        eprint!("Password [keep current]: ");
        let input = read_line()?;
        if input.is_empty() {
            entry.password.get().to_string()
        } else {
            input
        }
    };

    let new_url = prompt_with_default("URL", &entry.url)?;
    let new_notes = prompt_with_default("Notes", entry.notes.get())?;

    // Apply changes
    let mut updated = entry.clone();
    updated.title.set(&new_title);
    updated.username.set(&new_username);
    updated.password.set(&new_password);
    updated.url = new_url;
    updated.notes.set(&new_notes);
    updated.modified_at = chrono::Utc::now();

    vault.update_entry(updated)?;

    // Save
    let credentials = VaultCredentials::password_only(password);
    save_vault(vault, Path::new(vault_path), &credentials).await?;

    eprintln!("{} Entry '{}' updated", "✓".green(), new_title.bold());
    Ok(())
}

fn resolve_uuid(vault: &Vault, uuid_str: &str) -> anyhow::Result<Uuid> {
    if let Ok(uuid) = Uuid::parse_str(uuid_str) {
        return Ok(uuid);
    }
    // Partial UUID prefix match
    vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_str))
        .map(|e| e.uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_str))
}

fn prompt_with_default(label: &str, current: &str) -> anyhow::Result<String> {
    eprint!("{} [{}]: ", label, current.dimmed());
    let input = read_line()?;
    Ok(if input.is_empty() { current.to_string() } else { input })
}

fn read_line() -> anyhow::Result<String> {
    use std::io::BufRead;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    Ok(line.trim_end_matches('\n').trim_end_matches('\r').to_string())
}
