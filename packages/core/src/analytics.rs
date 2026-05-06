//! Vault Analytics Engine — KeePassEx
//!
//! Computes insights about vault health, usage patterns, and security posture.
//! All computation is on-device — no data leaves the vault.
//!
//! # Metrics
//! - Password strength distribution
//! - Entry creation/modification timeline
//! - Most accessed entries
//! - Breach exposure history
//! - OTP usage statistics
//! - Group/tag distribution
//! - Password age statistics

use crate::error::{KeePassExError, Result};
use chrono::{DateTime, Datelike, Duration, Utc};
use std::collections::HashMap;

/// Complete analytics report for a vault
#[derive(Debug, Clone)]
pub struct VaultAnalytics {
    /// When this report was generated
    pub generated_at: DateTime<Utc>,
    /// Total entries analyzed
    pub total_entries: usize,
    /// Password strength distribution (0=VeryWeak..4=VeryStrong → count)
    pub strength_distribution: StrengthDistribution,
    /// Entry creation timeline (month → count)
    pub creation_timeline: Vec<TimelinePoint>,
    /// Entry modification timeline (month → count)
    pub modification_timeline: Vec<TimelinePoint>,
    /// Most accessed entries (top 10)
    pub most_accessed: Vec<AccessedEntry>,
    /// Password age statistics
    pub password_age: PasswordAgeStats,
    /// Group distribution (group_name → entry_count)
    pub group_distribution: Vec<GroupStat>,
    /// Tag distribution (tag → count)
    pub tag_distribution: Vec<TagStat>,
    /// Feature usage counts
    pub feature_usage: FeatureUsage,
    /// Security summary
    pub security_summary: SecuritySummary,
}

/// Password strength distribution
#[derive(Debug, Clone, Default)]
pub struct StrengthDistribution {
    pub very_weak: usize,   // score 0
    pub weak: usize,        // score 1
    pub fair: usize,        // score 2
    pub strong: usize,      // score 3
    pub very_strong: usize, // score 4
    pub no_password: usize,
}

impl StrengthDistribution {
    pub fn total(&self) -> usize {
        self.very_weak + self.weak + self.fair + self.strong + self.very_strong + self.no_password
    }

    pub fn percent_strong_or_better(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        (self.strong + self.very_strong) as f64 / total as f64 * 100.0
    }
}

/// A point on the timeline (month + count)
#[derive(Debug, Clone)]
pub struct TimelinePoint {
    /// Year
    pub year: i32,
    /// Month (1-12)
    pub month: u32,
    /// Count of entries
    pub count: usize,
    /// Label: "Jan 2025"
    pub label: String,
}

/// An accessed entry with access count
#[derive(Debug, Clone)]
pub struct AccessedEntry {
    pub uuid: String,
    pub title: String,
    pub access_count: usize,
    pub last_accessed: Option<DateTime<Utc>>,
}

/// Password age statistics
#[derive(Debug, Clone, Default)]
pub struct PasswordAgeStats {
    /// Average age in days
    pub average_days: f64,
    /// Oldest password age in days
    pub oldest_days: u64,
    /// Newest password age in days
    pub newest_days: u64,
    /// Count of passwords older than 1 year
    pub older_than_1_year: usize,
    /// Count of passwords older than 6 months
    pub older_than_6_months: usize,
    /// Count of passwords changed in last 30 days
    pub changed_last_30_days: usize,
}

/// Group statistics
#[derive(Debug, Clone)]
pub struct GroupStat {
    pub name: String,
    pub count: usize,
    pub percent: f64,
}

/// Tag statistics
#[derive(Debug, Clone)]
pub struct TagStat {
    pub tag: String,
    pub count: usize,
}

/// Feature usage counts
#[derive(Debug, Clone, Default)]
pub struct FeatureUsage {
    pub with_otp: usize,
    pub with_passkey: usize,
    pub with_ssh_key: usize,
    pub with_attachment: usize,
    pub with_custom_fields: usize,
    pub with_expiry: usize,
    pub favorites: usize,
}

/// Security summary
#[derive(Debug, Clone, Default)]
pub struct SecuritySummary {
    /// Overall health score (0-100)
    pub health_score: u8,
    /// Count of weak passwords
    pub weak_count: usize,
    /// Count of reused passwords
    pub reused_count: usize,
    /// Count of expired entries
    pub expired_count: usize,
    /// Count of entries expiring in 30 days
    pub expiring_soon_count: usize,
    /// Count of breached passwords
    pub breached_count: usize,
    /// Count of entries with no password
    pub no_password_count: usize,
}

