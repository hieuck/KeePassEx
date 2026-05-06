//! CLI `breach` command — check vault passwords against HIBP

use keepassex_core::{Vault, breach};
use colored::Colorize;

pub async fn run(vault: &Vault, online: bool, format: &str) -> anyhow::Result<()> {
    let mode = if online { "online (HIBP)" } else { "offline" };
    eprintln!("{} Checking passwords {} ...", "🔍".bold(), mode.dimmed());

    let entries: Vec<(String, String, String)> = vault
        .all_entries()
        .filter(|e| !e.password.get().is_empty())
        .map(|e| (
            e.uuid.to_string(),
            e.title.get().to_string(),
            e.password.get().to_string(),
        ))
        .collect();

    let total = entries.len();
    let mut breached = Vec::new();

    if online {
        // Online HIBP check
        let passwords: Vec<(String, String)> = entries.iter()
            .map(|(uuid, _, pw)| (uuid.clone(), pw.clone()))
            .collect();

        let results = breach::check_vault_passwords(&passwords).await;

        for (uuid, result) in results {
            if result.is_breached {
                let title = entries.iter()
                    .find(|(u, _, _)| u == &uuid)
                    .map(|(_, t, _)| t.as_str())
                    .unwrap_or("Unknown");
                breached.push((title.to_string(), result.breach_count));
            }
        }
    } else {
        // Offline check
        for (_, title, password) in &entries {
            if breach::check_password_offline(password) {
                breached.push((title.clone(), 0u64));
            }
        }
    }

    if format == "json" {
        let json = serde_json::json!({
            "total_checked": total,
            "breached_count": breached.len(),
            "mode": mode,
            "breached": breached.iter().map(|(t, c)| serde_json::json!({
                "title": t,
                "breach_count": c,
            })).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json)?);
        return Ok(());
    }

    println!("\n{} Breach Check Results", "🛡️".bold());
    println!("{}", "─".repeat(50));
    println!("{:<25} {}", "Passwords checked:", total.to_string().bold());
    println!("{:<25} {}", "Mode:", mode.dimmed());
    println!();

    if breached.is_empty() {
        println!("{} No breached passwords found!", "✓".bright_green().bold());
        println!("{}", "Your passwords are not in known data breaches.".dimmed());
    } else {
        println!(
            "{} {} password(s) found in data breaches!",
            "⚠".red().bold(),
            breached.len().to_string().red().bold()
        );
        println!();
        for (title, count) in &breached {
            if *count > 0 {
                println!(
                    "  {} {} — seen {} times in breaches",
                    "•".red(),
                    title.bold(),
                    count.to_string().red()
                );
            } else {
                println!(
                    "  {} {} — found in common password list",
                    "•".red(),
                    title.bold()
                );
            }
        }
        println!();
        println!("{}", "Change these passwords immediately!".yellow());
    }

    Ok(())
}
