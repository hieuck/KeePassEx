//! Tauri commands — Natural Language Search
use keepassex_core::search::{build_search_filter, parse_nl_query, SearchFilter};
use serde::Serialize;
use tauri::{command, State};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub uuid: String,
    pub title: String,
    pub username: String,
    pub url: String,
    pub group_name: String,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub is_expired: bool,
    pub is_favorite: bool,
    pub modified_at: String,
    pub relevance_score: f32,
}

/// Execute a natural language search query against the vault
#[command]
pub fn nl_search(state: State<'_, AppState>, query: String) -> Result<Vec<SearchResult>, String> {
    let vault_guard = state.vault.read().map_err(|e| e.to_string())?;
    let open_vault = vault_guard.as_ref().ok_or("Vault not open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let vault = &open_vault.vault;

    // Parse natural language query
    let nl_query = parse_nl_query(&query);
    let filter = build_search_filter(&nl_query);

    // Apply filter using the public Vault API
    let mut results: Vec<SearchResult> = vault
        .all_entries()
        .filter(|entry| apply_filter(entry, &filter, vault))
        .map(|entry| {
            let group_name = vault
                .get_group(&entry.group_uuid)
                .map(|g| g.name.clone())
                .unwrap_or_default();

            SearchResult {
                uuid: entry.uuid.to_string(),
                title: entry.title.get().to_string(),
                username: entry.username.get().to_string(),
                url: entry.url.clone(),
                group_name,
                has_otp: entry.otp.is_some(),
                has_passkey: !entry.passkeys.is_empty(),
                is_expired: entry.check_expired(),
                is_favorite: entry.tags.contains(&"favorite".to_string()),
                modified_at: entry.modified_at.to_rfc3339(),
                relevance_score: calculate_relevance(entry, &query),
            }
        })
        .collect();

    // Sort by relevance
    results.sort_by(|a, b| {
        b.relevance_score
            .partial_cmp(&a.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
}

/// Parse a natural language query and return the interpreted filter (for UI display)
#[command]
pub fn parse_search_query(query: String) -> Result<serde_json::Value, String> {
    let nl_query = parse_nl_query(&query);
    let filter = build_search_filter(&nl_query);

    Ok(serde_json::json!({
        "intent": format!("{:?}", nl_query.intent),
        "group": filter.group,
        "tags": filter.tags,
        "expired_only": filter.expired_only,
        "weak_only": filter.weak_only,
        "reused_only": filter.reused_only,
        "favorites_only": filter.favorites_only,
        "has_otp": filter.has_otp,
        "has_passkey": filter.has_passkey,
        "text": filter.text,
        "lang": format!("{:?}", nl_query.lang),
    }))
}

fn apply_filter(
    entry: &keepassex_core::types::Entry,
    filter: &SearchFilter,
    vault: &keepassex_core::Vault,
) -> bool {
    // Text search
    if let Some(text) = &filter.text {
        let q = text.to_lowercase();
        let matches = entry.title.get().to_lowercase().contains(&q)
            || entry.username.get().to_lowercase().contains(&q)
            || entry.url.to_lowercase().contains(&q)
            || entry.notes.get().to_lowercase().contains(&q)
            || entry.tags.iter().any(|t| t.to_lowercase().contains(&q));
        if !matches {
            return false;
        }
    }

    // Group filter
    if let Some(group_name) = &filter.group {
        let matches = vault
            .get_group(&entry.group_uuid)
            .map(|g| g.name.to_lowercase().contains(&group_name.to_lowercase()))
            .unwrap_or(false);
        if !matches {
            return false;
        }
    }

    // Expired filter
    if filter.expired_only && !entry.check_expired() {
        return false;
    }

    // Expiring soon
    if let Some(days) = filter.expiring_within_days {
        if !entry.expires_within_days(days as i64) {
            return false;
        }
    }

    // Favorites
    if filter.favorites_only && !entry.tags.contains(&"favorite".to_string()) {
        return false;
    }

    // OTP filter
    if filter.has_otp == Some(true) && entry.otp.is_none() {
        return false;
    }
    if filter.has_otp == Some(false) && entry.otp.is_some() {
        return false;
    }

    // Passkey filter
    if filter.has_passkey == Some(true) && entry.passkeys.is_empty() {
        return false;
    }

    // SSH key filter
    if filter.has_ssh_key == Some(true) && entry.ssh_key.is_none() {
        return false;
    }

    // No password filter
    if filter.no_password_only && !entry.password.get().is_empty() {
        return false;
    }

    // Created after/before
    if let Some(after) = filter.created_after {
        if entry.created_at < after {
            return false;
        }
    }
    if let Some(before) = filter.created_before {
        if entry.created_at > before {
            return false;
        }
    }

    // Modified after/before
    if let Some(after) = filter.modified_after {
        if entry.modified_at < after {
            return false;
        }
    }

    true
}

fn calculate_relevance(entry: &keepassex_core::types::Entry, query: &str) -> f32 {
    let q = query.to_lowercase();
    let title = entry.title.get().to_lowercase();

    if title == q {
        return 1.0;
    }
    if title.starts_with(&q) {
        return 0.9;
    }
    if title.contains(&q) {
        return 0.7;
    }
    if entry.username.get().to_lowercase().contains(&q) {
        return 0.5;
    }
    if entry.url.to_lowercase().contains(&q) {
        return 0.4;
    }
    0.1
}