impl SecuritySummary {
    /// Calculate health score (0-100) based on issues
    pub fn calculate_score(&mut self, total: usize) {
        if total == 0 {
            self.health_score = 100;
            return;
        }
        let issues = self.weak_count
            + self.reused_count
            + self.expired_count
            + self.breached_count * 3 // breaches are 3x worse
            + self.no_password_count;
        let issue_ratio = issues as f64 / total as f64;
        self.health_score = ((1.0 - issue_ratio.min(1.0)) * 100.0) as u8;
    }
}

/// Input data for analytics computation (abstracted from vault types)
#[derive(Debug, Clone)]
pub struct EntryAnalyticsData {
    pub uuid: String,
    pub title: String,
    pub group_name: String,
    pub tags: Vec<String>,
    pub password_score: Option<u8>, // 0-4, None = no password
    pub created_at: Option<DateTime<Utc>>,
    pub modified_at: Option<DateTime<Utc>>,
    pub accessed_at: Option<DateTime<Utc>>,
    pub access_count: usize,
    pub expires_at: Option<DateTime<Utc>>,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub has_ssh_key: bool,
    pub has_attachment: bool,
    pub has_custom_fields: bool,
    pub is_favorite: bool,
    pub is_breached: bool,
    pub is_reused: bool,
}

/// Compute full analytics from entry data.
///
/// # Arguments
/// * `entries` — Slice of entry analytics data
///
/// # Returns
/// Complete `VaultAnalytics` report
pub fn compute_analytics(entries: &[EntryAnalyticsData]) -> VaultAnalytics {
    let now = Utc::now();
    let total = entries.len();

    let strength_distribution = compute_strength_distribution(entries);
    let creation_timeline = compute_timeline(entries, TimelineType::Created, 12);
    let modification_timeline = compute_timeline(entries, TimelineType::Modified, 12);
    let most_accessed = compute_most_accessed(entries, 10);
    let password_age = compute_password_age(entries, now);
    let group_distribution = compute_group_distribution(entries);
    let tag_distribution = compute_tag_distribution(entries);
    let feature_usage = compute_feature_usage(entries);
    let security_summary = compute_security_summary(entries, total, now);

    VaultAnalytics {
        generated_at: now,
        total_entries: total,
        strength_distribution,
        creation_timeline,
        modification_timeline,
        most_accessed,
        password_age,
        group_distribution,
        tag_distribution,
        feature_usage,
        security_summary,
    }
}

// ─── Computation Functions ────────────────────────────────────────────────────

fn compute_strength_distribution(entries: &[EntryAnalyticsData]) -> StrengthDistribution {
    let mut dist = StrengthDistribution::default();
    for e in entries {
        match e.password_score {
            None => dist.no_password += 1,
            Some(0) => dist.very_weak += 1,
            Some(1) => dist.weak += 1,
            Some(2) => dist.fair += 1,
            Some(3) => dist.strong += 1,
            Some(4) | Some(_) => dist.very_strong += 1,
        }
    }
    dist
}

enum TimelineType {
    Created,
    Modified,
}

fn compute_timeline(
    entries: &[EntryAnalyticsData],
    timeline_type: TimelineType,
    months_back: i64,
) -> Vec<TimelinePoint> {
    let now = Utc::now();
    let mut month_counts: HashMap<(i32, u32), usize> = HashMap::new();

    // Initialize last N months with 0
    for i in 0..months_back {
        let dt = now - Duration::days(i * 30);
        month_counts.entry((dt.year(), dt.month())).or_insert(0);
    }

    for entry in entries {
        let dt = match timeline_type {
            TimelineType::Created => entry.created_at,
            TimelineType::Modified => entry.modified_at,
        };
        if let Some(dt) = dt {
            let key = (dt.year(), dt.month());
            if month_counts.contains_key(&key) {
                *month_counts.entry(key).or_insert(0) += 1;
            }
        }
    }

    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut points: Vec<TimelinePoint> = month_counts
        .into_iter()
        .map(|((year, month), count)| TimelinePoint {
            year,
            month,
            count,
            label: format!("{} {}", month_names[(month - 1) as usize], year),
        })
        .collect();

    // Sort chronologically
    points.sort_by(|a, b| a.year.cmp(&b.year).then(a.month.cmp(&b.month)));
    points
}

