//! Emergency access tests

use crate::emergency_access::*;
use uuid::Uuid;

fn make_access(wait_days: u32) -> EmergencyAccess {
    EmergencyAccess::new(
        "grantee@example.com".to_string(),
        "Alice".to_string(),
        "alice@example.com".to_string(),
        vec![0x01, 0x02, 0x03, 0x04], // mock public key
        EmergencyAccessLevel::View,
        wait_days,
    )
}

#[test]
fn test_new_access_is_invited() {
    let access = make_access(7);
    assert_eq!(access.status, EmergencyAccessStatus::Invited);
    assert!(access.request_initiated_at.is_none());
    assert!(access.encrypted_vault_key.is_none());
    assert_eq!(access.wait_time_days, 7);
}

#[test]
fn test_initiate_request_requires_confirmed() {
    let mut access = make_access(7);
    // Cannot initiate from Invited state
    assert!(access.initiate_request().is_err());
}

#[test]
fn test_full_lifecycle() {
    let mut access = make_access(7);

    // Confirm
    access.status = EmergencyAccessStatus::Confirmed;

    // Initiate request
    access.initiate_request().unwrap();
    assert_eq!(access.status, EmergencyAccessStatus::RecoveryInitiated);
    assert!(access.request_initiated_at.is_some());

    // Approve
    access.approve_request().unwrap();
    assert_eq!(access.status, EmergencyAccessStatus::RecoveryApproved);

    // Wait period not elapsed (just approved)
    assert!(!access.is_wait_period_elapsed());
    assert_eq!(access.days_remaining(), Some(7));
}

#[test]
fn test_approve_requires_initiated() {
    let mut access = make_access(7);
    access.status = EmergencyAccessStatus::Confirmed;
    // Cannot approve without initiating first
    assert!(access.approve_request().is_err());
}

#[test]
fn test_revoke_clears_sensitive_data() {
    let mut access = make_access(7);
    access.status = EmergencyAccessStatus::Confirmed;
    access.initiate_request().unwrap();
    access.approve_request().unwrap();

    access.revoke();

    assert_eq!(access.status, EmergencyAccessStatus::Revoked);
    assert!(access.request_initiated_at.is_none());
    assert!(access.encrypted_vault_key.is_none());
}

#[test]
fn test_days_remaining_none_when_not_approved() {
    let access = make_access(7);
    assert!(access.days_remaining().is_none());
}

#[test]
fn test_days_remaining_when_approved() {
    let mut access = make_access(7);
    access.status = EmergencyAccessStatus::Confirmed;
    access.initiate_request().unwrap();
    access.approve_request().unwrap();

    let remaining = access.days_remaining();
    assert!(remaining.is_some());
    assert_eq!(remaining.unwrap(), 7); // Just approved, full wait period
}

#[test]
fn test_manager_add_and_get() {
    let mut manager = EmergencyAccessManager::new();
    let access = make_access(7);
    let id = access.id;

    manager.add_grant(access);

    assert!(manager.get_grant(&id).is_some());
    assert!(manager.get_grant(&Uuid::new_v4()).is_none());
}

#[test]
fn test_manager_remove_grant() {
    let mut manager = EmergencyAccessManager::new();
    let access = make_access(7);
    let id = access.id;

    manager.add_grant(access);
    assert_eq!(manager.grants.len(), 1);

    manager.remove_grant(&id);
    assert_eq!(manager.grants.len(), 0);
}

#[test]
fn test_manager_active_grants_excludes_revoked() {
    let mut manager = EmergencyAccessManager::new();

    let active = make_access(7);
    let mut revoked = make_access(3);
    revoked.revoke();

    manager.add_grant(active);
    manager.add_grant(revoked);

    assert_eq!(manager.grants.len(), 2);
    assert_eq!(manager.active_grants().len(), 1);
}

#[test]
fn test_manager_multiple_grants() {
    let mut manager = EmergencyAccessManager::new();

    for i in 0..5 {
        let mut access = EmergencyAccess::new(
            format!("user{}@example.com", i),
            format!("User {}", i),
            format!("user{}@example.com", i),
            vec![i as u8],
            EmergencyAccessLevel::View,
            7,
        );
        manager.add_grant(access);
    }

    assert_eq!(manager.grants.len(), 5);
    assert_eq!(manager.active_grants().len(), 5);
}

#[test]
fn test_access_level_takeover() {
    let access = EmergencyAccess::new(
        "grantee@example.com".to_string(),
        "Bob".to_string(),
        "bob@example.com".to_string(),
        vec![],
        EmergencyAccessLevel::Takeover,
        14,
    );

    assert_eq!(access.access_level, EmergencyAccessLevel::Takeover);
    assert_eq!(access.wait_time_days, 14);
}
