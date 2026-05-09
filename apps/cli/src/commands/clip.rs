//! kpx clip — copy entry field to clipboard with auto-clear
//!
//! Faster than `kpx get <uuid> --field password --copy`.
//! Supports: password, username, url, otp, notes, custom fields.
//!
//! Usage:
//!   kpx clip <uuid>                    # Copy password (default)
//!   kpx clip <uuid> --field username   # Copy username
//!   kpx clip <uuid> --field otp        # Copy current OTP code
//!   kpx clip <uuid> --clear 30         # Clear after 30 seconds (default: 10)

use colored::Colorize;
use keepassex_core::Vault;

pub fn run(vault: &Vault, uuid_prefix: &str, field: &str, clear_after: u64) -> anyhow::Result<()> {
    // Find entry by UUID prefix
    let entry = vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_prefix))?;

    let value = match field.to_lowercase().as_str() {
        "password" | "pw" | "pass" => {
            let pw = entry.password.get().to_string();
            if pw.is_empty() {
                anyhow::bail!("Entry '{}' has no password", entry.title.get());
            }
            pw
        }
        "username" | "user" | "u" => {
            let u = entry.username.get().to_string();
            if u.is_empty() {
                anyhow::bail!("Entry '{}' has no username", entry.title.get());
            }
            u
        }
        "url" => {
            if entry.url.is_empty() {
                anyhow::bail!("Entry '{}' has no URL", entry.title.get());
            }
            entry.url.clone()
        }
        "otp" | "totp" => {
            let otp_config = entry.otp.as_ref().ok_or_else(|| {
                anyhow::anyhow!("Entry '{}' has no OTP configured", entry.title.get())
            })?;
            let code = keepassex_core::otp::generate_totp(otp_config)
                .map_err(|e| anyhow::anyhow!("OTP error: {}", e))?;
            code.code
        }
        "notes" => {
            let n = entry.notes.get().to_string();
            if n.is_empty() {
                anyhow::bail!("Entry '{}' has no notes", entry.title.get());
            }
            n
        }
        custom => {
            // Try custom field
            entry
                .custom_fields
                .get(custom)
                .map(|f| f.value.get().to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Field '{}' not found in entry '{}'",
                        custom,
                        entry.title.get()
                    )
                })?
        }
    };

    // Copy to clipboard
    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| anyhow::anyhow!("Clipboard error: {}", e))?;
    clipboard
        .set_text(&value)
        .map_err(|e| anyhow::anyhow!("Clipboard write error: {}", e))?;

    let field_lower = field.to_lowercase();
    let field_display = match field_lower.as_str() {
        "password" | "pw" | "pass" => "password",
        "username" | "user" | "u" => "username",
        "otp" | "totp" => "OTP code",
        other => other,
    };

    println!(
        "{} {} for '{}' copied to clipboard",
        "✓".green().bold(),
        field_display.cyan(),
        entry.title.get().bold()
    );

    if clear_after > 0 {
        println!(
            "  {} Clipboard will be cleared in {}s",
            "→".dimmed(),
            clear_after.to_string().yellow()
        );

        // Spawn background thread to clear clipboard
        let clear_secs = clear_after;
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(clear_secs));
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text("");
            }
        });

        // Keep process alive for the clear duration
        // (only if running interactively — check if stdout is a tty)
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            if unsafe { libc::isatty(std::io::stdout().as_raw_fd()) } == 1 {
                std::thread::sleep(std::time::Duration::from_secs(clear_secs));
                println!("  {} Clipboard cleared.", "✓".green());
            }
        }
        #[cfg(not(unix))]
        {
            std::thread::sleep(std::time::Duration::from_secs(clear_secs));
            println!("  {} Clipboard cleared.", "✓".green());
        }
    }

    Ok(())
}
