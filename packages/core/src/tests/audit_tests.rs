//! Audit Log tests

use crate::audit_log::*;

#[test]
fn test_record_and_retrieve() {
    let mut log = AuditLog::new(100);
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "desktop"));
    log.record(
        AuditEvent::new(AuditEventType::EntryViewed, "desktop").with_entry("uuid-1", "Gmail"),
    );
    assert_eq!(log.len(), 2);
    assert!(!log.is_empty());
}

#[test]
fn test_max_events_rotation() {
    let mut log = AuditLog::new(3);
    for i in 0..5usize {
        log.record(
            AuditEvent::new(AuditEventType::VaultOpened, "desktop")
                .with_details(format!("event {}", i)),
        );
    }
    assert_eq!(log.len(), 3);
    // Oldest events should be dropped
    assert_eq!(log.events()[0].details.as_deref(), Some("event 2"));
    assert_eq!(log.events()[2].details.as_deref(), Some("event 4"));
}

#[test]
fn test_filter_by_type() {
    let mut log = AuditLog::new(100);
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "desktop"));
    log.record(AuditEvent::new(AuditEventType::EntryViewed, "desktop"));
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "mobile"));
    log.record(AuditEvent::new(AuditEventType::PasswordCopied, "desktop"));

    let opens = log.filter_by_type(&AuditEventType::VaultOpened);
    assert_eq!(opens.len(), 2);

    let views = log.filter_by_type(&AuditEventType::EntryViewed);
    assert_eq!(views.len(), 1);
}

#[test]
fn test_filter_by_entry() {
    let mut log = AuditLog::new(100);
    log.record(
        AuditEvent::new(AuditEventType::EntryViewed, "desktop").with_entry("uuid-1", "Gmail"),
    );
    log.record(
        AuditEvent::new(AuditEventType::PasswordCopied, "desktop").with_entry("uuid-1", "Gmail"),
    );
    log.record(
        AuditEvent::new(AuditEventType::EntryViewed, "desktop").with_entry("uuid-2", "GitHub"),
    );

    let gmail_events = log.filter_by_entry("uuid-1");
    assert_eq!(gmail_events.len(), 2);

    let github_events = log.filter_by_entry("uuid-2");
    assert_eq!(github_events.len(), 1);
}

#[test]
fn test_recent() {
    let mut log = AuditLog::new(100);
    for i in 0..10usize {
        log.record(
            AuditEvent::new(AuditEventType::EntryViewed, "desktop")
                .with_details(format!("event {}", i)),
        );
    }
    let recent = log.recent(3);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[2].details.as_deref(), Some("event 9"));
}

#[test]
fn test_recent_fewer_than_requested() {
    let mut log = AuditLog::new(100);
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "desktop"));
    let recent = log.recent(10);
    assert_eq!(recent.len(), 1);
}

#[test]
fn test_failed_unlock_count() {
    let mut log = AuditLog::new(100);
    let past = "2020-01-01T00:00:00Z";
    for _ in 0..3 {
        log.record(AuditEvent::new(
            AuditEventType::FailedUnlockAttempt,
            "desktop",
        ));
    }
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "desktop"));
    assert_eq!(log.failed_unlock_count(past), 3);
}

#[test]
fn test_clear() {
    let mut log = AuditLog::new(100);
    log.record(AuditEvent::new(AuditEventType::VaultOpened, "desktop"));
    log.record(AuditEvent::new(AuditEventType::VaultLocked, "desktop"));
    assert_eq!(log.len(), 2);
    log.clear();
    assert_eq!(log.len(), 0);
    assert!(log.is_empty());
}

#[test]
fn test_event_type_display() {
    assert_eq!(AuditEventType::VaultOpened.to_string(), "vault_opened");
    assert_eq!(
        AuditEventType::PasswordCopied.to_string(),
        "password_copied"
    );
    assert_eq!(
        AuditEventType::FailedUnlockAttempt.to_string(),
        "failed_unlock_attempt"
    );
    assert_eq!(
        AuditEventType::EmergencyAccessGranted.to_string(),
        "emergency_access_granted"
    );
}

#[test]
fn test_event_with_entry() {
    let event =
        AuditEvent::new(AuditEventType::EntryViewed, "desktop").with_entry("uuid-123", "GitHub");
    assert_eq!(event.entry_uuid.as_deref(), Some("uuid-123"));
    assert_eq!(event.entry_title.as_deref(), Some("GitHub"));
}

#[test]
fn test_event_with_details() {
    let event = AuditEvent::new(AuditEventType::SyncCompleted, "desktop")
        .with_details("WebDAV sync: 5 entries uploaded");
    assert_eq!(
        event.details.as_deref(),
        Some("WebDAV sync: 5 entries uploaded")
    );
}

#[test]
fn test_event_has_unique_id() {
    let e1 = AuditEvent::new(AuditEventType::VaultOpened, "desktop");
    let e2 = AuditEvent::new(AuditEventType::VaultOpened, "desktop");
    assert_ne!(e1.id, e2.id);
}

#[test]
fn test_event_has_timestamp() {
    let event = AuditEvent::new(AuditEventType::VaultOpened, "desktop");
    assert!(!event.timestamp.is_empty());
    // Should be valid RFC3339
    assert!(chrono::DateTime::parse_from_rfc3339(&event.timestamp).is_ok());
}

#[test]
fn test_all_event_types_have_display() {
    let types = [
        AuditEventType::VaultOpened,
        AuditEventType::VaultLocked,
        AuditEventType::VaultSaved,
        AuditEventType::EntryViewed,
        AuditEventType::EntryCreated,
        AuditEventType::EntryModified,
        AuditEventType::EntryDeleted,
        AuditEventType::EntryMoved,
        AuditEventType::PasswordCopied,
        AuditEventType::OtpGenerated,
        AuditEventType::EmergencyAccessRequested,
        AuditEventType::EmergencyAccessGranted,
        AuditEventType::EmergencyAccessRevoked,
        AuditEventType::SyncCompleted,
        AuditEventType::SyncFailed,
        AuditEventType::ImportCompleted,
        AuditEventType::ExportCompleted,
        AuditEventType::MasterPasswordChanged,
        AuditEventType::HardwareKeyAdded,
        AuditEventType::HardwareKeyRemoved,
        AuditEventType::BiometricUnlock,
        AuditEventType::FailedUnlockAttempt,
        AuditEventType::PluginInstalled,
        AuditEventType::PluginUninstalled,
    ];
    for t in &types {
        assert!(
            !t.to_string().is_empty(),
            "Event type {:?} has empty display",
            t
        );
    }
}
