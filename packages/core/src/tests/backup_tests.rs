//! Scheduled Backup tests

use crate::scheduled_backup::*;

fn make_config(enabled: bool, freq: BackupFrequency, last_backup: Option<&str>) -> BackupConfig {
    BackupConfig {
        enabled,
        frequency: freq,
        destination: "/tmp/backups".to_string(),
        max_backups: 5,
        timestamp_in_filename: true,
        last_backup_at: last_backup.map(|s| s.to_string()),
    }
}

#[test]
fn test_backup_due_never_backed_up() {
    let config = make_config(true, BackupFrequency::Daily, None);
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_disabled() {
    let config = make_config(false, BackupFrequency::Daily, None);
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_not_due_empty_destination() {
    let mut config = make_config(true, BackupFrequency::Daily, None);
    config.destination = String::new();
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_on_save_always() {
    let now = chrono::Utc::now().to_rfc3339();
    let config = make_config(true, BackupFrequency::OnSave, Some(&now));
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_daily_recent() {
    let recent = (chrono::Utc::now() - chrono::Duration::hours(2)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Daily, Some(&recent));
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_daily_old() {
    let old = (chrono::Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Daily, Some(&old));
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_weekly_recent() {
    let recent = (chrono::Utc::now() - chrono::Duration::days(3)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Weekly, Some(&recent));
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_weekly_old() {
    let old = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Weekly, Some(&old));
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_monthly_recent() {
    let recent = (chrono::Utc::now() - chrono::Duration::days(15)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Monthly, Some(&recent));
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_monthly_old() {
    let old = (chrono::Utc::now() - chrono::Duration::days(31)).to_rfc3339();
    let config = make_config(true, BackupFrequency::Monthly, Some(&old));
    assert!(is_backup_due(&config));
}

#[test]
fn test_frequency_display() {
    assert_eq!(BackupFrequency::OnSave.to_string(), "On Save");
    assert_eq!(BackupFrequency::Daily.to_string(), "Daily");
    assert_eq!(BackupFrequency::Weekly.to_string(), "Weekly");
    assert_eq!(BackupFrequency::Monthly.to_string(), "Monthly");
}

#[test]
fn test_default_config() {
    let config = BackupConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.max_backups, 10);
    assert!(config.timestamp_in_filename);
    assert!(config.destination.is_empty());
    assert!(config.last_backup_at.is_none());
}

#[test]
fn test_backup_result_success() {
    let result = BackupResult {
        success: true,
        backup_path: Some("/tmp/vault_backup_20250506.kdbx".to_string()),
        error: None,
        backups_pruned: 2,
    };
    assert!(result.success);
    assert!(result.backup_path.is_some());
    assert_eq!(result.backups_pruned, 2);
}

#[test]
fn test_backup_result_failure() {
    let result = BackupResult {
        success: false,
        backup_path: None,
        error: Some("Permission denied".to_string()),
        backups_pruned: 0,
    };
    assert!(!result.success);
    assert!(result.error.is_some());
}

#[test]
fn test_invalid_timestamp_treated_as_never() {
    let mut config = make_config(true, BackupFrequency::Daily, None);
    config.last_backup_at = Some("not-a-valid-timestamp".to_string());
    // Invalid timestamp should be treated as never backed up
    assert!(is_backup_due(&config));
}
