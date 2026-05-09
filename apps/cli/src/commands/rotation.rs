//! kpx rotation — show password rotation recommendations
//!
//! KeePassEx exclusive: proactive rotation engine with category-aware schedules.
//! No competitor (KeePass, KeePassXC, Keepassium, KeePass2Android) has this.
//!
//! Usage:
//!   kpx rotation                    # Show all recommendations
//!   kpx rotation --urgency overdue  # Filter by urgency
//!   kpx rotation --format json      # JSON output

use colored::Colorize;
use keepassex_core::expiry_engine::{analyze_vault_rotations, ExpiryInput, RotationUrgency};
use keepassex_core::Vault;

pub fn run(vault: &Vault, urgency_filter: Option<&str>, format: &str) -> anyhow::Result<()> {
    // Build inputs from all vault entries
    let inputs: Vec<ExpiryInput> = vault
        .all_entries()
        .map(|entry| {
            let tags_str = entry.tags.join(" ");
            let category_hint = {
                let result = keepassex_core::categorizer::categorize_entry(
                    entry.title.get(),
                    &entry.url,
                    &tags_str,
                );
                Some(format!("{:?}", result.category).to_lowercase())
            };

            ExpiryInput {
                uuid: entry.uuid.to_string(),
                title: entry.title.get().to_string(),
                password_modified_at: entry.modified_at,
                explicit_expiry: entry.expiry,
                category_hint,
                has_password: !entry.password.get().is_empty(),
            }
        })
        .collect();

    let mut recommendations = analyze_vault_rotations(&inputs);

    // Apply urgency filter
    if let Some(filter) = urgency_filter {
        recommendations.retain(|r| {
            let urgency_str = format!("{:?}", r.urgency).to_lowercase();
            urgency_str == filter.to_lowercase()
        });
    }

    if recommendations.is_empty() {
        println!("{}", "✅ No password rotation needed.".green());
        println!(
            "{}",
            "All passwords are within their recommended rotation schedule.".dimmed()
        );
        return Ok(());
    }

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = recommendations
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "uuid": r.entry_uuid,
                        "title": r.entry_title,
                        "urgency": format!("{:?}", r.urgency).to_lowercase(),
                        "age_days": r.age_days,
                        "recommended_max_days": r.recommended_max_days,
                        "days_until_overdue": r.days_until_overdue,
                        "message": r.message_en,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("{}", "🔄 Password Rotation Recommendations".bold());
            println!("{}", "─".repeat(70).dimmed());
            println!(
                "{:<30} {:<12} {:<8} {}",
                "Entry".bold(),
                "Urgency".bold(),
                "Age".bold(),
                "Message".bold()
            );
            println!("{}", "─".repeat(70).dimmed());

            for rec in &recommendations {
                let urgency_colored = match rec.urgency {
                    RotationUrgency::Expired => "EXPIRED".red().bold().to_string(),
                    RotationUrgency::Overdue => "OVERDUE".red().to_string(),
                    RotationUrgency::Soon => "SOON".yellow().to_string(),
                    RotationUrgency::Aging => "AGING".cyan().to_string(),
                    RotationUrgency::Fresh => "FRESH".green().to_string(),
                };

                let title = rec.entry_title.chars().take(28).collect::<String>();
                let age = format!("{}d", rec.age_days);

                println!(
                    "{:<30} {:<22} {:<8} {}",
                    title,
                    urgency_colored,
                    age.dimmed(),
                    rec.message_en.dimmed()
                );
            }

            println!("{}", "─".repeat(70).dimmed());

            // Summary
            let overdue = recommendations
                .iter()
                .filter(|r| {
                    r.urgency == RotationUrgency::Overdue || r.urgency == RotationUrgency::Expired
                })
                .count();
            let soon = recommendations
                .iter()
                .filter(|r| r.urgency == RotationUrgency::Soon)
                .count();
            let aging = recommendations
                .iter()
                .filter(|r| r.urgency == RotationUrgency::Aging)
                .count();

            if overdue > 0 {
                println!("  {} overdue/expired", overdue.to_string().red().bold());
            }
            if soon > 0 {
                println!("  {} due soon", soon.to_string().yellow());
            }
            if aging > 0 {
                println!("  {} aging", aging.to_string().cyan());
            }
            println!(
                "\n  {} Run {} to change a password.",
                "Tip:".bold(),
                "kpx edit <uuid>".cyan()
            );
        }
    }

    Ok(())
}
