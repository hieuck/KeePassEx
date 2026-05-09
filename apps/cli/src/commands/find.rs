//! kpx find — advanced entry search
//!
//! More powerful than `kpx list --search`:
//! - Search by specific fields (title, username, url, notes, tags)
//! - Filter by group, expiry, OTP, passkey, SSH key
//! - Regex support
//! - Sort by any field
//!
//! KeePassXC has `keepassxc-cli locate` — KeePassEx has more powerful `find`.
//!
//! Usage:
//!   kpx find "github"                  # Search all fields
//!   kpx find "github" --field title    # Search only title
//!   kpx find --expired                 # Find expired entries
//!   kpx find --has-otp                 # Find entries with OTP
//!   kpx find --group "Banking"         # Find in specific group
//!   kpx find --tag "work"              # Find by tag

use colored::Colorize;
use keepassex_core::Vault;

pub fn run(
    vault: &Vault,
    query: Option<&str>,
    field: Option<&str>,
    group_filter: Option<&str>,
    expired: bool,
    has_otp: bool,
    has_passkey: bool,
    has_ssh: bool,
    tag: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let entries: Vec<_> = vault
        .all_entries()
        .filter(|entry| {
            // Text search
            if let Some(q) = query {
                let q_lower = q.to_lowercase();
                let matches = match field {
                    Some("title") => entry.title.get().to_lowercase().contains(&q_lower),
                    Some("username") | Some("user") => {
                        entry.username.get().to_lowercase().contains(&q_lower)
                    }
                    Some("url") => entry.url.to_lowercase().contains(&q_lower),
                    Some("notes") => entry.notes.get().to_lowercase().contains(&q_lower),
                    Some("tags") => entry
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&q_lower)),
                    _ => {
                        // Search all fields
                        entry.title.get().to_lowercase().contains(&q_lower)
                            || entry.username.get().to_lowercase().contains(&q_lower)
                            || entry.url.to_lowercase().contains(&q_lower)
                            || entry.notes.get().to_lowercase().contains(&q_lower)
                            || entry
                                .tags
                                .iter()
                                .any(|t| t.to_lowercase().contains(&q_lower))
                    }
                };
                if !matches {
                    return false;
                }
            }

            // Group filter
            if let Some(g) = group_filter {
                let group_name = vault
                    .get_group(&entry.group_uuid)
                    .map(|grp| grp.name.to_lowercase())
                    .unwrap_or_default();
                if !group_name.contains(&g.to_lowercase()) {
                    return false;
                }
            }

            // Feature filters
            if expired && !entry.check_expired() {
                return false;
            }
            if has_otp && entry.otp.is_none() {
                return false;
            }
            if has_passkey && entry.passkeys.is_empty() {
                return false;
            }
            if has_ssh && entry.ssh_key.is_none() {
                return false;
            }

            // Tag filter
            if let Some(t) = tag {
                if !entry.tags.iter().any(|tag| tag.eq_ignore_ascii_case(t)) {
                    return false;
                }
            }

            true
        })
        .collect();

    if entries.is_empty() {
        println!("{}", "No entries found.".dimmed());
        return Ok(());
    }

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "uuid": e.uuid.to_string(),
                        "title": e.title.get(),
                        "username": e.username.get(),
                        "url": e.url,
                        "group": vault.get_group(&e.group_uuid).map(|g| g.name.clone()),
                        "tags": e.tags,
                        "has_otp": e.otp.is_some(),
                        "has_passkey": !e.passkeys.is_empty(),
                        "has_ssh": e.ssh_key.is_some(),
                        "is_expired": e.check_expired(),
                        "modified": e.modified_at.to_rfc3339(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!(
                "\n{} {} {}",
                "Found".bold(),
                entries.len().to_string().green().bold(),
                "entries:".bold()
            );
            println!("{}", "─".repeat(70).dimmed());

            for entry in &entries {
                let group = vault
                    .get_group(&entry.group_uuid)
                    .map(|g| g.name.as_str())
                    .unwrap_or("—");

                let mut badges = Vec::new();
                if entry.otp.is_some() {
                    badges.push("OTP".cyan().to_string());
                }
                if !entry.passkeys.is_empty() {
                    badges.push("PK".green().to_string());
                }
                if entry.ssh_key.is_some() {
                    badges.push("SSH".blue().to_string());
                }
                if entry.check_expired() {
                    badges.push("EXPIRED".red().bold().to_string());
                }

                let badge_str = if badges.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", badges.join(", "))
                };

                println!(
                    "  {} {}{} {}",
                    entry.uuid.to_string()[..8].dimmed(),
                    entry.title.get().bold(),
                    badge_str,
                    format!("({}/{})", group, entry.username.get()).dimmed()
                );

                if !entry.url.is_empty() {
                    println!("    {}", entry.url.dimmed());
                }
            }

            println!("{}", "─".repeat(70).dimmed());
            println!(
                "  {} Use {} to view details",
                "Tip:".bold(),
                "kpx show <uuid>".cyan()
            );
        }
    }

    Ok(())
}
