//! Tauri commands — Vault Analytics
use keepassex_core::analytics::{
    compute_analytics, EntryAnalyticsData, FeatureUsage, PasswordAgeStats,
    SecuritySummary, StrengthDistribution, TimelinePoint, VaultAnalytics,
};
use serde::Serialize;
use tauri::{command, State};

use crate::state::AppState;

/// Get full vault analytics report
#[command]
pub async fn get_vault_analytics(state: State<'_, AppState>) -> Result<VaultAnalytics, String> {
    let vault_guard = state.vault.lock().await;
    let vault = vault_guard.as_ref().ok_or("Vault not open")?;

    // Convert vault entries to analytics data
    let entries: Vec<EntryAnalyticsData> = vault
        .entries
        .values()
        .map(|e| {
            let password_score = if e.password.get().is_empty() {
                None
            } else {
                // Simple strength scoring (0-4)
                let pwd = e.password.get();
                let score = calculate_strength_score(pwd);
                Some(score)
            };

            EntryAnalyticsData {
                uuid: e.uuid.to_string(),
                title: e.title.get().to_string(),
                group_name: vault
                    .groups
                    .get(&e.group_uuid)
                    .map(|g| g.name.clone())
                    .unwrap_or_default(),
                tags: e.tags.clone(),
                password_score,
                created_at: Some(e.created_at),
                modified_at: Some(e.modified_at),
                accessed_at: Some(e.accessed_at),
                access_count: 0, // TODO: track access count
                expires_at: e.expiry,
                has_otp: e.otp.is_some(),
                has_passkey: !e.passkeys.is_empty(),
                has_ssh_key: e.ssh_key.is_some(),
                has_attachment: !e.attachments.is_empty(),
                has_custom_fields: !e.custom_fields.is_empty(),
                is_favorite: e.tags.contains(&"favorite".to_string()),
                is_breached: false, // TODO: check breach status
                is_reused: false,   // TODO: check reuse
            }
        })
        .collect();

    Ok(compute_analytics(&entries))
}

/// Export analytics report as PDF/HTML
#[command]
pub async fn export_analytics_report(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let analytics = get_vault_analytics(state).await?;

    // Generate HTML report
    let html = generate_html_report(&analytics);
    std::fs::write(&output_path, html).map_err(|e| e.to_string())?;

    Ok(output_path)
}

fn calculate_strength_score(password: &str) -> u8 {
    let len = password.len();
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety = [has_upper, has_lower, has_digit, has_symbol]
        .iter()
        .filter(|&&b| b)
        .count();

    match (len, variety) {
        (0..=5, _) => 0,
        (6..=8, v) if v <= 2 => 1,
        (6..=8, _) => 2,
        (9..=12, v) if v <= 2 => 2,
        (9..=12, _) => 3,
        (_, v) if v >= 3 => 4,
        _ => 3,
    }
}

fn generate_html_report(analytics: &VaultAnalytics) -> String {
    let ss = &analytics.security_summary;
    let sd = &analytics.strength_distribution;

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>KeePassEx Vault Analytics Report</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 40px; color: #1a1a1a; }}
  h1 {{ color: #2563eb; }} h2 {{ color: #374151; border-bottom: 1px solid #e5e7eb; padding-bottom: 8px; }}
  .score {{ font-size: 48px; font-weight: bold; color: {}; }}
  .grid {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin: 20px 0; }}
  .card {{ background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; padding: 16px; }}
  .card-value {{ font-size: 24px; font-weight: bold; }}
  .bar {{ background: #e5e7eb; border-radius: 4px; height: 20px; margin: 4px 0; }}
  .bar-fill {{ height: 100%; border-radius: 4px; }}
  footer {{ margin-top: 40px; color: #9ca3af; font-size: 12px; }}
</style>
</head>
<body>
<h1>🔐 KeePassEx Vault Analytics Report</h1>
<p>Generated: {} | Total entries: {}</p>

<h2>Security Health Score</h2>
<div class="score">{}/100</div>

<h2>Summary</h2>
<div class="grid">
  <div class="card"><div class="card-value">{}</div><div>Weak Passwords</div></div>
  <div class="card"><div class="card-value">{}</div><div>Reused Passwords</div></div>
  <div class="card"><div class="card-value">{}</div><div>Expired Entries</div></div>
  <div class="card"><div class="card-value">{}</div><div>Breached Passwords</div></div>
  <div class="card"><div class="card-value">{}</div><div>No Password</div></div>
  <div class="card"><div class="card-value">{}</div><div>Expiring Soon</div></div>
</div>

<h2>Password Strength Distribution</h2>
<p>Very Weak: {} | Weak: {} | Fair: {} | Strong: {} | Very Strong: {} | No Password: {}</p>

<h2>Feature Usage</h2>
<p>OTP: {} | Passkeys: {} | SSH Keys: {} | Attachments: {} | Favorites: {}</p>

<footer>Generated by KeePassEx — Your passwords, your control</footer>
</body>
</html>"#,
        if ss.health_score >= 80 { "#16a34a" } else if ss.health_score >= 60 { "#d97706" } else { "#dc2626" },
        analytics.generated_at.format("%Y-%m-%d %H:%M UTC"),
        analytics.total_entries,
        ss.health_score,
        ss.weak_count, ss.reused_count, ss.expired_count,
        ss.breached_count, ss.no_password_count, ss.expiring_soon_count,
        sd.very_weak, sd.weak, sd.fair, sd.strong, sd.very_strong, sd.no_password,
        analytics.feature_usage.with_otp,
        analytics.feature_usage.with_passkey,
        analytics.feature_usage.with_ssh_key,
        analytics.feature_usage.with_attachment,
        analytics.feature_usage.favorites,
    )
}
