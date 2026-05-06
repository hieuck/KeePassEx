//! Vault health audit — password quality, duplicates, expiry, breach check

use crate::generator::PasswordGenerator;
use crate::types::Entry;
use crate::vault::Vault;
use std::collections::HashMap;

/// Full vault health report
#[derive(Debug, Clone)]
pub struct VaultHealthReport {
    pub total_entries: usize,
    pub weak_passwords: Vec<WeakPasswordIssue>,
    pub reused_passwords: Vec<ReusedPasswordGroup>,
    pub expired_entries: Vec<ExpiredEntry>,
    pub expiring_soon: Vec<ExpiringEntry>,
    pub entries_without_password: Vec<EntryRef>,
    pub entries_without_url: Vec<EntryRef>,
    pub old_passwords: Vec<OldPasswordEntry>,
    pub score: u8, // 0-100
}

#[derive(Debug, Clone)]
pub struct WeakPasswordIssue {
    pub entry_uuid: String,
    pub entry_title: String,
    pub strength_score: u8,
    pub strength_label: String,
}

#[derive(Debug, Clone)]
pub struct ReusedPasswordGroup {
    pub password_hash: String, // SHA256 prefix for display
    pub entries: Vec<EntryRef>,
}

#[derive(Debug, Clone)]
pub struct ExpiredEntry {
    pub entry_uuid: String,
    pub entry_title: String,
    pub expired_at: String,
}

#[derive(Debug, Clone)]
pub struct ExpiringEntry {
    pub entry_uuid: String,
    pub entry_title: String,
    pub expires_at: String,
    pub days_remaining: i64,
}

#[derive(Debug, Clone)]
pub struct OldPasswordEntry {
    pub entry_uuid: String,
    pub entry_title: String,
    pub last_modified: String,
    pub days_old: i64,
}

#[derive(Debug, Clone)]
pub struct EntryRef {
    pub uuid: String,
    pub title: String,
}

/// Run full health audit on vault
pub fn audit_vault(vault: &Vault) -> VaultHealthReport {
    let entries: Vec<&Entry> = vault.all_entries().collect();
    let total = entries.len();

    let weak = find_weak_passwords(&entries);
    let reused = find_reused_passwords(&entries);
    let expired = find_expired(&entries);
    let expiring = find_expiring_soon(&entries, 30);
    let no_password = find_entries_without_password(&entries);
    let no_url = find_entries_without_url(&entries);
    let old = find_old_passwords(&entries, 365);

    let score = calculate_score(
        total,
        weak.len(),
        reused.len(),
        expired.len(),
        no_password.len(),
    );

    VaultHealthReport {
        total_entries: total,
        weak_passwords: weak,
        reused_passwords: reused,
        expired_entries: expired,
        expiring_soon: expiring,
        entries_without_password: no_password,
        entries_without_url: no_url,
        old_passwords: old,
        score,
    }
}

fn find_weak_passwords(entries: &[&Entry]) -> Vec<WeakPasswordIssue> {
    entries
        .iter()
        .filter(|e| !e.password.get().is_empty())
        .filter_map(|e| {
            let strength = PasswordGenerator::score_strength(e.password.get());
            if strength.score() < 3 {
                Some(WeakPasswordIssue {
                    entry_uuid: e.uuid.to_string(),
                    entry_title: e.title.get().to_string(),
                    strength_score: strength.score(),
                    strength_label: strength.label_en().to_string(),
                })
            } else {
                None
            }
        })
        .collect()
}

fn find_reused_passwords(entries: &[&Entry]) -> Vec<ReusedPasswordGroup> {
    use sha2::{Digest, Sha256};

    let mut password_map: HashMap<String, Vec<EntryRef>> = HashMap::new();

    for entry in entries {
        let pw = entry.password.get();
        if pw.is_empty() {
            continue;
        }

        let mut hasher = Sha256::new();
        hasher.update(pw.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        password_map.entry(hash).or_default().push(EntryRef {
            uuid: entry.uuid.to_string(),
            title: entry.title.get().to_string(),
        });
    }

    password_map
        .into_iter()
        .filter(|(_, entries)| entries.len() > 1)
        .map(|(hash, entries)| ReusedPasswordGroup {
            password_hash: hash[..8].to_string(),
            entries,
        })
        .collect()
}

fn find_expired(entries: &[&Entry]) -> Vec<ExpiredEntry> {
    entries
        .iter()
        .filter(|e| e.is_expired || e.check_expired())
        .map(|e| ExpiredEntry {
            entry_uuid: e.uuid.to_string(),
            entry_title: e.title.get().to_string(),
            expired_at: e
                .expiry
                .map(|t| t.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        })
        .collect()
}

fn find_expiring_soon(entries: &[&Entry], days: i64) -> Vec<ExpiringEntry> {
    entries
        .iter()
        .filter(|e| e.expires_within_days(days))
        .map(|e| {
            let expiry = e.expiry.unwrap();
            let days_remaining = (expiry - chrono::Utc::now()).num_days();
            ExpiringEntry {
                entry_uuid: e.uuid.to_string(),
                entry_title: e.title.get().to_string(),
                expires_at: expiry.format("%Y-%m-%d").to_string(),
                days_remaining,
            }
        })
        .collect()
}

fn find_entries_without_password(entries: &[&Entry]) -> Vec<EntryRef> {
    entries
        .iter()
        .filter(|e| e.password.get().is_empty())
        .map(|e| EntryRef {
            uuid: e.uuid.to_string(),
            title: e.title.get().to_string(),
        })
        .collect()
}

fn find_entries_without_url(entries: &[&Entry]) -> Vec<EntryRef> {
    entries
        .iter()
        .filter(|e| e.url.is_empty())
        .map(|e| EntryRef {
            uuid: e.uuid.to_string(),
            title: e.title.get().to_string(),
        })
        .collect()
}

fn find_old_passwords(entries: &[&Entry], max_age_days: i64) -> Vec<OldPasswordEntry> {
    let now = chrono::Utc::now();
    entries
        .iter()
        .filter(|e| {
            let age = (now - e.modified_at).num_days();
            age > max_age_days && !e.password.get().is_empty()
        })
        .map(|e| {
            let age = (now - e.modified_at).num_days();
            OldPasswordEntry {
                entry_uuid: e.uuid.to_string(),
                entry_title: e.title.get().to_string(),
                last_modified: e.modified_at.format("%Y-%m-%d").to_string(),
                days_old: age,
            }
        })
        .collect()
}

fn calculate_score(
    total: usize,
    weak: usize,
    reused: usize,
    expired: usize,
    no_password: usize,
) -> u8 {
    if total == 0 {
        return 100;
    }

    let issues = weak + reused * 2 + expired + no_password;
    let ratio = issues as f64 / total as f64;
    let score = ((1.0 - ratio.min(1.0)) * 100.0) as u8;
    score
}
