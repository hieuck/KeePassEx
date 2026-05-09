//! kpx passwd — change entry password
//!
//! KeePassXC has `keepassxc-cli edit --generate-password` — KeePassEx has dedicated passwd.
//!
//! Usage:
//!   kpx passwd <uuid>                    # Prompt for new password
//!   kpx passwd <uuid> --generate         # Auto-generate strong password
//!   kpx passwd <uuid> --generate --length 24  # Generate with custom length

use colored::Colorize;
use keepassex_core::generator::PasswordGenerator;
use keepassex_core::types::PasswordGeneratorConfig;
use keepassex_core::Vault;

pub async fn run(
    vault: &mut Vault,
    vault_path: &str,
    master_password: &str,
    uuid_prefix: &str,
    generate: bool,
    length: usize,
) -> anyhow::Result<()> {
    let entry = vault
        .all_entries()
        .find(|e| e.uuid.to_string().starts_with(uuid_prefix))
        .ok_or_else(|| anyhow::anyhow!("Entry not found: {}", uuid_prefix))?;

    let uuid = entry.uuid;
    let title = entry.title.get().to_string();

    let new_password = if generate {
        let config = PasswordGeneratorConfig {
            length,
            use_symbols: true,
            ..Default::default()
        };
        let pw = PasswordGenerator::generate(&config).map_err(|e| anyhow::anyhow!("{}", e))?;
        let entropy = PasswordGenerator::estimate_entropy(&pw);
        println!(
            "{} Generated password for '{}': {} ({:.0} bits entropy)",
            "⚡".bold(),
            title.bold(),
            pw.yellow(),
            entropy
        );
        pw
    } else {
        // Prompt for new password
        let pw1 = rpassword::prompt_password("New password: ")?;
        if pw1.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }
        let pw2 = rpassword::prompt_password("Confirm password: ")?;
        if pw1 != pw2 {
            anyhow::bail!("Passwords do not match");
        }
        pw1
    };

    // Update entry
    let entry = vault
        .get_entry_mut(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;

    entry.password.set(new_password.clone());
    entry.modified_at = chrono::Utc::now();

    // Save vault
    use keepassex_core::vault::operations::{save_vault, VaultCredentials};
    use std::path::Path;
    let credentials = VaultCredentials::password_only(master_password);
    save_vault(vault, Path::new(vault_path), &credentials)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save vault: {}", e))?;

    println!(
        "{} Password for '{}' updated and vault saved.",
        "✓".green().bold(),
        title.bold()
    );

    // Copy to clipboard
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(&new_password);
        println!(
            "  {} New password copied to clipboard (clears in 10s)",
            "→".dimmed()
        );
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(10));
            if let Ok(mut cb) = arboard::Clipboard::new() {
                let _ = cb.set_text("");
            }
        });
    }

    Ok(())
}
