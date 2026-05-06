//! Zero-Knowledge Password Verification (ZKPV)
//!
//! Allows verifying a master password is correct WITHOUT decrypting the vault.
//! Uses a commitment scheme: H(password || salt) stored in vault header.
//! On unlock attempt, recompute and compare — no plaintext password ever stored.
//!
//! # Why this matters
//! - Prevents timing attacks on password comparison
//! - Enables fast "wrong password" detection before expensive Argon2id KDF
//! - Audit log can record failed attempts without storing the attempted password
//!
//! # Protocol
//! 1. Setup: commitment = Argon2id(password, salt, low_cost_params)
//! 2. Verify: recompute commitment, compare in constant time
//! 3. If match → proceed with full Argon2id KDF for vault key derivation
//!
//! This is NOT the vault encryption key — it's a fast pre-check only.

use crate::error::{KeePassExError, Result};
use argon2::{Argon2, Params, Version};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// ZKPV commitment stored in vault header
#[derive(Debug, Clone)]
pub struct ZkpvCommitment {
    /// Random salt (32 bytes)
    pub salt: [u8; 32],
    /// Commitment hash (32 bytes)
    pub commitment: [u8; 32],
    /// Argon2id parameters (low cost for fast pre-check)
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl ZkpvCommitment {
    /// Default low-cost params for fast pre-check (NOT for key derivation)
    const DEFAULT_M_COST: u32 = 4096; // 4 MB (vs 64 MB for key derivation)
    const DEFAULT_T_COST: u32 = 1;
    const DEFAULT_P_COST: u32 = 1;

    /// Create a new commitment from a password
    pub fn create(password: &str) -> Result<Self> {
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);

        let commitment = Self::compute(
            password,
            &salt,
            Self::DEFAULT_M_COST,
            Self::DEFAULT_T_COST,
            Self::DEFAULT_P_COST,
        )?;

        Ok(ZkpvCommitment {
            salt,
            commitment,
            m_cost: Self::DEFAULT_M_COST,
            t_cost: Self::DEFAULT_T_COST,
            p_cost: Self::DEFAULT_P_COST,
        })
    }

    /// Verify a password against this commitment (constant-time)
    pub fn verify(&self, password: &str) -> Result<bool> {
        let candidate = Self::compute(password, &self.salt, self.m_cost, self.t_cost, self.p_cost)?;
        Ok(constant_time_eq(&candidate, &self.commitment))
    }

    /// Serialize to bytes for KDBX header storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(32 + 32 + 12);
        out.extend_from_slice(&self.salt);
        out.extend_from_slice(&self.commitment);
        out.extend_from_slice(&self.m_cost.to_le_bytes());
        out.extend_from_slice(&self.t_cost.to_le_bytes());
        out.extend_from_slice(&self.p_cost.to_le_bytes());
        out
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 76 {
            return Err(KeePassExError::Other("ZKPV commitment too short".into()));
        }
        let mut salt = [0u8; 32];
        let mut commitment = [0u8; 32];
        salt.copy_from_slice(&bytes[0..32]);
        commitment.copy_from_slice(&bytes[32..64]);
        let m_cost = u32::from_le_bytes([bytes[64], bytes[65], bytes[66], bytes[67]]);
        let t_cost = u32::from_le_bytes([bytes[68], bytes[69], bytes[70], bytes[71]]);
        let p_cost = u32::from_le_bytes([bytes[72], bytes[73], bytes[74], bytes[75]]);
        Ok(ZkpvCommitment {
            salt,
            commitment,
            m_cost,
            t_cost,
            p_cost,
        })
    }

    fn compute(
        password: &str,
        salt: &[u8; 32],
        m_cost: u32,
        t_cost: u32,
        p_cost: u32,
    ) -> Result<[u8; 32]> {
        let params = Params::new(m_cost, t_cost, p_cost, Some(32))
            .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;
        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let mut output = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut output)
            .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;
        Ok(output)
    }
}