fn compute_most_accessed(entries: &[EntryAnalyticsData], limit: usize) -> Vec<AccessedEntry> {
    let mut sorted: Vec<&EntryAnalyticsData> = entries.iter().collect();
    sorted.sort_by(|a, b| b.access_count.cmp(&a.access_count));
    sorted
        .into_iter()
        .take(limit)
        .filter(|e| e.access_count > 0)
        .map(|e| AccessedEntry {
            uuid: e.uuid.clone(),
            title: e.title.clone(),
            access_count: e.access_count,
            last_accessed: e.accessed_at,
        })
        .collect()
}

fn compute_password_age(entries: &[EntryAnalyticsData], now: DateTime<Utc>) -> PasswordAgeStats {
    let mut stats = PasswordAgeStats::default();
    let mut ages: Vec<u64> = Vec::new();

    for entry in entries {
        if entry.password_score.is_none() {
            continue; // Skip entries without password
        }
        if let Some(modified) = entry.modified_at {
            let age_days = (now - modified).num_days().max(0) as u64;
            ages.push(age_days);

            if age_days > 365 {
                stats.older_than_1_year += 1;
            }
            if age_days > 180 {
                stats.older_than_6_months += 1;
            }
            if age_days <= 30 {
                stats.changed_last_30_days += 1;
            }
        }
    }

    if !ages.is_empty() {
        stats.average_days = ages.iter().sum::<u64>() as f64 / ages.len() as f64;
        stats.oldest_days = *ages.iter().max().unwrap_or(&0);
        stats.newest_days = *ages.iter().min().unwrap_or(&0);
    }

    stats
}

fn compute_group_distribution(entries: &[EntryAnalyticsData]) -> Vec<GroupStat> {
    let total = entries.len();
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        *counts.entry(entry.group_name.clone()).or_insert(0) += 1;
    }
    let mut stats: Vec<GroupStat> = counts
        .into_iter()
        .map(|(name, count)| GroupStat {
            percent: if total > 0 {
                count as f64 / total as f64 * 100.0
            } else {
                0.0
            },
            name,
            count,
        })
        .collect();
    stats.sort_by(|a, b| b.count.cmp(&a.count));
    stats
}

fn compute_tag_distribution(entries: &[EntryAnalyticsData]) -> Vec<TagStat> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        for tag in &entry.tags {
            *counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut stats: Vec<TagStat> = counts
        .into_iter()
        .map(|(tag, count)| TagStat { tag, count })
        .collect();
    stats.sort_by(|a, b| b.count.cmp(&a.count));
    stats
}

fn compute_feature_usage(entries: &[EntryAnalyticsData]) -> FeatureUsage {
    let mut usage = FeatureUsage::default();
    for e in entries {
        if e.has_otp {
            usage.with_otp += 1;
        }
        if e.has_passkey {
            usage.with_passkey += 1;
        }
        if e.has_ssh_key {
            usage.with_ssh_key += 1;
        }
        if e.has_attachment {
            usage.with_attachment += 1;
        }
        if e.has_custom_fields {
            usage.with_custom_fields += 1;
        }
        if e.expires_at.is_some() {
            usage.with_expiry += 1;
        }
        if e.is_favorite {
            usage.favorites += 1;
        }
    }
    usage
}

