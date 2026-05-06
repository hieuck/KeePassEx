//! Audit Log — track vault access and modification events
//!
//! Records who accessed what, when, and from which platform.
//! Used for emergency access auditing and security review.
//! The audit log is stored encrypted within the vault file.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- Types ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    VaultOpened,
    VaultLocked,
    VaultSaved,
    EntryViewed,
    EntryCreated,
    EntryModified,
    EntryDeleted,
    EntryMoved,
    PasswordCopied,
    OtpGenerated,
    EmergencyAccessRequested,
    EmergencyAccessGranted,
    EmergencyAccessRevoked,
    SyncCompleted,
    SyncFailed,
    ImportCompleted,
    ExportCompleted,
    MasterPasswordChanged,
    HardwareKeyAdded,
    HardwareKeyRemoved,
    BiometricUnlock,
    FailedUnlockAttempt,
    PluginInstalled,
    PluginUninstalled,
}

impl std::fmt::Display for AuditEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::VaultOpened => "vault_opened",
            Self::VaultLocked => "vault_locked",
            Self::VaultSaved => "vault_saved",
            Self::EntryViewed => "entry_viewed",
            Self::EntryCreated => "entry_created",
            Self::EntryModified => "entry_modified",
            Self::EntryDeleted => "entry_deleted",
            Self::EntryMoved => "entry_moved",
            Self::PasswordCopied => "password_copied",
            Self::OtpGenerated => "otp_generated",
            Self::EmergencyAccessRequested => "emergency_access_requested",
            Self::EmergencyAccessGranted => "emergency_access_granted",
            Self::EmergencyAccessRevoked => "emergency_access_revoked",
            Self::SyncCompleted => "sync_completed",
            Self::SyncFailed => "sync_failed",
            Self::ImportCompleted => "import_completed",
            Self::ExportCompleted => "export_completed",
            Self::MasterPasswordChanged => "master_password_changed",
            Self::HardwareKeyAdded => "hardware_key_added",
            Self::HardwareKeyRemoved => "hardware_key_removed",
            Self::BiometricUnlock => "biometric_unlock",
            Self::FailedUnlockAttempt => "failed_unlock_attempt",
            Self::PluginInstalled => "plugin_installed",
            Self::PluginUninstalled => "plugin_uninstalled",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub event_type: AuditEventType,
    pub timestamp: String,
    pub platform: String,
    pub entry_uuid: Option<String>,
    pub entry_title: Option<String>,
    pub details: Option<String>,
    pub ip_address: Option<String>,
}

impl AuditEvent {
    pub fn new(event_type: AuditEventType, platform: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: chrono::Utc::now().to_rfc3339(),
            platform: platform.into(),
            entry_uuid: None,
            entry_title: None,
            details: None,
            ip_address: None,
        }
    }

    pub fn with_entry(mut self, uuid: impl Into<String>, title: impl Into<String>) -> Self {
        self.entry_uuid = Some(uuid.into());
        self.entry_title = Some(title.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

// --- Audit Log ---

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
    max_events: usize,
}

impl AuditLog {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    pub fn record(&mut self, event: AuditEvent) {
        self.events.push(event);
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn filter_by_type(&self, event_type: &AuditEventType) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| &e.event_type == event_type)
            .collect()
    }

    pub fn filter_by_entry(&self, entry_uuid: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.entry_uuid.as_deref() == Some(entry_uuid))
            .collect()
    }

    pub fn recent(&self, count: usize) -> &[AuditEvent] {
        let start = self.events.len().saturating_sub(count);
        &self.events[start..]
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn failed_unlock_count(&self, since: &str) -> usize {
        self.events
            .iter()
            .filter(|e| {
                e.event_type == AuditEventType::FailedUnlockAttempt && e.timestamp.as_str() >= since
            })
            .count()
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_retrieve() {
        let mut log = AuditLog::new(100);
        log.record(AuditEvent::new(AuditEventType::VaultOpened, ""));
        log.record(AuditEvent::new(AuditEventType::EntryViewed, "").with_entry("", ""));
    }

    #[test]
    fn test_max_events_rotation() {
        let mut log = AuditLog::new(3);
        for _i in 0..5 {
            log.record(AuditEvent::new(AuditEventType::VaultOpened, ""));
        }
        assert!(log.events().len() <= 3);
    }

    #[test]
    fn test_filter_by_type() {
        let mut log = AuditLog::new(100);
        log.record(AuditEvent::new(AuditEventType::VaultOpened, ""));
        log.record(AuditEvent::new(AuditEventType::EntryViewed, ""));
        log.record(AuditEvent::new(AuditEventType::VaultOpened, ""));
        let opens = log.filter_by_type(&AuditEventType::VaultOpened);
        assert_eq!(opens.len(), 2);
    }

    #[test]
    fn test_filter_by_entry() {
        let mut log = AuditLog::new(100);
        log.record(AuditEvent::new(AuditEventType::EntryViewed, "").with_entry("uuid-1", "Gmail"));
        log.record(
            AuditEvent::new(AuditEventType::PasswordCopied, "").with_entry("uuid-2", "GitHub"),
        );
        log.record(AuditEvent::new(AuditEventType::EntryViewed, "").with_entry("uuid-1", "Gmail"));
        let gmail_events = log.filter_by_entry("uuid-1");
        assert_eq!(gmail_events.len(), 2);
    }

    #[test]
    fn test_failed_unlock_count() {
        let mut log = AuditLog::new(100);
        for _ in 0..3 {
            log.record(AuditEvent::new(AuditEventType::FailedUnlockAttempt, ""));
        }
        assert_eq!(
            log.filter_by_type(&AuditEventType::FailedUnlockAttempt)
                .len(),
            3
        );
    }

    #[test]
    fn test_event_type_display() {
        let event = AuditEvent::new(AuditEventType::VaultOpened, "desktop");
        assert_eq!(event.platform, "desktop");
    }

    #[test]
    fn test_recent() {
        let mut log = AuditLog::new(100);
        for _i in 0..10 {
            log.record(AuditEvent::new(AuditEventType::EntryViewed, ""));
        }
        let recent = log.recent(3);
        assert_eq!(recent.len(), 3);
    }
}
