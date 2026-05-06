//! Notification system — expiry warnings, breach alerts, sync errors

use crate::types::Entry;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    ExpiryWarning,
    BreachAlert,
    SyncError,
    EmergencyAccessRequest,
    UpdateAvailable,
    MasterKeyChangeRecommended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppNotification {
    pub id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub entry_uuid: Option<String>,
    pub created_at: String,
    pub read: bool,
    pub action_url: Option<String>,
}

impl AppNotification {
    pub fn new(
        notification_type: NotificationType,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            notification_type,
            title: title.into(),
            body: body.into(),
            entry_uuid: None,
            created_at: chrono::Utc::now().to_rfc3339(),
            read: false,
            action_url: None,
        }
    }

    pub fn with_entry(mut self, entry_uuid: impl Into<String>) -> Self {
        self.entry_uuid = Some(entry_uuid.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub expiry_warning_days: u32,
    pub breach_alerts: bool,
    pub sync_errors: bool,
    pub emergency_access_requests: bool,
    pub update_available: bool,
    pub use_system_notifications: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            expiry_warning_days: 14,
            breach_alerts: true,
            sync_errors: true,
            emergency_access_requests: true,
            update_available: true,
            use_system_notifications: true,
        }
    }
}

pub struct NotificationGenerator {
    settings: NotificationSettings,
}

impl NotificationGenerator {
    pub fn new(settings: NotificationSettings) -> Self {
        Self { settings }
    }

    pub fn check_expiry_warnings(&self, entries: &[&Entry]) -> Vec<AppNotification> {
        if !self.settings.enabled {
            return vec![];
        }

        let now = chrono::Utc::now();
        let threshold = chrono::Duration::days(self.settings.expiry_warning_days as i64);

        entries
            .iter()
            .filter_map(|entry| {
                let expiry = entry.expiry?;
                let title = entry.title.get();
                let uuid = entry.uuid.to_string();

                if expiry <= now {
                    Some(
                        AppNotification::new(
                            NotificationType::ExpiryWarning,
                            format!("\"{}\" has expired", title),
                            "This entry's password has expired. Update it now.",
                        )
                        .with_entry(uuid),
                    )
                } else if expiry - now <= threshold {
                    let days = (expiry - now).num_days();
                    Some(
                        AppNotification::new(
                            NotificationType::ExpiryWarning,
                            format!("\"{}\" expires in {} days", title, days),
                            format!(
                                "This entry will expire on {}. Consider updating the password.",
                                expiry.format("%Y-%m-%d")
                            ),
                        )
                        .with_entry(uuid),
                    )
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn breach_alert(
        &self,
        entry_title: &str,
        entry_uuid: &str,
        breach_count: u64,
    ) -> Option<AppNotification> {
        if !self.settings.enabled || !self.settings.breach_alerts {
            return None;
        }
        Some(AppNotification::new(
            NotificationType::BreachAlert,
            format!("Password breach detected: \"{}\"", entry_title),
            format!("This password was found {} time(s) in known data breaches. Change it immediately.", breach_count),
        ).with_entry(entry_uuid))
    }

    pub fn sync_error(&self, provider: &str, error: &str) -> Option<AppNotification> {
        if !self.settings.enabled || !self.settings.sync_errors {
            return None;
        }
        Some(AppNotification::new(
            NotificationType::SyncError,
            format!("Sync failed: {}", provider),
            error,
        ))
    }

    pub fn emergency_access_request(&self, requester_name: &str) -> Option<AppNotification> {
        if !self.settings.enabled || !self.settings.emergency_access_requests {
            return None;
        }
        Some(AppNotification::new(
            NotificationType::EmergencyAccessRequest,
            format!("{} requested emergency access", requester_name),
            "You have a waiting period to deny this request. Review it now.",
        ))
    }

    pub fn update_available(&self, version: &str) -> Option<AppNotification> {
        if !self.settings.enabled || !self.settings.update_available {
            return None;
        }
        Some(AppNotification::new(
            NotificationType::UpdateAvailable,
            format!("KeePassEx {} is available", version),
            "A new version of KeePassEx is ready to install.",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breach_alert_generated() {
        let gen = NotificationGenerator::new(NotificationSettings::default());
        let n = gen.breach_alert("Gmail", "uuid-1", 5432).unwrap();
        assert_eq!(n.notification_type, NotificationType::BreachAlert);
        assert!(n.body.contains("5432"));
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
    fn test_notifications_disabled_globally() {
        let gen = NotificationGenerator::new(NotificationSettings {
            enabled: false,
            ..Default::default()
        });
        assert!(gen.breach_alert("test", "uuid", 1).is_none());
        assert!(gen.sync_error("WebDAV", "error").is_none());
        assert!(gen.emergency_access_request("Bob").is_none());
    }

    #[test]
    fn test_notification_unique_id() {
        let gen = NotificationGenerator::new(NotificationSettings::default());
        let n1 = gen.breach_alert("t1", "u1", 1).unwrap();
        let n2 = gen.breach_alert("t2", "u2", 2).unwrap();
        assert_ne!(n1.id, n2.id);
    }
}
