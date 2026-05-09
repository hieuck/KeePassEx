//! Health audit Tauri commands

use crate::state::AppState;
use keepassex_core::expiry_engine::{analyze_vault_rotations, ExpiryInput};
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

#[derive(Debug, Serialize, Clone)]
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

/// Password rotation recommendation DTO
#[derive(Debug, Serialize, Clone)]
pub struct RotationRecommendationDto {
    pub entry_uuid: String,
    pub entry_title: String,
    pub urgency: String, // "fresh" | "aging" | "soon" | "overdue" | "expired"
    pub urgency_color: String, // hex color
    pub age_days: i64,
    pub recommended_max_days: i64,
    pub days_until_overdue: i64,
    pub message_en: String,
    pub message_vi: String,
}

/// Duplicate entry DTO
#[derive(Debug, Serialize, Clone)]
pub struct DuplicateGroupDto {
    pub reason: String, // "same_password" | "same_url_username" | "same_title"
    pub entries: Vec<EntryRefDto>,
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

/// Get password rotation recommendations for all vault entries
/// KeePassEx exclusive: no competitor has proactive rotation engine
#[tauri::command]
pub fn get_rotation_recommendations(
    state: State<'_, AppState>,
) -> Result<Vec<RotationRecommendationDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let inputs: Vec<ExpiryInput> = open_vault
        .vault
        .all_entries()
        .map(|entry| {
            // Use categorizer to get category hint
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

    let recommendations = analyze_vault_rotations(&inputs);

    Ok(recommendations
        .into_iter()
        .map(|r| RotationRecommendationDto {
            entry_uuid: r.entry_uuid,
            entry_title: r.entry_title,
            urgency: format!("{:?}", r.urgency).to_lowercase(),
            urgency_color: r.urgency.color_hex().to_string(),
            age_days: r.age_days,
            recommended_max_days: r.recommended_max_days,
            days_until_overdue: r.days_until_overdue,
            message_en: r.message_en,
            message_vi: r.message_vi,
        })
        .collect())
}

/// Find duplicate entries (same password, same URL+username, or same title)
/// KeePassEx exclusive: no competitor has built-in duplicate detection
#[tauri::command]
pub fn find_duplicate_entries(
    state: State<'_, AppState>,
) -> Result<Vec<DuplicateGroupDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let entries: Vec<_> = open_vault.vault.all_entries().collect();
    let mut duplicates: Vec<DuplicateGroupDto> = Vec::new();

    // 1. Same password (non-empty)
    let mut password_groups: std::collections::HashMap<String, Vec<EntryRefDto>> =
        std::collections::HashMap::new();
    for entry in &entries {
        let pw = entry.password.get().to_string();
        if !pw.is_empty() {
            password_groups.entry(pw).or_default().push(EntryRefDto {
                uuid: entry.uuid.to_string(),
                title: entry.title.get().to_string(),
            });
        }
    }
    for (_, group) in password_groups {
        if group.len() > 1 {
            duplicates.push(DuplicateGroupDto {
                reason: "same_password".to_string(),
                entries: group,
            });
        }
    }

    // 2. Same URL + username (non-empty)
    let mut url_user_groups: std::collections::HashMap<String, Vec<EntryRefDto>> =
        std::collections::HashMap::new();
    for entry in &entries {
        let url = entry.url.trim().to_lowercase();
        let user = entry.username.get().trim().to_lowercase();
        if !url.is_empty() && !user.is_empty() {
            let key = format!("{}|{}", url, user);
            url_user_groups.entry(key).or_default().push(EntryRefDto {
                uuid: entry.uuid.to_string(),
                title: entry.title.get().to_string(),
            });
        }
    }
    for (_, group) in url_user_groups {
        if group.len() > 1 {
            duplicates.push(DuplicateGroupDto {
                reason: "same_url_username".to_string(),
                entries: group,
            });
        }
    }

    // 3. Same title (case-insensitive, non-empty)
    let mut title_groups: std::collections::HashMap<String, Vec<EntryRefDto>> =
        std::collections::HashMap::new();
    for entry in &entries {
        let title = entry.title.get().trim().to_lowercase();
        if !title.is_empty() {
            title_groups.entry(title).or_default().push(EntryRefDto {
                uuid: entry.uuid.to_string(),
                title: entry.title.get().to_string(),
            });
        }
    }
    for (_, group) in title_groups {
        if group.len() > 1 {
            duplicates.push(DuplicateGroupDto {
                reason: "same_title".to_string(),
                entries: group,
            });
        }
    }

    // Sort: same_password first (most critical), then url+user, then title
    duplicates.sort_by_key(|d| match d.reason.as_str() {
        "same_password" => 0,
        "same_url_username" => 1,
        _ => 2,
    });

    Ok(duplicates)
}
