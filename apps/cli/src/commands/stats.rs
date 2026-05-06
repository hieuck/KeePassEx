//! Vault statistics command

use keepassex_core::Vault;
use colored::Colorize;

pub fn run(vault: &Vault) -> anyhow::Result<()> {
    let entries: Vec<_> = vault.all_entries().collect();
    let groups: Vec<_> = vault.all_groups().collect();

    let with_otp = entries.iter().filter(|e| e.otp.is_some()).count();
    let with_passkey = entries.iter().filter(|e| !e.passkeys.is_empty()).count();
    let with_ssh = entries.iter().filter(|e| e.ssh_key.is_some()).count();
    let expired = entries.iter().filter(|e| e.is_expired()).count();
    let with_attachments = entries.iter().filter(|e| !e.attachments.is_empty()).count();

    println!("\n{}", "📊 Vault Statistics".bold());
    println!("{}", "─".repeat(40));
    println!("{:<25} {}", "Total entries:", entries.len().to_string().bold());
    println!("{:<25} {}", "Total groups:", groups.len().to_string().bold());
    println!("{:<25} {}", "With OTP:", with_otp.to_string().cyan());
    println!("{:<25} {}", "With Passkey:", with_passkey.to_string().cyan());
    println!("{:<25} {}", "With SSH key:", with_ssh.to_string().cyan());
    println!("{:<25} {}", "With attachments:", with_attachments.to_string().cyan());
    println!("{:<25} {}", "Expired:", expired.to_string().red());
    println!("{}", "─".repeat(40));
    println!("{:<25} {}", "Vault name:", vault.meta.name.bold());
    println!("{:<25} {}", "Last modified:", vault.meta.modified_at.format("%Y-%m-%d %H:%M").to_string().dimmed());

    Ok(())
}
