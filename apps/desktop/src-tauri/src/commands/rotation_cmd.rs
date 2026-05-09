//! Password rotation Tauri commands
//!
//! KeePassEx exclusive: bulk password rotation with AI-generated replacements.
//! No competitor has this feature.

use crate::state::AppState;
use keepassex_core::expiry_engine::{analyze_vault_rotations, ExpiryInput};
use keepassex_core::generator::PasswordGenerator;
use keepassex_core::types::PasswordGeneratorConfig;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Clone)]
pub struct RotationSummaryDto {
    pub total_checked: usize,
    pub overdue: usize,
    pub soon: usize,
    pub aging: usize,
    pub fresh: usize,
}

#[derive(Debug, Deserialize)]
pub struct BulkRotateArgs {
    /// UUIDs of entries to rotate
    pub entry_uuids: Vec<String>,
    /// Password generation config (optional — uses defaults if None)
    pub length: Option<usize>,
    pub use_symbols: Option<bool>,
}

/// Get rotation summary for the vault
#[tauri::command]
pub fn get_rotation_summary(state: State<'_, AppState>) -> Result<RotationSummaryDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let inputs: Vec<ExpiryInput> = open_vault
        .vault
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

    let recs = analyze_vault_rotations(&inputs);

    let overdue = recs
        .iter()
        .filter(|r| {
            matches!(
                r.urgency,
                keepassex_core::expiry_engine::RotationUrgency::Overdue
                    | keepassex_core::expiry_engine::RotationUrgency::Expired
            )
        })
        .count();
    let soon = recs
        .iter()
        .filter(|r| {
            matches!(
                r.urgency,
                keepassex_core::expiry_engine::RotationUrgency::Soon
            )
        })
        .count();
    let aging = recs
        .iter()
        .filter(|r| {
            matches!(
                r.urgency,
                keepassex_core::expiry_engine::RotationUrgency::Aging
            )
        })
        .count();

    Ok(RotationSummaryDto {
        total_checked: inputs.len(),
        overdue,
        soon,
        aging,
        fresh: inputs.len().saturating_sub(overdue + soon + aging),
    })
}

/// Bulk rotate passwords for multiple entries
/// Generates a new strong password for each entry and saves
#[tauri::command]
pub fn bulk_rotate_passwords(
    args: BulkRotateArgs,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let config = PasswordGeneratorConfig {
        length: args.length.unwrap_or(20),
        use_symbols: args.use_symbols.unwrap_or(true),
        ..Default::default()
    };

    let mut rotated = Vec::new();

    for uuid_str in &args.entry_uuids {
        let uuid = Uuid::parse_str(uuid_str).map_err(|e| e.to_string())?;

        // Generate new password
        let new_password = PasswordGenerator::generate(&config).map_err(|e| e.to_string())?;

        // Update entry
        let entry = open_vault
            .vault
            .get_entry_mut(&uuid)
            .ok_or_else(|| format!("Entry not found: {}", uuid_str))?;

        entry.password.set(new_password);
        entry.modified_at = chrono::Utc::now();
        rotated.push(uuid_str.clone());
    }

    if !rotated.is_empty() {
        open_vault.vault.dirty = true;
    }

    tracing::info!("Bulk rotated {} passwords", rotated.len());
    Ok(rotated)
}
