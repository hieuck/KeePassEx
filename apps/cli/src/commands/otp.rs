//! OTP command

use keepassex_core::{Vault, otp};
use colored::Colorize;
use uuid::Uuid;

pub async fn run(vault: &Vault, uuid_str: &str, watch: bool) -> anyhow::Result<()> {
    let uuid = Uuid::parse_str(uuid_str)
        .or_else(|_| {
            vault
                .all_entries()
                .find(|e| e.uuid.to_string().starts_with(uuid_str))
                .map(|e| Ok(e.uuid))
                .unwrap_or_else(|| Err(anyhow::anyhow!("Entry not found")))
        })?;

    let entry = vault
        .get_entry(&uuid)
        .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;

    let otp_config = entry
        .otp
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Entry has no OTP configured"))?;

    loop {
        let code = otp::generate_totp(otp_config)?;

        // Clear line and print
        print!("\r");
        print!(
            "{} {} [{}s] ",
            entry.title.get().bold(),
            code.code.bright_cyan().bold(),
            code.remaining_seconds.to_string().yellow()
        );

        // Progress bar
        let bar_width = 20;
        let filled = (code.progress() * bar_width as f32) as usize;
        let bar: String = "█".repeat(filled) + &"░".repeat(bar_width - filled);
        print!("{}", bar.dimmed());

        use std::io::Write;
        std::io::stdout().flush()?;

        if !watch {
            println!();
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
