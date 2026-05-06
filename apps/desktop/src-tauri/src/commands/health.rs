//! Health audit Tauri commands

use crate::state::AppState;
use keepassex_core::health;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct HealthReportDto {
    pub total_entries: usize,
    pub score: u8,
    pub weak_count: usize,
    pub reused_count: usize,
    pub expired_count: usize,
    pub expiring_soon_count: usize,
    pub no_password_count: usize,
    pub old_password_count: usize,
    pub weak_passwords: Vec<WeakPasswordDto>,
    pub reused_passwords: Vec<ReusedGroupDto>,
    pub expired_entries: Vec<ExpiredDto>,
    pub expiring_soon: Vec<ExpiringDto>,
}

#[derive(Debug, Serialize)]
pub struct WeakPasswordDto {
    pub entry_uuid: String,
    pub entry_title: String,
    pub strength_score: u8,
    pub strength_label: String,
}

#[derive(Debug, Serialize)]
pub struct ReusedGroupDto {
    pub entries: Vec<EntryRefDto>,
}

#[derive(Debug, Serialize)]
pub struct EntryRefDto {
    pub uuid: String,
    pub title: String,
}

#[derive(Debug, Serialize)]
pub struct ExpiredDto {
    pub entry_uuid: String,
    pub entry_title: String,
    pub expired_at: String,
}

#[derive(Debug, Serialize)]
pub struct ExpiringDto {
    pub entry_uuid: String,
    pub entry_title: String,
    pub expires_at: String,
    pub days_remaining: i64,
}

#[tauri::command]
pub fn audit_vault(state: State<'_, AppState>) -> Result<HealthReportDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let report = health::audit_vault(&open_vault.vault);

    Ok(HealthReportDto {
        total_entries: report.total_entries,
        score: report.score,
        weak_count: report.weak_passwords.len(),
        reused_count: report.reused_passwords.len(),
        expired_count: report.expired_entries.len(),
        expiring_soon_count: report.expiring_soon.len(),
        no_password_count: report.entries_without_password.len(),
        old_password_count: report.old_passwords.len(),
        weak_passwords: report
            .weak_passwords
            .into_iter()
            .map(|w| WeakPasswordDto {
                entry_uuid: w.entry_uuid,
                entry_title: w.entry_title,
                strength_score: w.strength_score,
                strength_label: w.strength_label,
            })
            .collect(),
        reused_passwords: report
            .reused_passwords
            .into_iter()
            .map(|r| ReusedGroupDto {
                entries: r
                    .entries
                    .into_iter()
                    .map(|e| EntryRefDto {
                        uuid: e.uuid,
                        title: e.title,
                    })
                    .collect(),
            })
            .collect(),
        expired_entries: report
            .expired_entries
            .into_iter()
            .map(|e| ExpiredDto {
                entry_uuid: e.entry_uuid,
                entry_title: e.entry_title,
                expired_at: e.expired_at,
            })
            .collect(),
        expiring_soon: report
            .expiring_soon
            .into_iter()
            .map(|e| ExpiringDto {
                entry_uuid: e.entry_uuid,
                entry_title: e.entry_title,
                expires_at: e.expires_at,
                days_remaining: e.days_remaining,
            })
            .collect(),
    })
}
