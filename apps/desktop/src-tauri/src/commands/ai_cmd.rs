//! AI password suggestion Tauri commands

use crate::state::AppState;
use keepassex_core::ai::{suggest_passwords, SuggestionContext};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Deserialize)]
pub struct SuggestPasswordsArgs {
    pub url: String,
    pub title: String,
    pub category: String,
    /// Number of suggestions to return (default: 5)
    pub count: Option<usize>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PasswordSuggestionDto {
    pub password: String,
    pub entropy: f64,
    pub strength_score: u8,
    pub rationale_en: String,
    pub rationale_vi: String,
    pub strategy: String,
}

/// Generate AI-powered password suggestions for an entry context.
/// Uses existing vault passwords to learn style preferences.
#[tauri::command]
pub fn suggest_passwords_cmd(
    args: SuggestPasswordsArgs,
    state: State<'_, AppState>,
) -> Vec<PasswordSuggestionDto> {
    let count = args.count.unwrap_or(5).min(10);

    // Collect existing strong passwords from vault for style learning
    let existing_passwords: Vec<String> = {
        let vault_lock = state.vault.read().unwrap();
        if let Some(ref ov) = *vault_lock {
            if !ov.locked {
                ov.vault
                    .all_entries()
                    .filter(|e| !e.password.get().is_empty())
                    .map(|e| e.password.get().to_string())
                    .take(50) // Limit to 50 for performance
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    let ctx = SuggestionContext {
        url: &args.url,
        title: &args.title,
        category: &args.category,
        existing_passwords: &existing_passwords,
    };

    suggest_passwords(&ctx, count)
        .into_iter()
        .map(|s| PasswordSuggestionDto {
            password: s.password,
            entropy: s.entropy,
            strength_score: s.strength_score,
            rationale_en: s.rationale_en,
            rationale_vi: s.rationale_vi,
            strategy: format!("{:?}", s.strategy),
        })
        .collect()
}