/// Constant-time byte comparison (prevents timing attacks)
fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// Fast password hint system — stores a hint without revealing the password
#[derive(Debug, Clone)]
pub struct PasswordHint {
    /// Encrypted hint (XOR with SHA256(password) — only readable if you know the password)
    pub encrypted_hint: Vec<u8>,
    /// Nonce for hint encryption
    pub nonce: [u8; 32],
}

impl PasswordHint {
    /// Store a hint, encrypted so only someone who knows the password can read it
    pub fn create(hint: &str, password: &str) -> Self {
        let mut nonce = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut nonce);

        let key = Self::derive_hint_key(password, &nonce);
        let encrypted: Vec<u8> = hint
            .as_bytes()
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();

        PasswordHint {
            encrypted_hint: encrypted,
            nonce,
        }
    }

    /// Decrypt the hint using the password
    pub fn decrypt(&self, password: &str) -> Option<String> {
        let key = Self::derive_hint_key(password, &self.nonce);
        let decrypted: Vec<u8> = self
            .encrypted_hint
            .iter()
            .zip(key.iter().cycle())
            .map(|(b, k)| b ^ k)
            .collect();
        String::from_utf8(decrypted).ok()
    }

    fn derive_hint_key(password: &str, nonce: &[u8; 32]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(b"kpx-hint-v1:");
        hasher.update(password.as_bytes());
        hasher.update(nonce);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_create_and_verify() {
        let commitment = ZkpvCommitment::create("MyMasterPassword123!").unwrap();
        assert!(commitment.verify("MyMasterPassword123!").unwrap());
    }

    #[test]
    fn test_commitment_wrong_password_fails() {
        let commitment = ZkpvCommitment::create("CorrectPassword").unwrap();
        assert!(!commitment.verify("WrongPassword").unwrap());
    }

    #[test]
    fn test_commitment_serialization_roundtrip() {
        let original = ZkpvCommitment::create("TestPassword").unwrap();
        let bytes = original.to_bytes();
        let restored = ZkpvCommitment::from_bytes(&bytes).unwrap();

        assert_eq!(original.salt, restored.salt);
        assert_eq!(original.commitment, restored.commitment);
        assert_eq!(original.m_cost, restored.m_cost);
    }

    #[test]
    fn test_commitment_different_salts() {
        let c1 = ZkpvCommitment::create("SamePassword").unwrap();
        let c2 = ZkpvCommitment::create("SamePassword").unwrap();
        // Different salts → different commitments
        assert_ne!(c1.salt, c2.salt);
        assert_ne!(c1.commitment, c2.commitment);
        // But both verify correctly
        assert!(c1.verify("SamePassword").unwrap());
        assert!(c2.verify("SamePassword").unwrap());
    }

    #[test]
    fn test_constant_time_eq_same() {
        let a = [0x42u8; 32];
        let b = [0x42u8; 32];
        assert!(constant_time_eq(&a, &b));
    }

    #[test]
    fn test_constant_time_eq_different() {
        let a = [0x42u8; 32];
        let mut b = [0x42u8; 32];
        b[15] = 0x43;
        assert!(!constant_time_eq(&a, &b));
    }

    #[test]
    fn test_password_hint_encrypt_decrypt() {
        let hint = "First pet's name";
        let password = "MySecretPassword";
        let stored = PasswordHint::create(hint, password);
        let decrypted = stored.decrypt(password).unwrap();
        assert_eq!(decrypted, hint);
    }

    #[test]
    fn test_password_hint_wrong_password_garbles() {
        let hint = "My hint";
        let stored = PasswordHint::create(hint, "correct");
        // Wrong password produces garbage (not the original hint)
        let wrong = stored.decrypt("wrong");
        assert!(wrong.is_none() || wrong.unwrap() != hint);
    }

    #[test]
    fn test_commitment_bytes_length() {
        let c = ZkpvCommitment::create("test").unwrap();
        assert_eq!(c.to_bytes().len(), 76); // 32 + 32 + 4 + 4 + 4
    }
}
