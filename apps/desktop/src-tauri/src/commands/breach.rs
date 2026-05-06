//! Breach check Tauri commands

use crate::state::AppState;
use keepassex_core::breach::{check_vault_passwords, check_password_offline, sha1_hex};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct BreachCheckResult {
    pub entry_uuid: String,
    pub entry_title: String,
    pub is_breached: bool,
    pub breach_count: u64,
    pub hash_prefix: String,
}

#[derive(Debug, Serialize)]
pub struct VaultBreachReport {
    pub total_checked: usize,
    pub breached_count: usize,
    pub results: Vec<BreachCheckResult>,
    pub used_online: bool,
}

/// Check all vault passwords against HIBP (online, k-anonymity)
#[tauri::command]
pub async fn check_vault_breaches(
    online: bool,
    state: State<'_, AppState>,
) -> Result<VaultBreachReport, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    // Collect passwords with their entry info
    let entries: Vec<(String, String, String)> = open_vault
        .vault
        .all_entries()
        .filter(|e| !e.password.get().is_empty())
        .map(|e| (
            e.uuid.to_string(),
            e.title.get().to_string(),
            e.password.get().to_string(),
        ))
        .collect();

    let total_checked = entries.len();
    let mut results = Vec::new();

    if online {
        // Online HIBP check (k-anonymity)
        let passwords: Vec<(String, String)> = entries
            .iter()
            .map(|(uuid, _, pw)| (uuid.clone(), pw.clone()))
            .collect();

        let breach_results = check_vault_passwords(&passwords).await;

        for (uuid, result) in breach_results {
            let entry_info = entries.iter().find(|(u, _, _)| u == &uuid);
            if let Some((_, title, _)) = entry_info {
                if result.is_breached {
                    results.push(BreachCheckResult {
                        entry_uuid: uuid,
                        entry_title: title.clone(),
                        is_breached: true,
                        breach_count: result.breach_count,
                        hash_prefix: result.password_hash_prefix,
                    });
                }
            }
        }
    } else {
        // Offline check against common passwords
        for (uuid, title, password) in &entries {
            let is_breached = check_password_offline(password);
            if is_breached {
                let hash = sha1_hex(password);
                results.push(BreachCheckResult {
                    entry_uuid: uuid.clone(),
                    entry_title: title.clone(),
                    is_breached: true,
                    breach_count: 0, // Unknown count in offline mode
                    hash_prefix: hash[..5].to_string(),
                });
            }
        }
    }

    let breached_count = results.len();

    Ok(VaultBreachReport {
        total_checked,
        breached_count,
        results,
        used_online: online,
    })
}

/// Check a single password against HIBP
#[tauri::command]
pub async fn check_password_breach(
    password: String,
    online: bool,
) -> Result<BreachCheckResult, String> {
    if online {
        let result = keepassex_core::breach::check_password_hibp(&password)
            .await
            .map_err(|e| e.to_string())?;

        Ok(BreachCheckResult {
            entry_uuid: String::new(),
            entry_title: String::new(),
            is_breached: result.is_breached,
            breach_count: result.breach_count,
            hash_prefix: result.password_hash_prefix,
        })
    } else {
        let is_breached = check_password_offline(&password);
        let hash = sha1_hex(&password);
        Ok(BreachCheckResult {
            entry_uuid: String::new(),
            entry_title: String::new(),
            is_breached,
            breach_count: 0,
            hash_prefix: hash[..5].to_string(),
        })
    }
}
