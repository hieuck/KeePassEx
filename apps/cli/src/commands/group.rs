//! kpx group — manage vault groups (folders)
//!
//! KeePassXC has group management in CLI; KeePassEx now matches it.
//!
//! Usage:
//!   kpx group list                          # List all groups
//!   kpx group create "Banking"              # Create group in root
//!   kpx group create "Cards" --parent <uuid> # Create in parent group
//!   kpx group rename <uuid> "New Name"      # Rename group
//!   kpx group delete <uuid>                 # Delete (move to recycle bin)
//!   kpx group delete <uuid> --permanent     # Permanently delete
//!   kpx group move <uuid> --parent <uuid>   # Move to different parent

use colored::Colorize;
use keepassex_core::Vault;

pub fn run_list(vault: &Vault, format: &str) -> anyhow::Result<()> {
    let groups: Vec<_> = vault.all_groups().collect();

    if groups.is_empty() {
        println!("{}", "No groups found.".dimmed());
        return Ok(());
    }

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = groups
                .iter()
                .map(|g| {
                    serde_json::json!({
                        "uuid": g.uuid.to_string(),
                        "name": g.name,
                        "parent_uuid": g.parent_uuid.map(|u| u.to_string()),
                        "entry_count": g.entry_count,
                        "child_group_count": g.child_group_count,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("{}", "Groups".bold());
            println!("{}", "─".repeat(60).dimmed());

            // Build tree display
            let root_uuid = vault.root_group_uuid;
            print_group_tree(vault, &root_uuid.to_string(), 0, &groups);

            println!("{}", "─".repeat(60).dimmed());
            println!("{} groups total", groups.len().to_string().bold());
        }
    }

    Ok(())
}

fn print_group_tree(
    vault: &Vault,
    group_uuid: &str,
    depth: usize,
    all_groups: &[&keepassex_core::types::Group],
) {
    if let Ok(uuid) = uuid::Uuid::parse_str(group_uuid) {
        if let Some(group) = vault.get_group(&uuid) {
            let indent = "  ".repeat(depth);
            let icon = if depth == 0 { "🏠" } else { "📁" };
            let entry_count = vault.get_group_entries(&uuid).len();

            println!(
                "{}{} {} {} {}",
                indent,
                icon,
                group.name.bold(),
                format!("({})", entry_count).dimmed(),
                group_uuid[..8].dimmed()
            );

            // Print children
            for child in all_groups {
                if child.parent_uuid == Some(uuid) {
                    print_group_tree(vault, &child.uuid.to_string(), depth + 1, all_groups);
                }
            }
        }
    }
}

pub fn run_create(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    name: &str,
    parent_uuid: Option<&str>,
) -> anyhow::Result<()> {
    let parent = if let Some(p) = parent_uuid {
        uuid::Uuid::parse_str(p).map_err(|_| anyhow::anyhow!("Invalid parent UUID: {}", p))?
    } else {
        vault.root_group_uuid
    };

    let group_uuid = vault
        .create_group(name, parent)
        .map_err(|e| anyhow::anyhow!("Failed to create group: {}", e))?;

    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Group '{}' created ({})",
        "✓".green().bold(),
        name.bold(),
        group_uuid.to_string()[..8].dimmed()
    );

    Ok(())
}

pub fn run_rename(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    new_name: &str,
) -> anyhow::Result<()> {
    let uuid = vault
        .all_groups()
        .find(|g| g.uuid.to_string().starts_with(uuid_prefix))
        .map(|g| g.uuid)
        .ok_or_else(|| anyhow::anyhow!("Group not found: {}", uuid_prefix))?;

    let old_name = vault
        .get_group(&uuid)
        .map(|g| g.name.clone())
        .unwrap_or_default();

    // Rename via get_group_mut
    if let Some(group) = vault.get_group_mut(&uuid) {
        group.name = new_name.to_string();
        group.modified_at = chrono::Utc::now();
    } else {
        anyhow::bail!("Group not found: {}", uuid_prefix);
    }
    vault.dirty = true;

    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Group '{}' renamed to '{}'",
        "✓".green().bold(),
        old_name.dimmed(),
        new_name.bold()
    );

    Ok(())
}

pub fn run_delete(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    permanent: bool,
    force: bool,
) -> anyhow::Result<()> {
    let group = vault
        .all_groups()
        .find(|g| g.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Group not found: {}", uuid_prefix))?;

    let uuid = group.uuid;
    let name = group.name.clone();
    let entry_count = vault.get_group_entries_recursive(&uuid).len();

    if !force {
        if entry_count > 0 {
            eprintln!(
                "{} Group '{}' contains {} entries.",
                "⚠".yellow(),
                name.bold(),
                entry_count.to_string().yellow()
            );
        }
        eprint!("Delete group '{}'? [y/N] ", name.bold());
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    vault
        .delete_group(&uuid, permanent)
        .map_err(|e| anyhow::anyhow!("Failed to delete group: {}", e))?;

    save_vault(vault, vault_path, master_password)?;

    println!(
        "{} Group '{}' {}",
        "✓".green().bold(),
        name.bold(),
        if permanent {
            "permanently deleted"
        } else {
            "moved to recycle bin"
        }
    );

    Ok(())
}

pub fn run_move(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    new_parent_uuid: &str,
) -> anyhow::Result<()> {
    let group = vault
        .all_groups()
        .find(|g| g.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Group not found: {}", uuid_prefix))?;

    let uuid = group.uuid;
    let name = group.name.clone();

    let parent_uuid = uuid::Uuid::parse_str(new_parent_uuid)
        .map_err(|_| anyhow::anyhow!("Invalid parent UUID: {}", new_parent_uuid))?;

    vault
        .move_group(&uuid, parent_uuid)
        .map_err(|e| anyhow::anyhow!("Failed to move group: {}", e))?;

    save_vault(vault, vault_path, master_password)?;

    println!("{} Group '{}' moved", "✓".green().bold(), name.bold());

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
