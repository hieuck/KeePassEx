//! Tauri commands — Vault Analytics
use keepassex_core::analytics::{compute_analytics, EntryAnalyticsData, VaultAnalytics};
use tauri::{command, State};

use crate::state::AppState;

/// Get full vault analytics report
#[command]
pub fn get_vault_analytics(state: State<'_, AppState>) -> Result<VaultAnalytics, String> {
    let vault_guard = state.vault.read().map_err(|e| e.to_string())?;
    let open_vault = vault_guard.as_ref().ok_or("Vault not open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let vault = &open_vault.vault;

    // Convert vault entries to analytics data using the public Vault API
    let entries: Vec<EntryAnalyticsData> = vault
        .all_entries()
        .map(|e| {
            let password_score = if e.password.get().is_empty() {
                None
            } else {
                Some(calculate_strength_score(e.password.get()))
            };

            let group_name = vault
                .get_group(&e.group_uuid)
                .map(|g| g.name.clone())
                .unwrap_or_default();

            EntryAnalyticsData {
                uuid: e.uuid.to_string(),
                title: e.title.get().to_string(),
                group_name,
                tags: e.tags.clone(),
                password_score,
                created_at: Some(e.created_at),
                modified_at: Some(e.modified_at),
                accessed_at: Some(e.accessed_at),
                access_count: e.usage_count as usize,
                expires_at: e.expiry,
                has_otp: e.otp.is_some(),
                has_passkey: !e.passkeys.is_empty(),
                has_ssh_key: e.ssh_key.is_some(),
                has_attachment: !e.attachments.is_empty(),
                has_custom_fields: !e.custom_fields.is_empty(),
                is_favorite: e.tags.contains(&"favorite".to_string()),
                is_breached: false,
                is_reused: false,
            }
        })
        .collect();

    Ok(compute_analytics(&entries))
}

/// Export analytics report as HTML
#[command]
pub fn export_analytics_report(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let analytics = get_vault_analytics(state)?;
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
    let score_color = if ss.health_score >= 80 {
        "#16a34a"
    } else if ss.health_score >= 60 {
        "#d97706"
    } else {
        "#dc2626"
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>KeePassEx Vault Analytics Report</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 40px; color: #1a1a1a; }}
  h1 {{ color: #2563eb; }}
  h2 {{ color: #374151; border-bottom: 1px solid #e5e7eb; padding-bottom: 8px; }}
  .score {{ font-size: 48px; font-weight: bold; color: {score_color}; }}
  .grid {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; margin: 20px 0; }}
  .card {{ background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; padding: 16px; }}
  .card-value {{ font-size: 24px; font-weight: bold; }}
  footer {{ margin-top: 40px; color: #9ca3af; font-size: 12px; }}
</style>
</head>
<body>
<h1>🔐 KeePassEx Vault Analytics Report</h1>
<p>Generated: {generated} | Total entries: {total}</p>

<h2>Security Health Score</h2>
<div class="score">{score}/100</div>

<h2>Summary</h2>
<div class="grid">
  <div class="card"><div class="card-value">{weak}</div><div>Weak Passwords</div></div>
  <div class="card"><div class="card-value">{reused}</div><div>Reused Passwords</div></div>
  <div class="card"><div class="card-value">{expired}</div><div>Expired Entries</div></div>
  <div class="card"><div class="card-value">{breached}</div><div>Breached Passwords</div></div>
  <div class="card"><div class="card-value">{no_pwd}</div><div>No Password</div></div>
  <div class="card"><div class="card-value">{expiring}</div><div>Expiring Soon</div></div>
</div>

<h2>Password Strength Distribution</h2>
<p>Very Weak: {vw} | Weak: {w} | Fair: {f} | Strong: {s} | Very Strong: {vs} | No Password: {np}</p>

<h2>Feature Usage</h2>
<p>OTP: {otp} | Passkeys: {pk} | SSH Keys: {ssh} | Attachments: {att} | Favorites: {fav}</p>

<footer>Generated by KeePassEx — Your passwords, your control</footer>
</body>
</html>"#,
        score_color = score_color,
        generated = analytics.generated_at.format("%Y-%m-%d %H:%M UTC"),
        total = analytics.total_entries,
        score = ss.health_score,
        weak = ss.weak_count,
        reused = ss.reused_count,
        expired = ss.expired_count,
        breached = ss.breached_count,
        no_pwd = ss.no_password_count,
        expiring = ss.expiring_soon_count,
        vw = sd.very_weak,
        w = sd.weak,
        f = sd.fair,
        s = sd.strong,
        vs = sd.very_strong,
        np = sd.no_password,
        otp = analytics.feature_usage.with_otp,
        pk = analytics.feature_usage.with_passkey,
        ssh = analytics.feature_usage.with_ssh_key,
        att = analytics.feature_usage.with_attachment,
        fav = analytics.feature_usage.favorites,
    )
}
