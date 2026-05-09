//! kpx tag — manage entry tags
//!
//! KeePassXC doesn't have a dedicated tag command — KeePassEx does.
//!
//! Usage:
//!   kpx tag list                       # List all tags with entry counts
//!   kpx tag add <uuid> <tag>           # Add tag to entry
//!   kpx tag remove <uuid> <tag>        # Remove tag from entry
//!   kpx tag entries <tag>              # List entries with tag
//!   kpx tag rename <old> <new>         # Rename tag across all entries

use colored::Colorize;
use keepassex_core::Vault;

pub fn run_list(vault: &Vault, format: &str) -> anyhow::Result<()> {
    let mut tag_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for entry in vault.all_entries() {
        for tag in &entry.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }

    if tag_counts.is_empty() {
        println!("{}", "No tags found.".dimmed());
        return Ok(());
    }

    let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = tags
                .iter()
                .map(|(tag, count)| serde_json::json!({ "tag": tag, "count": count }))
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("{}", "Tags".bold());
            println!("{}", "─".repeat(40).dimmed());
            for (tag, count) in &tags {
                println!(
                    "  {} {}",
                    format!("{:<20}", tag).cyan(),
                    count.to_string().dimmed()
                );
            }
            println!("{}", "─".repeat(40).dimmed());
            println!(
                "{} tags, {} tagged entries",
                tags.len().to_string().bold(),
                tags.iter()
                    .map(|(_, c)| c)
                    .sum::<usize>()
                    .to_string()
                    .bold()
            );
        }
    }

    Ok(())
}

pub fn run_add(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    tag: &str,
) -> anyhow::Result<()> {
    let entry = vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_prefix))?;

    let uuid = entry.uuid;
    let title = entry.title.get().to_string();

    if entry.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
        println!(
            "{} Tag '{}' already exists on '{}'",
            "ℹ".cyan(),
            tag.bold(),
            title.bold()
        );
        return Ok(());
    }

    let entry = vault
        .get_entry_mut(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;
    entry.tags.push(tag.to_string());
    entry.modified_at = chrono::Utc::now();
    vault.dirty = true;

    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Tag '{}' added to '{}'",
        "✓".green().bold(),
        tag.cyan(),
        title.bold()
    );
    Ok(())
}

pub fn run_remove(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    tag: &str,
) -> anyhow::Result<()> {
    let entry = vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_prefix))?;

    let uuid = entry.uuid;
    let title = entry.title.get().to_string();

    let entry = vault
        .get_entry_mut(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;
    let before = entry.tags.len();
    entry.tags.retain(|t| !t.eq_ignore_ascii_case(tag));

    if entry.tags.len() == before {
        anyhow::bail!("Tag '{}' not found on '{}'", tag, title);
    }

    entry.modified_at = chrono::Utc::now();
    vault.dirty = true;

    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Tag '{}' removed from '{}'",
        "✓".green().bold(),
        tag.cyan(),
        title.bold()
    );
    Ok(())
}

pub fn run_entries(vault: &Vault, tag: &str, format: &str) -> anyhow::Result<()> {
    let entries: Vec<_> = vault
        .all_entries()
        .filter(|e| e.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
        .collect();

    if entries.is_empty() {
        println!("{}", format!("No entries with tag '{}'.", tag).dimmed());
        return Ok(());
    }

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| serde_json::json!({ "uuid": e.uuid.to_string(), "title": e.title.get(), "username": e.username.get() }))
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!(
                "{} entries with tag '{}':",
                entries.len().to_string().bold(),
                tag.cyan()
            );
            for e in &entries {
                println!(
                    "  {} {}",
                    e.uuid.to_string()[..8].dimmed(),
                    e.title.get().bold()
                );
            }
        }
    }

    Ok(())
}

pub fn run_rename(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    old_tag: &str,
    new_tag: &str,
) -> anyhow::Result<()> {
    let mut count = 0;
    let uuids: Vec<_> = vault
        .all_entries()
        .filter(|e| e.tags.iter().any(|t| t.eq_ignore_ascii_case(old_tag)))
        .map(|e| e.uuid)
        .collect();

    for uuid in &uuids {
        if let Some(entry) = vault.get_entry_mut(uuid) {
            for tag in &mut entry.tags {
                if tag.eq_ignore_ascii_case(old_tag) {
                    *tag = new_tag.to_string();
                    count += 1;
                }
            }
            entry.modified_at = chrono::Utc::now();
        }
    }

    if count == 0 {
        anyhow::bail!("Tag '{}' not found", old_tag);
    }

    vault.dirty = true;
    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Tag '{}' renamed to '{}' on {} entries",
        "✓".green().bold(),
        old_tag.dimmed(),
        new_tag.cyan(),
        count.to_string().bold()
    );
    Ok(())
}

fn save_vault(vault: &Vault, vault_path: &str, master_password: &str) -> anyhow::Result<()> {
    use keepassex_core::vault::operations::{save_vault as core_save, VaultCredentials};
    use std::path::Path;
    let credentials = VaultCredentials::password_only(master_password);
    let rt = tokio::runtime::Handle::current();
    rt.block_on(core_save(vault, Path::new(vault_path), &credentials))
        .map_err(|e| anyhow::anyhow!("Failed to save vault: {}", e))
}
