//! Scheduled backup tests — uses the public is_backup_due() API

use crate::scheduled_backup::{is_backup_due, BackupConfig, BackupFrequency};

#[test]
fn test_default_backup_config() {
    let config = BackupConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.max_backups, 10);
    assert!(config.timestamp_in_filename);
}

#[test]
fn test_backup_config_serialization() {
    let config = BackupConfig {
        enabled: true,
        destination: "/tmp/backups".to_string(),
        frequency: BackupFrequency::Daily,
        max_backups: 5,
        timestamp_in_filename: true,
        last_backup_at: None,
    };
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: BackupConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.max_backups, config.max_backups);
}

#[test]
fn test_backup_frequency_variants_serialize() {
    let freqs = [
        BackupFrequency::OnSave,
        BackupFrequency::Daily,
        BackupFrequency::Weekly,
        BackupFrequency::Monthly,
    ];
    for freq in &freqs {
        let json = serde_json::to_string(freq).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_backup_due_when_never_backed_up() {
    let config = BackupConfig {
        enabled: true,
        destination: "/tmp/backups".to_string(),
        frequency: BackupFrequency::Daily,
        max_backups: 10,
        timestamp_in_filename: true,
        last_backup_at: None,
    };
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_when_disabled() {
    let config = BackupConfig {
        enabled: false,
        ..BackupConfig::default()
    };
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_not_due_empty_destination() {
    let config = BackupConfig {
        enabled: true,
        destination: String::new(),
        ..BackupConfig::default()
    };
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_on_save_always() {
    let config = BackupConfig {
        enabled: true,
        frequency: BackupFrequency::OnSave,
        destination: "/tmp/backups".to_string(),
        last_backup_at: Some(chrono::Utc::now().to_rfc3339()),
        ..BackupConfig::default()
    };
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_daily_recent() {
    let recent = (chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
    let config = BackupConfig {
        enabled: true,
        frequency: BackupFrequency::Daily,
        destination: "/tmp/backups".to_string(),
        last_backup_at: Some(recent),
        ..BackupConfig::default()
    };
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_daily_old() {
    let old = (chrono::Utc::now() - chrono::Duration::hours(25)).to_rfc3339();
    let config = BackupConfig {
        enabled: true,
        frequency: BackupFrequency::Daily,
        destination: "/tmp/backups".to_string(),
        last_backup_at: Some(old),
        ..BackupConfig::default()
    };
    assert!(is_backup_due(&config));
}

#[test]
fn test_backup_not_due_weekly_recent() {
    let recent = (chrono::Utc::now() - chrono::Duration::days(3)).to_rfc3339();
    let config = BackupConfig {
        enabled: true,
        frequency: BackupFrequency::Weekly,
        destination: "/tmp/backups".to_string(),
        last_backup_at: Some(recent),
        ..BackupConfig::default()
    };
    assert!(!is_backup_due(&config));
}

#[test]
fn test_backup_due_weekly_old() {
    let old = (chrono::Utc::now() - chrono::Duration::days(8)).to_rfc3339();
    let config = BackupConfig {
        enabled: true,
        frequency: BackupFrequency::Weekly,
        destination: "/tmp/backups".to_string(),
        last_backup_at: Some(old),
        ..BackupConfig::default()
    };
    assert!(is_backup_due(&config));
}

#[test]
fn test_frequency_display() {
    assert_eq!(BackupFrequency::Daily.to_string(), "Daily");
    assert_eq!(BackupFrequency::OnSave.to_string(), "On Save");
    assert_eq!(BackupFrequency::Weekly.to_string(), "Weekly");
    assert_eq!(BackupFrequency::Monthly.to_string(), "Monthly");
}
