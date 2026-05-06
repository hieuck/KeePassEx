//! Health audit command

use keepassex_core::{Vault, health};
use colored::Colorize;

pub fn run(vault: &Vault, format: &str) -> anyhow::Result<()> {
    let report = health::audit_vault(vault);

    if format == "json" {
        println!("{}", serde_json::json!({
            "score": report.score,
            "total_entries": report.total_entries,
            "weak_passwords": report.weak_passwords.len(),
            "reused_passwords": report.reused_passwords.len(),
            "expired_entries": report.expired_entries.len(),
            "expiring_soon": report.expiring_soon.len(),
        }));
        return Ok(());
    }

    // Score display
    let score_colored = match report.score {
        90..=100 => format!("{}/100", report.score).bright_green().bold(),
        70..=89 => format!("{}/100", report.score).green().bold(),
        50..=69 => format!("{}/100", report.score).yellow().bold(),
        _ => format!("{}/100", report.score).red().bold(),
    };

    println!("\n{} Vault Health Report", "🛡️".bold());
    println!("Score: {}", score_colored);
    println!("Total entries: {}", report.total_entries);
    println!();

    if report.weak_passwords.is_empty()
        && report.reused_passwords.is_empty()
        && report.expired_entries.is_empty()
    {
        println!("{}", "✓ No issues found! Your vault is healthy.".bright_green());
        return Ok(());
    }

    if !report.weak_passwords.is_empty() {
        println!("{} Weak passwords ({})", "⚠".yellow(), report.weak_passwords.len());
        for w in &report.weak_passwords {
            println!("  • {} — {}", w.entry_title.bold(), w.strength_label.red());
        }
        println!();
    }

    if !report.reused_passwords.is_empty() {
        println!("{} Reused passwords ({} groups)", "⚠".yellow(), report.reused_passwords.len());
        for group in &report.reused_passwords {
            let titles: Vec<&str> = group.entries.iter().map(|e| e.title.as_str()).collect();
            println!("  • {}", titles.join(", ").bold());
        }
        println!();
    }

    if !report.expired_entries.is_empty() {
        println!("{} Expired entries ({})", "✗".red(), report.expired_entries.len());
        for e in &report.expired_entries {
            println!("  • {} — expired {}", e.entry_title.bold(), e.expired_at.red());
        }
        println!();
    }

    if !report.expiring_soon.is_empty() {
        println!("{} Expiring soon ({})", "⏰".yellow(), report.expiring_soon.len());
        for e in &report.expiring_soon {
            println!(
                "  • {} — {} days remaining",
                e.entry_title.bold(),
                e.days_remaining.to_string().yellow()
            );
        }
    }

    Ok(())
}
