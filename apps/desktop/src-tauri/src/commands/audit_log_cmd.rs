//! Audit Log Tauri commands

use crate::state::AppState;
use keepassex_core::audit_log::{AuditEvent, AuditEventType, AuditLog};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct AuditEventDto {
    pub id: String,
    pub event_type: String,
    pub timestamp: String,
    pub platform: String,
    pub entry_uuid: Option<String>,
    pub entry_title: Option<String>,
    pub details: Option<String>,
}

impl From<&AuditEvent> for AuditEventDto {
    fn from(e: &AuditEvent) -> Self {
        Self {
            id: e.id.clone(),
            event_type: e.event_type.to_string(),
            timestamp: e.timestamp.clone(),
            platform: e.platform.clone(),
            entry_uuid: e.entry_uuid.clone(),
            entry_title: e.entry_title.clone(),
            details: e.details.clone(),
        }
    }
}

/// Get recent audit log events
#[tauri::command]
pub fn get_audit_log(
    limit: usize,
    state: State<'_, AppState>,
) -> Result<Vec<AuditEventDto>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let events = open_vault.vault.audit_log.recent(limit);
    Ok(events.iter().map(AuditEventDto::from).collect())
}

/// Clear the audit log
#[tauri::command]
pub fn clear_audit_log(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;
    open_vault.vault.audit_log.clear();
    open_vault.vault.dirty = true;
    Ok(())
}

/// Export audit log as JSON
#[tauri::command]
pub fn export_audit_log(state: State<'_, AppState>) -> Result<String, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let events: Vec<AuditEventDto> = open_vault
        .vault
        .audit_log
        .events()
        .iter()
        .map(AuditEventDto::from)
        .collect();

    serde_json::to_string_pretty(&events).map_err(|e| e.to_string())
}

/// Record an audit event (called from frontend for UI-level events)
#[tauri::command]
pub fn record_audit_event(
    event_type: String,
    entry_uuid: Option<String>,
    entry_title: Option<String>,
    details: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let event_type_parsed = match event_type.as_str() {
        "vault_opened" => AuditEventType::VaultOpened,
        "vault_locked" => AuditEventType::VaultLocked,
        "entry_viewed" => AuditEventType::EntryViewed,
        "password_copied" => AuditEventType::PasswordCopied,
        "otp_generated" => AuditEventType::OtpGenerated,
        _ => return Ok(()), // Unknown event types are silently ignored
    };

    let mut event = AuditEvent::new(event_type_parsed, "desktop");
    if let (Some(uuid), Some(title)) = (entry_uuid, entry_title) {
        event = event.with_entry(uuid, title);
    }
    if let Some(d) = details {
        event = event.with_details(d);
    }

    let mut vault_lock = state.vault.write().unwrap();
    if let Some(open_vault) = vault_lock.as_mut() {
        open_vault.vault.audit_log.record(event);
    }

    Ok(())
}
