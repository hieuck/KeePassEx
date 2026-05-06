//! SSH key management and agent protocol

use crate::error::{KeePassExError, Result};
use crate::types::{SshKeyEntry, SshKeyType};

/// Parse SSH public key to extract fingerprint and type
pub fn parse_ssh_public_key(public_key: &str) -> Result<SshKeyInfo> {
    let parts: Vec<&str> = public_key.trim().splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(KeePassExError::SshKeyError("Invalid public key format".into()));
    }

    let key_type = match parts[0] {
        "ssh-ed25519" => SshKeyType::Ed25519,
        "ssh-rsa" => SshKeyType::Rsa4096,
        "ecdsa-sha2-nistp256" => SshKeyType::EcdsaP256,
        "ecdsa-sha2-nistp384" => SshKeyType::EcdsaP384,
        _ => return Err(KeePassExError::SshKeyError(format!("Unknown key type: {}", parts[0]))),
    };

    let comment = parts.get(2).map(|s| s.to_string()).unwrap_or_default();

    // Compute fingerprint (SHA256 of base64-decoded key data)
    let key_data = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        parts[1],
    )
    .map_err(|e| KeePassExError::SshKeyError(e.to_string()))?;

    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&key_data);
    let hash = hasher.finalize();
    let fingerprint = format!(
        "SHA256:{}",
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD_NO_PAD, hash)
    );

    Ok(SshKeyInfo {
        key_type,
        fingerprint,
        comment,
        bits: estimate_key_bits(&key_data),
    })
}

#[derive(Debug, Clone)]
pub struct SshKeyInfo {
    pub key_type: SshKeyType,
    pub fingerprint: String,
    pub comment: String,
    pub bits: usize,
}

fn estimate_key_bits(key_data: &[u8]) -> usize {
    // Rough estimate based on key data length
    match key_data.len() {
        0..=100 => 256,
        101..=300 => 2048,
        301..=400 => 3072,
        _ => 4096,
    }
}

/// SSH Agent protocol message types
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentMessageType {
    RequestIdentities = 11,
    IdentitiesAnswer = 12,
    SignRequest = 13,
    SignResponse = 14,
    AddIdentity = 17,
    RemoveIdentity = 18,
    RemoveAllIdentities = 19,
    Failure = 5,
    Success = 6,
}

/// SSH Agent — manages loaded keys for signing
pub struct SshAgent {
    loaded_keys: Vec<LoadedKey>,
}

struct LoadedKey {
    entry: SshKeyEntry,
    loaded_at: chrono::DateTime<chrono::Utc>,
}

impl SshAgent {
    pub fn new() -> Self {
        Self { loaded_keys: Vec::new() }
    }

    pub fn add_key(&mut self, entry: SshKeyEntry) {
        // Remove existing key with same fingerprint
        self.loaded_keys.retain(|k| k.entry.fingerprint != entry.fingerprint);
        self.loaded_keys.push(LoadedKey {
            entry,
            loaded_at: chrono::Utc::now(),
        });
    }

    pub fn remove_key(&mut self, fingerprint: &str) {
        self.loaded_keys.retain(|k| k.entry.fingerprint != fingerprint);
    }

    pub fn remove_all(&mut self) {
        self.loaded_keys.clear();
    }

    pub fn list_keys(&self) -> Vec<&SshKeyEntry> {
        self.loaded_keys.iter().map(|k| &k.entry).collect()
    }

    /// Remove expired keys (based on agent_duration)
    pub fn cleanup_expired(&mut self) {
        let now = chrono::Utc::now();
        self.loaded_keys.retain(|k| {
            if let Some(duration) = k.entry.agent_duration {
                let elapsed = (now - k.loaded_at).num_seconds() as u64;
                elapsed < duration
            } else {
                true // No expiry
            }
        });
    }
}

impl Default for SshAgent {
    fn default() -> Self {
        Self::new()
    }
}
