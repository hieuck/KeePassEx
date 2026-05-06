//! Analytics module tests — uses the public compute_analytics() API

use crate::analytics::{compute_analytics, EntryAnalyticsData};
use chrono::{Duration, Utc};

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
        access_count: 0,
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
fn test_analytics_empty_vault() {
    let analytics = compute_analytics(&[]);
    assert_eq!(analytics.total_entries, 0);
    assert_eq!(analytics.security_summary.health_score, 100);
}

#[test]
fn test_analytics_entry_count() {
    let entries = vec![
        make_entry("e1", Some(3), "Root"),
        make_entry("e2", Some(4), "Root"),
        make_entry("e3", None, "Root"),
    ];
    let analytics = compute_analytics(&entries);
    assert_eq!(analytics.total_entries, 3);
}

#[test]
fn test_analytics_strength_distribution() {
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
fn test_analytics_no_password_count() {
    let entries = vec![
        make_entry("e1", Some(3), "Root"),
        make_entry("e2", None, "Root"),
        make_entry("e3", None, "Root"),
    ];
    let analytics = compute_analytics(&entries);
    assert_eq!(analytics.security_summary.no_password_count, 2);
}

#[test]
fn test_analytics_group_distribution() {
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
fn test_analytics_feature_usage_otp() {
    let mut entries = vec![
        make_entry("e1", Some(3), "Root"),
        make_entry("e2", Some(3), "Root"),
    ];
    entries[0].has_otp = true;
    let analytics = compute_analytics(&entries);
    assert_eq!(analytics.feature_usage.with_otp, 1);
}

#[test]
fn test_analytics_most_accessed() {
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
fn test_analytics_password_age_old() {
    let mut entries = vec![make_entry("e1", Some(3), "Root")];
    entries[0].modified_at = Some(Utc::now() - Duration::days(400));
    let analytics = compute_analytics(&entries);
    assert_eq!(analytics.password_age.older_than_1_year, 1);
}

#[test]
fn test_analytics_percent_strong_or_better() {
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

#[test]
fn test_analytics_security_summary_weak_count() {
    let entries = vec![
        make_entry("e1", Some(0), "Root"),
        make_entry("e2", Some(1), "Root"),
        make_entry("e3", Some(4), "Root"),
    ];
    let analytics = compute_analytics(&entries);
    assert_eq!(analytics.security_summary.weak_count, 2);
}

#[test]
fn test_analytics_tag_distribution() {
    let mut entries = vec![
        make_entry("e1", Some(3), "Root"),
        make_entry("e2", Some(3), "Root"),
    ];
    entries[0].tags = vec!["finance".to_string(), "important".to_string()];
    entries[1].tags = vec!["finance".to_string()];
    let analytics = compute_analytics(&entries);
    let finance = analytics
        .tag_distribution
        .iter()
        .find(|t| t.tag == "finance");
    assert!(finance.is_some());
    assert_eq!(finance.unwrap().count, 2);
}
