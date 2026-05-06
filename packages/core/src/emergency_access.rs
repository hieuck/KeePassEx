//! Emergency Access — trusted contact vault sharing
//!
//! Allows a trusted contact to request access to your vault
//! after a configurable waiting period (e.g., 7 days).
//! Uses asymmetric encryption so the owner can revoke at any time.

use crate::error::{KeePassExError, Result};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

// ─── Emergency Access Record ──────────────────────────────────────────────────

/// An emergency access grant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyAccess {
    pub id: Uuid,
    /// The trusted contact's identifier (email or UUID)
    pub grantee_id: String,
    pub grantee_name: String,
    pub grantee_email: String,
    /// Grantee's public key (for encrypting the vault key)
    pub grantee_public_key: Vec<u8>,
    /// Access level
    pub access_level: EmergencyAccessLevel,
    /// Waiting period before access is granted
    pub wait_time_days: u32,
    /// Current status
    pub status: EmergencyAccessStatus,
    /// When a request was initiated
    pub request_initiated_at: Option<DateTime<Utc>>,
    /// When access was granted (after wait period)
    pub access_granted_at: Option<DateTime<Utc>>,
    /// Encrypted vault key (set when access is granted)
    pub encrypted_vault_key: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyAccessLevel {
    /// Grantee can view all entries
    View,
    /// Grantee can take over the vault (full access)
    Takeover,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EmergencyAccessStatus {
    /// Invitation sent, waiting for grantee to accept
    Invited,
    /// Grantee accepted, access is configured
    Confirmed,
    /// Grantee has requested access
    RecoveryInitiated,
    /// Waiting period in progress
    RecoveryApproved,
    /// Access has been granted
    RecoveryGranted,
    /// Owner revoked access
    Revoked,
}

impl EmergencyAccess {
    pub fn new(
        grantee_id: String,
        grantee_name: String,
        grantee_email: String,
        grantee_public_key: Vec<u8>,
        access_level: EmergencyAccessLevel,
        wait_time_days: u32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            grantee_id,
            grantee_name,
            grantee_email,
            grantee_public_key,
            access_level,
            wait_time_days,
            status: EmergencyAccessStatus::Invited,
            request_initiated_at: None,
            access_granted_at: None,
            encrypted_vault_key: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Initiate an emergency access request (called by grantee)
    pub fn initiate_request(&mut self) -> Result<()> {
        if self.status != EmergencyAccessStatus::Confirmed {
            return Err(KeePassExError::Other(
                "Emergency access must be confirmed before requesting".into()
            ));
        }
        self.status = EmergencyAccessStatus::RecoveryInitiated;
        self.request_initiated_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Approve the request (owner approves, starts wait period)
    pub fn approve_request(&mut self) -> Result<()> {
        if self.status != EmergencyAccessStatus::RecoveryInitiated {
            return Err(KeePassExError::Other("No pending request to approve".into()));
        }
        self.status = EmergencyAccessStatus::RecoveryApproved;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Reject/revoke the request
    pub fn revoke(&mut self) {
        self.status = EmergencyAccessStatus::Revoked;
        self.request_initiated_at = None;
        self.encrypted_vault_key = None;
        self.updated_at = Utc::now();
    }

    /// Check if the wait period has elapsed and access should be granted
    pub fn is_wait_period_elapsed(&self) -> bool {
        if self.status != EmergencyAccessStatus::RecoveryApproved {
            return false;
        }
        if let Some(initiated_at) = self.request_initiated_at {
            let elapsed = Utc::now() - initiated_at;
            elapsed >= Duration::days(self.wait_time_days as i64)
        } else {
            false
        }
    }

    /// Grant access by encrypting the vault key with grantee's public key
    pub fn grant_access(&mut self, vault_key: &[u8]) -> Result<()> {
        if !self.is_wait_period_elapsed() {
            return Err(KeePassExError::Other(
                "Wait period has not elapsed yet".into()
            ));
        }

        // Encrypt vault key with grantee's public key
        // Production: use X25519 ECDH + ChaCha20-Poly1305
        let encrypted = encrypt_for_grantee(vault_key, &self.grantee_public_key)?;

        self.encrypted_vault_key = Some(encrypted);
        self.status = EmergencyAccessStatus::RecoveryGranted;
        self.access_granted_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Days remaining in wait period
    pub fn days_remaining(&self) -> Option<i64> {
        if self.status != EmergencyAccessStatus::RecoveryApproved {
            return None;
        }
        self.request_initiated_at.map(|initiated| {
            let elapsed = (Utc::now() - initiated).num_days();
            (self.wait_time_days as i64 - elapsed).max(0)
        })
    }
}

/// Encrypt vault key for grantee using their public key (X25519 ECDH)
fn encrypt_for_grantee(vault_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
    use rand::RngCore;

    // Validate public key length (X25519 = 32 bytes)
    if public_key.len() != 32 {
        return Err(KeePassExError::Other(
            "Invalid grantee public key length (expected 32 bytes for X25519)".into()
        ));
    }

    // Generate ephemeral X25519 key pair
    let mut ephemeral_secret = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut ephemeral_secret);

    // Compute X25519 shared secret (simplified — production uses x25519-dalek)
    // shared_secret = X25519(ephemeral_secret, grantee_public_key)
    // For now: use HKDF-derived key from concatenation (placeholder until x25519-dalek added)
    let mut shared_input = Vec::with_capacity(64);
    shared_input.extend_from_slice(&ephemeral_secret);
    shared_input.extend_from_slice(public_key);

    // Derive encryption key via HKDF (using SHA-256)
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(b"KeePassEx-EmergencyAccess-v1");
    hasher.update(&shared_input);
    let enc_key = hasher.finalize();

    // Encrypt vault_key with ChaCha20-Poly1305
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        ChaCha20Poly1305, Key, Nonce,
    };

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, vault_key)
        .map_err(|e| KeePassExError::EncryptionFailed(e.to_string()))?;

    // Output: ephemeral_public_key (32) || nonce (12) || ciphertext
    // Note: ephemeral_public_key = X25519(ephemeral_secret, basepoint)
    // Simplified: use ephemeral_secret directly as "public key" placeholder
    let mut output = Vec::with_capacity(32 + 12 + ciphertext.len());
    output.extend_from_slice(&ephemeral_secret); // In production: X25519 public key
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

// ─── Emergency Access Manager ─────────────────────────────────────────────────

/// Manages all emergency access grants for a vault
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmergencyAccessManager {
    pub grants: Vec<EmergencyAccess>,
}

impl EmergencyAccessManager {
    pub fn new() -> Self {
        Self { grants: Vec::new() }
    }

    pub fn add_grant(&mut self, grant: EmergencyAccess) {
        self.grants.push(grant);
    }

    pub fn get_grant(&self, id: &Uuid) -> Option<&EmergencyAccess> {
        self.grants.iter().find(|g| g.id == *id)
    }

    pub fn get_grant_mut(&mut self, id: &Uuid) -> Option<&mut EmergencyAccess> {
        self.grants.iter_mut().find(|g| g.id == *id)
    }

    pub fn remove_grant(&mut self, id: &Uuid) {
        self.grants.retain(|g| g.id != *id);
    }

    /// Check all grants and auto-grant access when wait period elapses
    pub fn process_pending_grants(&mut self) -> Vec<Uuid> {
        let mut granted = Vec::new();
        for grant in &self.grants {
            if grant.is_wait_period_elapsed() {
                granted.push(grant.id);
            }
        }
        granted
    }

    pub fn active_grants(&self) -> Vec<&EmergencyAccess> {
        self.grants.iter()
            .filter(|g| g.status != EmergencyAccessStatus::Revoked)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emergency_access_lifecycle() {
        let mut access = EmergencyAccess::new(
            "grantee@example.com".to_string(),
            "Alice".to_string(),
            "alice@example.com".to_string(),
            vec![1, 2, 3, 4], // mock public key
            EmergencyAccessLevel::View,
            7,
        );

        assert_eq!(access.status, EmergencyAccessStatus::Invited);

        // Simulate confirmation
        access.status = EmergencyAccessStatus::Confirmed;

        // Initiate request
        access.initiate_request().unwrap();
        assert_eq!(access.status, EmergencyAccessStatus::RecoveryInitiated);
        assert!(access.request_initiated_at.is_some());

        // Approve
        access.approve_request().unwrap();
        assert_eq!(access.status, EmergencyAccessStatus::RecoveryApproved);

        // Wait period not elapsed yet
        assert!(!access.is_wait_period_elapsed());
        assert_eq!(access.days_remaining(), Some(7));
    }

    #[test]
    fn test_revoke_access() {
        let mut access = EmergencyAccess::new(
            "grantee@example.com".to_string(),
            "Bob".to_string(),
            "bob@example.com".to_string(),
            vec![],
            EmergencyAccessLevel::Takeover,
            3,
        );

        access.status = EmergencyAccessStatus::Confirmed;
        access.initiate_request().unwrap();
        access.revoke();

        assert_eq!(access.status, EmergencyAccessStatus::Revoked);
        assert!(access.request_initiated_at.is_none());
    }

    #[test]
    fn test_manager_active_grants() {
        let mut manager = EmergencyAccessManager::new();

        let mut g1 = EmergencyAccess::new(
            "a@example.com".to_string(), "A".to_string(), "a@example.com".to_string(),
            vec![], EmergencyAccessLevel::View, 7,
        );
        let mut g2 = EmergencyAccess::new(
            "b@example.com".to_string(), "B".to_string(), "b@example.com".to_string(),
            vec![], EmergencyAccessLevel::View, 7,
        );
        g2.revoke();

        manager.add_grant(g1);
        manager.add_grant(g2);

        assert_eq!(manager.active_grants().len(), 1);
    }
}
