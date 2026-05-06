//! Get entry command

use keepassex_core::Vault;
use colored::Colorize;
use uuid::Uuid;

pub fn run(vault: &Vault, uuid_str: &str, field: &str, copy: bool) -> anyhow::Result<()> {
    // Support partial UUID match
    let uuid = if uuid_str.len() == 36 {
        Uuid::parse_str(uuid_str)?
    } else {
        // Find by partial UUID prefix
        vault
            .all_entries()
            .find(|e| e.uuid.to_string().starts_with(uuid_str))
            .map(|e| e.uuid)
            .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_str))?
    };

    let entry = vault
        .get_entry(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;

    let value = match field {
        "password" | "pw" => entry.password.get().to_string(),
        "username" | "user" | "u" => entry.username.get().to_string(),
        "url" => entry.url.clone(),
        "notes" => entry.notes.get().to_string(),
        "title" => entry.title.get().to_string(),
        _ => {
            // Print all fields
            println!("{}", "─".repeat(50));
            println!("{}: {}", "Title".bold(), entry.title.get());
            println!("{}: {}", "Username".bold(), entry.username.get());
            println!("{}: {}", "Password".bold(), "••••••••".dimmed());
            println!("{}: {}", "URL".bold(), entry.url);
            if !entry.notes.get().is_empty() {
                println!("{}: {}", "Notes".bold(), entry.notes.get());
            }
            if entry.otp.is_some() {
                println!("{}: {}", "OTP".bold(), "configured".green());
            }
            if !entry.passkeys.is_empty() {
                println!("{}: {} passkey(s)", "Passkeys".bold(), entry.passkeys.len());
            }
            println!("{}: {}", "Modified".bold(), entry.modified_at.format("%Y-%m-%d %H:%M"));
            println!("{}", "─".repeat(50));
            return Ok(());
        }
    };

    if copy {
        // Copy to clipboard
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let mut child = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn()?;
            use std::io::Write;
            child.stdin.as_mut().unwrap().write_all(value.as_bytes())?;
            child.wait()?;
            eprintln!("{}", "Copied to clipboard".green());
        }
        #[cfg(not(target_os = "macos"))]
        {
            println!("{}", value);
        }
    } else {
        println!("{}", value);
    }

    Ok(())
}
