//! kpx show — display entry details in terminal
//!
//! KeePassXC has `keepassxc-cli show` — KeePassEx now matches it.
//! More powerful: supports field selection, JSON output, OTP display.
//!
//! Usage:
//!   kpx show <uuid>                    # Show all fields (password hidden)
//!   kpx show <uuid> --show-password    # Show password in plaintext
//!   kpx show <uuid> --field password   # Show only password
//!   kpx show <uuid> --format json      # JSON output

use colored::Colorize;
use keepassex_core::Vault;

pub fn run(
    vault: &Vault,
    uuid_prefix: &str,
    show_password: bool,
    field: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let entry = vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_prefix))?;

    // Single field mode
    if let Some(f) = field {
        let value = match f.to_lowercase().as_str() {
            "title" => entry.title.get().to_string(),
            "username" | "user" => entry.username.get().to_string(),
            "password" | "pw" => {
                if show_password {
                    entry.password.get().to_string()
                } else {
                    "*** (use --show-password to reveal)".to_string()
                }
            }
            "url" => entry.url.clone(),
            "notes" => entry.notes.get().to_string(),
            "uuid" => entry.uuid.to_string(),
            "group" => vault
                .get_group(&entry.group_uuid)
                .map(|g| g.name.clone())
                .unwrap_or_default(),
            other => entry
                .custom_fields
                .get(other)
                .map(|f| f.value.get().to_string())
                .ok_or_else(|| anyhow::anyhow!("Field '{}' not found", other))?,
        };
        println!("{}", value);
        return Ok(());
    }

    // Full entry display
    match format {
        "json" => {
            let mut obj = serde_json::json!({
                "uuid": entry.uuid.to_string(),
                "title": entry.title.get(),
                "username": entry.username.get(),
                "url": entry.url,
                "notes": entry.notes.get(),
                "has_otp": entry.otp.is_some(),
                "has_passkey": !entry.passkeys.is_empty(),
                "has_ssh_key": entry.ssh_key.is_some(),
                "tags": entry.tags,
                "created_at": entry.created_at.to_rfc3339(),
                "modified_at": entry.modified_at.to_rfc3339(),
                "group": vault.get_group(&entry.group_uuid).map(|g| g.name.clone()),
            });
            if show_password {
                obj["password"] = serde_json::Value::String(entry.password.get().to_string());
            }
            if !entry.custom_fields.is_empty() {
                let cf: serde_json::Map<String, serde_json::Value> = entry
                    .custom_fields
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            serde_json::Value::String(v.value.get().to_string()),
                        )
                    })
                    .collect();
                obj["custom_fields"] = serde_json::Value::Object(cf);
            }
            println!("{}", serde_json::to_string_pretty(&obj)?);
        }
        _ => {
            let group_name = vault
                .get_group(&entry.group_uuid)
                .map(|g| g.name.as_str())
                .unwrap_or("—");

            println!("\n{}", entry.title.get().bold().underline());
            println!("{}", "─".repeat(50).dimmed());

            print_field("UUID", &entry.uuid.to_string()[..8], false);
            print_field("Group", group_name, false);
            print_field("Username", entry.username.get(), false);

            if show_password {
                print_field("Password", entry.password.get(), true);
            } else if !entry.password.get().is_empty() {
                print_field("Password", "*** (use --show-password)", false);
            }

            if !entry.url.is_empty() {
                print_field("URL", &entry.url, false);
            }

            if entry.otp.is_some() {
                // Generate current OTP
                if let Ok(code) = keepassex_core::otp::generate_totp(entry.otp.as_ref().unwrap()) {
                    print_field(
                        "OTP",
                        &format!("{} ({}s remaining)", code.code, code.remaining_seconds),
                        false,
                    );
                }
            }

            if !entry.tags.is_empty() {
                print_field("Tags", &entry.tags.join(", "), false);
            }

            if !entry.custom_fields.is_empty() {
                println!("\n{}", "Custom Fields:".bold());
                for (key, field) in &entry.custom_fields {
                    if field.value.protected {
                        print_field(key, "*** (protected)", false);
                    } else {
                        print_field(key, field.value.get(), false);
                    }
                }
            }

            if !entry.notes.get().is_empty() {
                println!("\n{}", "Notes:".bold());
                println!("{}", entry.notes.get().dimmed());
            }

            println!("\n{}", "─".repeat(50).dimmed());
            println!(
                "Created: {}  Modified: {}",
                entry.created_at.format("%Y-%m-%d").to_string().dimmed(),
                entry.modified_at.format("%Y-%m-%d").to_string().dimmed()
            );

            if !entry.passkeys.is_empty() {
                println!("🔑 {} passkey(s)", entry.passkeys.len());
            }
            if entry.ssh_key.is_some() {
                println!(
                    "🔐 SSH key: {}",
                    entry.ssh_key.as_ref().unwrap().fingerprint.dimmed()
                );
            }
        }
    }

    Ok(())
}

fn print_field(label: &str, value: &str, sensitive: bool) {
    let label_colored = format!("{:<12}", label).cyan();
    let value_colored = if sensitive {
        value.yellow().to_string()
    } else {
        value.to_string()
    };
    println!("{} {}", label_colored, value_colored);
}
