//! CLI `add` command — create a new entry interactively or from flags

use keepassex_core::{Vault, vault::operations::{save_vault, VaultCredentials}};
use keepassex_core::generator::{PasswordGenerator};
use keepassex_core::types::PasswordGeneratorConfig;
use colored::Colorize;
use uuid::Uuid;
use std::path::Path;

pub async fn run(
    vault: &mut Vault,
    vault_path: &str,
    password: &str,
    title: String,
    username: Option<String>,
    password_arg: Option<String>,
    url: Option<String>,
    notes: Option<String>,
    group: Option<String>,
    generate: bool,
) -> anyhow::Result<()> {
    // Resolve group
    let group_uuid = if let Some(group_str) = group {
        // Try UUID first, then search by name
        if let Ok(uuid) = Uuid::parse_str(&group_str) {
            uuid
        } else {
            vault
                .all_groups()
                .find(|g| g.name.eq_ignore_ascii_case(&group_str))
                .map(|g| g.uuid)
                .unwrap_or(vault.root_group_uuid)
        }
    } else {
        vault.root_group_uuid
    };

    // Resolve password
    let final_password = if generate {
        let config = PasswordGeneratorConfig::default();
        let pw = PasswordGenerator::generate(&config)?;
        let strength = PasswordGenerator::score_strength(&pw);
        eprintln!(
            "{} Generated password: {} ({})",
            "✓".green(),
            pw.bold(),
            strength.label_en().green()
        );
        pw
    } else if let Some(pw) = password_arg {
        pw
    } else {
        // Prompt interactively
        rpassword::prompt_password("Password (leave empty to skip): ")?
    };

    // Create entry
    let entry_uuid = vault.create_entry(group_uuid)?;
    if let Some(entry) = vault.get_entry_mut(&entry_uuid) {
        entry.title.set(&title);
        entry.username.set(username.as_deref().unwrap_or(""));
        entry.password.set(&final_password);
        entry.url = url.unwrap_or_default();
        entry.notes.set(notes.as_deref().unwrap_or(""));
    }

    // Save vault
    let credentials = VaultCredentials::password_only(password);
    save_vault(vault, Path::new(vault_path), &credentials).await?;

    eprintln!(
        "{} Entry '{}' created ({})",
        "✓".green(),
        title.bold(),
        entry_uuid.to_string()[..8].dimmed()
    );

    Ok(())
}
