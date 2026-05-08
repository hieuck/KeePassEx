//! Output formatting utilities for the KeePassEx CLI

use colored::Colorize;

pub fn success(msg: &str) {
    eprintln!("{} {}", "✓".green(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red(), msg);
}

pub fn warn(msg: &str) {
    eprintln!("{} {}", "⚠".yellow(), msg);
}

pub fn info(msg: &str) {
    eprintln!("{} {}", "ℹ".blue(), msg);
}

/// Print a success message (alias for success())
pub fn print_success(msg: &str) {
    success(msg);
}

/// Print a table of entries
pub fn print_table_entries(entries: &[&keepassex_core::types::Entry], show_passwords: bool) {
    if entries.is_empty() {
        println!("{}", "No entries found.".dimmed());
        return;
    }

    println!(
        "{:<38} {:<20} {:<20} {}",
        "UUID".bold(),
        "Title".bold(),
        "Username".bold(),
        "URL".bold()
    );
    println!("{}", "─".repeat(100).dimmed());

    for entry in entries {
        let uuid_short = &entry.uuid.to_string()[..8];
        let title = truncate(entry.title.get(), 18);
        let username = truncate(entry.username.get(), 18);
        let url = truncate(&entry.url, 30);

        let expired_marker = if entry.check_expired() {
            " ⏰".red().to_string()
        } else {
            String::new()
        };

        println!(
            "{:<38} {:<20} {:<20} {}{}",
            uuid_short.dimmed(),
            title.bold(),
            username.cyan(),
            url.dimmed(),
            expired_marker,
        );

        if show_passwords && !entry.password.get().is_empty() {
            println!(
                "  {}: {}",
                "Password".dimmed(),
                entry.password.get().yellow()
            );
        }
    }

    println!();
    println!("{} entries", entries.len().to_string().bold());
}

/// Print a generic table (key-value rows)
pub fn print_table(rows: &[(&str, &str)]) {
    for (key, value) in rows {
        println!("{:<25} {}", format!("{}:", key).bold(), value);
    }
}

/// Print data as JSON
pub fn print_json(value: &serde_json::Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{} Failed to serialize JSON: {}", "✗".red(), e),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
