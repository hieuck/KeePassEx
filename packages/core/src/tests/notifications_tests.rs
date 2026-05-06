//! Notifications tests — uses the public NotificationGenerator API

use crate::notifications::{NotificationGenerator, NotificationSettings, NotificationType};

#[test]
fn test_default_notification_settings() {
    let settings = NotificationSettings::default();
    assert!(settings.enabled);
    assert!(settings.breach_alerts);
    assert!(settings.sync_errors);
    assert_eq!(settings.expiry_warning_days, 14);
}

#[test]
fn test_breach_alert_generated() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n = gen.breach_alert("Gmail", "uuid-1", 5432).unwrap();
    assert_eq!(n.notification_type, NotificationType::BreachAlert);
    assert!(n.body.contains("5432"));
    assert_eq!(n.entry_uuid, Some("uuid-1".to_string()));
}

#[test]
fn test_breach_alert_disabled() {
    let gen = NotificationGenerator::new(NotificationSettings {
        breach_alerts: false,
        ..Default::default()
    });
    assert!(gen.breach_alert("Gmail", "uuid-1", 100).is_none());
}

#[test]
fn test_sync_error_notification() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n = gen.sync_error("WebDAV", "Connection refused").unwrap();
    assert_eq!(n.notification_type, NotificationType::SyncError);
    assert!(n.body.contains("Connection refused"));
}

#[test]
fn test_sync_error_disabled() {
    let gen = NotificationGenerator::new(NotificationSettings {
        sync_errors: false,
        ..Default::default()
    });
    assert!(gen.sync_error("WebDAV", "error").is_none());
}

#[test]
fn test_emergency_access_notification() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n = gen.emergency_access_request("Alice").unwrap();
    assert_eq!(
        n.notification_type,
        NotificationType::EmergencyAccessRequest
    );
    assert!(n.title.contains("Alice"));
}

#[test]
fn test_emergency_access_disabled() {
    let gen = NotificationGenerator::new(NotificationSettings {
        emergency_access_requests: false,
        ..Default::default()
    });
    assert!(gen.emergency_access_request("Bob").is_none());
}

#[test]
fn test_update_available_notification() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n = gen.update_available("2.0.0").unwrap();
    assert_eq!(n.notification_type, NotificationType::UpdateAvailable);
    assert!(n.title.contains("2.0.0"));
}

#[test]
fn test_notifications_disabled_globally() {
    let gen = NotificationGenerator::new(NotificationSettings {
        enabled: false,
        ..Default::default()
    });
    assert!(gen.breach_alert("test", "uuid", 1).is_none());
    assert!(gen.sync_error("WebDAV", "error").is_none());
    assert!(gen.emergency_access_request("Bob").is_none());
    assert!(gen.update_available("1.0.0").is_none());
}

#[test]
fn test_notification_unique_id() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n1 = gen.breach_alert("t1", "u1", 1).unwrap();
    let n2 = gen.breach_alert("t2", "u2", 2).unwrap();
    assert_ne!(n1.id, n2.id);
}

#[test]
fn test_notification_not_read_by_default() {
    let gen = NotificationGenerator::new(NotificationSettings::default());
    let n = gen.breach_alert("Gmail", "uuid", 1).unwrap();
    assert!(!n.read);
}

#[test]
fn test_notification_settings_serialization() {
    let settings = NotificationSettings::default();
    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: NotificationSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(
        deserialized.expiry_warning_days,
        settings.expiry_warning_days
    );
    assert_eq!(deserialized.enabled, settings.enabled);
}
