//! List entries command

use keepassex_core::{Vault, types::SearchQuery};
use colored::Colorize;
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct EntryRow {
    #[tabled(rename = "UUID")]
    uuid: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Username")]
    username: String,
    #[tabled(rename = "URL")]
    url: String,
    #[tabled(rename = "Modified")]
    modified: String,
}

pub fn run(
    vault: &Vault,
    group: Option<String>,
    search: Option<String>,
    show_passwords: bool,
    format: &str,
) -> anyhow::Result<()> {
    let entries: Vec<_> = if let Some(query) = search {
        let q = SearchQuery::new(query);
        vault.search(&q)
    } else {
        vault.all_entries().collect()
    };

    if entries.is_empty() {
        println!("{}", "No entries found.".dimmed());
        return Ok(());
    }

    match format {
        "json" => {
            let json: Vec<serde_json::Value> = entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "uuid": e.uuid.to_string(),
                        "title": e.title.get(),
                        "username": e.username.get(),
                        "url": e.url,
                        "modified": e.modified_at.to_rfc3339(),
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            let rows: Vec<EntryRow> = entries
                .iter()
                .map(|e| EntryRow {
                    uuid: e.uuid.to_string()[..8].to_string() + "...",
                    title: e.title.get().to_string(),
                    username: e.username.get().to_string(),
                    url: truncate(&e.url, 40),
                    modified: e.modified_at.format("%Y-%m-%d").to_string(),
                })
                .collect();

            println!("{}", Table::new(rows));
            println!("\n{} entries", entries.len().to_string().bold());
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}
