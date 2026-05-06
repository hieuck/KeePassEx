//! Tauri commands — Context-Aware Password Strength Advisor
use keepassex_core::password_advisor::{advise_password, PasswordAdvice};
use tauri::command;

/// Analyze a password in context and return actionable advice (EN + VI).
///
/// Unlike a generic strength meter, this considers the entry's URL/title/category
/// to give specific recommendations — e.g., banking needs 16+ chars with symbols.
#[command]
pub fn advise_password_strength(
    password: String,
    entry_title: String,
    entry_url: String,
    category: String,
) -> PasswordAdvice {
    advise_password(&password, &entry_title, &entry_url, &category)
}
