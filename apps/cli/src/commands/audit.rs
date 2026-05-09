//! kpx audit — show vault audit log
//!
//! Reads the audit log stored inside the KDBX vault and displays it
//! in table, JSON, or CSV format.

use colored::Colorize;
use keepassex_core::audit_log::AuditEventType;
use keepassex_core::Vault;

pub fn run(vault: &Vault, limit: usize, format: &str) -> anyhow::Result<()> {
    let log = &vault.audit_log;
    let events = log.recent(limit);

    if events.is_empty() {
        println!("{}", "No audit events recorded.".dimmed());
        println!(
            "{}",
            "Audit events are recorded automatically as you use the vault.".dimmed()
        );
        return Ok(());
    }

    match format {
        "json" => print_json(events),
        "csv" => print_csv(events),
        _ => print_table(events),
    }

    Ok(())
}

fn print_table(events: &[keepassex_core::audit_log::AuditEvent]) {
    println!("{}", "Audit Log".bold());
    println!("{}", "─".repeat(90).dimmed());
    println!(
        "{:<22} {:<28} {:<28} {}",
        "Timestamp".bold(),
        "Event".bold(),
        "Entry".bold(),
        "Details".bold()
    );
    println!("{}", "─".repeat(90).dimmed());

    // Show newest first
    for event in events.iter().rev() {
        // Truncate timestamp to readable format
        let ts = if event.timestamp.len() >= 19 {
            &event.timestamp[..19]
        } else {
            &event.timestamp
        };

        let event_label = format_event_type(&event.event_type);
        let entry = event
            .entry_title
            .as_deref()
            .unwrap_or("—")
            .chars()
            .take(26)
            .collect::<String>();
        let details = event
            .details
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(28)
            .collect::<String>();

        let colored_event = match &event.event_type {
            AuditEventType::VaultOpened
            | AuditEventType::EntryViewed
            | AuditEventType::BiometricUnlock => event_label.green().to_string(),
            AuditEventType::FailedUnlockAttempt => event_label.red().bold().to_string(),
            AuditEventType::EntryDeleted
            | AuditEventType::VaultLocked
            | AuditEventType::ExportCompleted => event_label.yellow().to_string(),
            AuditEventType::PasswordCopied | AuditEventType::OtpGenerated => {
                event_label.blue().to_string()
            }
            _ => event_label.cyan().to_string(),
        };

        println!(
            "{:<22} {:<38} {:<28} {}",
            ts.dimmed(),
            colored_event,
            entry,
            details.dimmed()
        );
    }

    println!("{}", "─".repeat(90).dimmed());
    println!("  {} events shown", events.len().to_string().bold());
}

fn print_json(events: &[keepassex_core::audit_log::AuditEvent]) {
    let json_events: Vec<serde_json::Value> = events
        .iter()
        .map(|e| {
            serde_json::json!({
                "id": e.id,
                "timestamp": e.timestamp,
                "event_type": e.event_type.to_string(),
                "platform": e.platform,
                "entry_uuid": e.entry_uuid,
                "entry_title": e.entry_title,
                "details": e.details,
                "ip_address": e.ip_address,
            })
        })
        .collect();

    println!(
        "{}",
        serde_json::to_string_pretty(&json_events).unwrap_or_default()
    );
}

fn print_csv(events: &[keepassex_core::audit_log::AuditEvent]) {
    println!("id,timestamp,event_type,platform,entry_uuid,entry_title,details");
    for e in events {
        println!(
            "{},{},{},{},{},{},{}",
            e.id,
            e.timestamp,
            e.event_type,
            e.platform,
            e.entry_uuid.as_deref().unwrap_or(""),
            e.entry_title.as_deref().unwrap_or("").replace(',', ";"),
            e.details.as_deref().unwrap_or("").replace(',', ";"),
        );
    }
}

fn format_event_type(t: &AuditEventType) -> String {
    t.to_string()
}