fn compute_security_summary(
    entries: &[EntryAnalyticsData],
    total: usize,
    now: DateTime<Utc>,
) -> SecuritySummary {
    let mut summary = SecuritySummary::default();
    let soon = now + Duration::days(30);

    for e in entries {
        if e.password_score.is_none() {
            summary.no_password_count += 1;
        } else if e.password_score.unwrap_or(4) <= 1 {
            summary.weak_count += 1;
        }
        if e.is_reused {
            summary.reused_count += 1;
        }
        if e.is_breached {
            summary.breached_count += 1;
        }
        if let Some(exp) = e.expires_at {
            if exp < now {
                summary.expired_count += 1;
            } else if exp < soon {
                summary.expiring_soon_count += 1;
            }
        }
    }

    summary.calculate_score(total);
    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(title: &str, score: Option<u8>, group: &str) -> EntryAnalyticsData {
        EntryAnalyticsData {
            uuid: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            group_name: group.to_string(),
            tags: vec![],
            password_score: score,
            created_at: Some(Utc::now() - Duration::days(30)),
            modified_at: Some(Utc::now() - Duration::days(10)),
            accessed_at: Some(Utc::now() - Duration::days(1)),
            access_count: 5,
            expires_at: None,
            has_otp: false,
            has_passkey: false,
            has_ssh_key: false,
            has_attachment: false,
            has_custom_fields: false,
            is_favorite: false,
            is_breached: false,
            is_reused: false,
        }
    }

    #[test]
    fn test_strength_distribution() {
        let entries = vec![
            make_entry("e1", Some(0), "Root"),
            make_entry("e2", Some(1), "Root"),
            make_entry("e3", Some(2), "Root"),
            make_entry("e4", Some(3), "Root"),
            make_entry("e5", Some(4), "Root"),
            make_entry("e6", None, "Root"),
        ];
        let analytics = compute_analytics(&entries);
        let dist = &analytics.strength_distribution;
        assert_eq!(dist.very_weak, 1);
        assert_eq!(dist.weak, 1);
        assert_eq!(dist.fair, 1);
        assert_eq!(dist.strong, 1);
        assert_eq!(dist.very_strong, 1);
        assert_eq!(dist.no_password, 1);
        assert_eq!(dist.total(), 6);
    }

    #[test]
    fn test_group_distribution() {
        let entries = vec![
            make_entry("e1", Some(3), "Banking"),
            make_entry("e2", Some(3), "Banking"),
            make_entry("e3", Some(3), "Social"),
        ];
        let analytics = compute_analytics(&entries);
        let banking = analytics
            .group_distribution
            .iter()
            .find(|g| g.name == "Banking");
        assert!(banking.is_some());
        assert_eq!(banking.unwrap().count, 2);
    }

    #[test]
    fn test_security_summary_health_score() {
        let mut entries: Vec<EntryAnalyticsData> = (0..10)
            .map(|i| make_entry(&format!("e{}", i), Some(4), "Root"))
            .collect();
        // Add 2 weak entries
        entries[0].password_score = Some(0);
        entries[1].password_score = Some(1);

        let analytics = compute_analytics(&entries);
        // 2/10 weak = 80% health
        assert!(analytics.security_summary.health_score >= 70);
        assert_eq!(analytics.security_summary.weak_count, 2);
    }

    #[test]
    fn test_feature_usage() {
        let mut entries = vec![
            make_entry("e1", Some(3), "Root"),
            make_entry("e2", Some(3), "Root"),
        ];
        entries[0].has_otp = true;
        entries[1].has_passkey = true;

        let analytics = compute_analytics(&entries);
        assert_eq!(analytics.feature_usage.with_otp, 1);
        assert_eq!(analytics.feature_usage.with_passkey, 1);
    }

    #[test]
    fn test_most_accessed() {
        let mut entries = vec![
            make_entry("e1", Some(3), "Root"),
            make_entry("e2", Some(3), "Root"),
        ];
        entries[0].access_count = 100;
        entries[1].access_count = 5;

        let analytics = compute_analytics(&entries);
        assert_eq!(analytics.most_accessed[0].title, "e1");
        assert_eq!(analytics.most_accessed[0].access_count, 100);
    }

    #[test]
    fn test_password_age_stats() {
        let mut entries = vec![make_entry("e1", Some(3), "Root")];
        entries[0].modified_at = Some(Utc::now() - Duration::days(400));

        let analytics = compute_analytics(&entries);
        assert_eq!(analytics.password_age.older_than_1_year, 1);
    }

    #[test]
    fn test_empty_vault() {
        let analytics = compute_analytics(&[]);
        assert_eq!(analytics.total_entries, 0);
        assert_eq!(analytics.security_summary.health_score, 100);
    }

    #[test]
    fn test_percent_strong_or_better() {
        let entries = vec![
            make_entry("e1", Some(3), "Root"),
            make_entry("e2", Some(4), "Root"),
            make_entry("e3", Some(1), "Root"),
            make_entry("e4", Some(0), "Root"),
        ];
        let analytics = compute_analytics(&entries);
        let pct = analytics.strength_distribution.percent_strong_or_better();
        assert!((pct - 50.0).abs() < 0.01);
    }
}
