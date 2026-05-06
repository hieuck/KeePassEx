//! CLI — Natural Language Search command
//!
//! kpx search "show expired entries"
//! kpx search "find weak passwords in Banking group"
//! kpx search "entries with OTP created last month"
//! kpx search "tìm mật khẩu yếu"  (Vietnamese)

use crate::output::print_table_entries;
use colored::Colorize;

pub fn run(
    query: &str,
    vault_path: &str,
    password: &str,
    format: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    use keepassex_core::search::{build_search_filter, parse_nl_query};

    if !quiet {
        println!("{} {}", "🔍 Natural language search:".cyan(), query.bold());
    }

    // Parse the natural language query
    let nl_query = parse_nl_query(query);

    if !quiet {
        let intent_desc = describe_intent(&nl_query);
        println!("{} {}", "  Intent:".dimmed(), intent_desc.yellow());
        if let Some(group) = &nl_query.group {
            println!("{} {}", "  Group:".dimmed(), group.yellow());
        }
        if !nl_query.features.is_empty() {
            let features: Vec<String> = nl_query
                .features
                .iter()
                .map(|f| format!("{:?}", f))
                .collect();
            println!(
                "{} {}",
                "  Features:".dimmed(),
                features.join(", ").yellow()
            );
        }
        println!();
    }

    // Build executable filter
    let filter = build_search_filter(&nl_query);

    // TODO: Execute filter against vault
    // For now, show the parsed filter as JSON
    if format == "json" {
        println!(
            "{}",
            serde_json::to_string_pretty(&filter_to_json(&filter))?
        );
    } else {
        println!(
            "{}",
            "Search filter built successfully. Connect to vault to execute.".green()
        );
        println!("  Expired only: {}", filter.expired_only);
        println!("  Weak only: {}", filter.weak_only);
        println!("  Favorites: {}", filter.favorites_only);
        if let Some(text) = &filter.text {
            println!("  Text: {}", text);
        }
        if let Some(group) = &filter.group {
            println!("  Group: {}", group);
        }
    }

    Ok(())
}

fn describe_intent(query: &keepassex_core::search::NlQuery) -> String {
    use keepassex_core::search::NlIntent;
    match &query.intent {
        NlIntent::All => "Show all entries".to_string(),
        NlIntent::Expired => "Expired entries".to_string(),
        NlIntent::ExpiringSoon { days } => format!("Expiring within {} days", days),
        NlIntent::Weak => "Weak passwords".to_string(),
        NlIntent::Reused => "Reused passwords".to_string(),
        NlIntent::NoPassword => "Entries without password".to_string(),
        NlIntent::Breached => "Breached passwords".to_string(),
        NlIntent::Favorites => "Favorites".to_string(),
        NlIntent::Recent => "Recently used".to_string(),
        NlIntent::WithFeature(f) => format!("Entries with {:?}", f),
        NlIntent::CreatedIn(_) => "Created in time range".to_string(),
        NlIntent::ModifiedIn(_) => "Modified in time range".to_string(),
        NlIntent::NotUsedIn(_) => "Not used in time range".to_string(),
        NlIntent::Search(text) => format!("Search: {}", text),
    }
}

fn filter_to_json(filter: &keepassex_core::search::SearchFilter) -> serde_json::Value {
    serde_json::json!({
        "text": filter.text,
        "group": filter.group,
        "tags": filter.tags,
        "expired_only": filter.expired_only,
        "weak_only": filter.weak_only,
        "reused_only": filter.reused_only,
        "no_password_only": filter.no_password_only,
        "breached_only": filter.breached_only,
        "favorites_only": filter.favorites_only,
        "has_otp": filter.has_otp,
        "has_passkey": filter.has_passkey,
        "has_ssh_key": filter.has_ssh_key,
    })
}
