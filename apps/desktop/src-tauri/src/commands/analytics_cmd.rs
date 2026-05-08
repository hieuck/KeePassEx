//! Tauri commands — Vault Analytics
use keepassex_core::analytics::{compute_analytics, EntryAnalyticsData};
use serde::Serialize;
use tauri::{command, State};

use crate::state::AppState;

// ─── Serializable DTOs ────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct VaultAnalyticsDto {
    pub generated_at: String,
    pub total_entries: usize,
    pub strength_distribution: StrengthDistributionDto,
    pub security_summary: SecuritySummaryDto,
    pub feature_usage: FeatureUsageDto,
    pub password_age: PasswordAgeDto,
    pub group_distribution: Vec<GroupStatDto>,
    pub tag_distribution: Vec<TagStatDto>,
    pub most_accessed: Vec<AccessedEntryDto>,
}

#[derive(Debug, Serialize)]
pub struct StrengthDistributionDto {
    pub very_weak: usize,
    pub weak: usize,
    pub fair: usize,
    pub strong: usize,
    pub very_strong: usize,
    pub no_password: usize,
    pub percent_strong_or_better: f64,
}

#[derive(Debug, Serialize)]
pub struct SecuritySummaryDto {
    pub health_score: u8,
    pub weak_count: usize,
    pub reused_count: usize,
    pub expired_count: usize,
    pub expiring_soon_count: usize,
    pub breached_count: usize,
    pub no_password_count: usize,
}

#[derive(Debug, Serialize)]
pub struct FeatureUsageDto {
    pub with_otp: usize,
    pub with_passkey: usize,
    pub with_ssh_key: usize,
    pub with_attachment: usize,
    pub with_custom_fields: usize,
    pub with_expiry: usize,
    pub favorites: usize,
}

#[derive(Debug, Serialize)]
pub struct PasswordAgeDto {
    pub average_days: f64,
    pub oldest_days: u64,
    pub newest_days: u64,
    pub older_than_1_year: usize,
    pub older_than_6_months: usize,
    pub changed_last_30_days: usize,
}

#[derive(Debug, Serialize)]
pub struct GroupStatDto {
    pub name: String,
    pub count: usize,
    pub percent: f64,
}

#[derive(Debug, Serialize)]
pub struct TagStatDto {
    pub tag: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct AccessedEntryDto {
    pub uuid: String,
    pub title: String,
    pub access_count: usize,
    pub last_accessed: Option<String>,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// Get full vault analytics report
#[command]
pub fn get_vault_analytics(state: State<'_, AppState>) -> Result<VaultAnalyticsDto, String> {
    let vault_guard = state.vault.read().map_err(|e| e.to_string())?;
    let open_vault = vault_guard.as_ref().ok_or("Vault not open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let vault = &open_vault.vault;

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

    let a = compute_analytics(&entries);

    Ok(VaultAnalyticsDto {
        generated_at: a.generated_at.to_rfc3339(),
        total_entries: a.total_entries,
        strength_distribution: StrengthDistributionDto {
            very_weak: a.strength_distribution.very_weak,
            weak: a.strength_distribution.weak,
            fair: a.strength_distribution.fair,
            strong: a.strength_distribution.strong,
            very_strong: a.strength_distribution.very_strong,
            no_password: a.strength_distribution.no_password,
            percent_strong_or_better: a.strength_distribution.percent_strong_or_better(),
        },
        security_summary: SecuritySummaryDto {
            health_score: a.security_summary.health_score,
            weak_count: a.security_summary.weak_count,
            reused_count: a.security_summary.reused_count,
            expired_count: a.security_summary.expired_count,
            expiring_soon_count: a.security_summary.expiring_soon_count,
            breached_count: a.security_summary.breached_count,
            no_password_count: a.security_summary.no_password_count,
        },
        feature_usage: FeatureUsageDto {
            with_otp: a.feature_usage.with_otp,
            with_passkey: a.feature_usage.with_passkey,
            with_ssh_key: a.feature_usage.with_ssh_key,
            with_attachment: a.feature_usage.with_attachment,
            with_custom_fields: a.feature_usage.with_custom_fields,
            with_expiry: a.feature_usage.with_expiry,
            favorites: a.feature_usage.favorites,
        },
        password_age: PasswordAgeDto {
            average_days: a.password_age.average_days,
            oldest_days: a.password_age.oldest_days,
            newest_days: a.password_age.newest_days,
            older_than_1_year: a.password_age.older_than_1_year,
            older_than_6_months: a.password_age.older_than_6_months,
            changed_last_30_days: a.password_age.changed_last_30_days,
        },
        group_distribution: a
            .group_distribution
            .into_iter()
            .map(|g| GroupStatDto {
                name: g.name,
                count: g.count,
                percent: g.percent,
            })
            .collect(),
        tag_distribution: a
            .tag_distribution
            .into_iter()
            .map(|t| TagStatDto {
                tag: t.tag,
                count: t.count,
            })
            .collect(),
        most_accessed: a
            .most_accessed
            .into_iter()
            .map(|e| AccessedEntryDto {
                uuid: e.uuid,
                title: e.title,
                access_count: e.access_count,
                last_accessed: e.last_accessed.map(|d| d.to_rfc3339()),
            })
            .collect(),
    })
}

/// Export analytics report as HTML
#[command]
pub fn export_analytics_report(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<String, String> {
    let analytics = get_vault_analytics(state)?;
    let ss = &analytics.security_summary;
    let sd = &analytics.strength_distribution;
    let score_color = if ss.health_score >= 80 {
        "#16a34a"
    } else if ss.health_score >= 60 {
        "#d97706"
    } else {
        "#dc2626"
    };

    let html = format!(
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8">
<title>KeePassEx Analytics</title>
<style>body{{font-family:system-ui;margin:40px}}h1{{color:#2563eb}}
.score{{font-size:48px;font-weight:bold;color:{score_color}}}
.grid{{display:grid;grid-template-columns:repeat(3,1fr);gap:16px;margin:20px 0}}
.card{{background:#f9fafb;border:1px solid #e5e7eb;border-radius:8px;padding:16px}}
.val{{font-size:24px;font-weight:bold}}</style></head>
<body><h1>🔐 KeePassEx Vault Analytics</h1>
<p>Generated: {gen} | Entries: {total}</p>
<h2>Health Score</h2><div class="score">{score}/100</div>
<h2>Summary</h2><div class="grid">
<div class="card"><div class="val">{weak}</div>Weak</div>
<div class="card"><div class="val">{reused}</div>Reused</div>
<div class="card"><div class="val">{expired}</div>Expired</div>
<div class="card"><div class="val">{breached}</div>Breached</div>
<div class="card"><div class="val">{no_pwd}</div>No Password</div>
<div class="card"><div class="val">{expiring}</div>Expiring Soon</div>
</div>
<h2>Strength</h2>
<p>Very Weak:{vw} Weak:{w} Fair:{f} Strong:{s} Very Strong:{vs}</p>
<h2>Features</h2>
<p>OTP:{otp} Passkeys:{pk} SSH:{ssh} Attachments:{att}</p>
<footer style="margin-top:40px;color:#9ca3af;font-size:12px">KeePassEx — Your passwords, your control</footer>
</body></html>"#,
        score_color = score_color,
        gen = analytics.generated_at,
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
        otp = analytics.feature_usage.with_otp,
        pk = analytics.feature_usage.with_passkey,
        ssh = analytics.feature_usage.with_ssh_key,
        att = analytics.feature_usage.with_attachment,
    );

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
