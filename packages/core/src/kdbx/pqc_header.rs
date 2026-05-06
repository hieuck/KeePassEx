//! KDBX PQC Header Extension
//!
//! Stores quantum-resistant key material in KDBX custom header fields.
//! Unknown header fields are silently ignored by KeePass/KeePassXC,
//! so PQC vaults remain openable by legacy clients (they just won't
//! benefit from quantum resistance).
//!
//! # Header field IDs (custom range 0x80–0xFF)
//! - 0x80 = PQC_ALGORITHM   — u8: 0=Classical, 1=HybridKyber768
//! - 0x81 = PQC_PUBLIC_KEY  — bytes: recipient public key
//! - 0x82 = PQC_ENCAPSULATION — bytes: encapsulated shared secret
//!
//! # Vault key derivation with PQC enabled
//! 1. Argon2id KDF → composite_key (as normal)
//! 2. PQC decapsulate(private_key, encapsulation) → pqc_shared
//! 3. Final vault key = HKDF(composite_key || pqc_shared, "kpx-final-key")
//!
//! This means even if Argon2id is broken, the PQC layer still protects.

use crate::crypto::pqc::{
    decapsulate, derive_pqc_keypair, encapsulate, PqcAlgorithm, PqcEncapsulation,
};
use crate::error::{KeePassExError, Result};
use zeroize::Zeroize;

/// Custom KDBX header field IDs for PQC
pub const PQC_FIELD_ALGORITHM: u8 = 0x80;
pub const PQC_FIELD_PUBLIC_KEY: u8 = 0x81;
pub const PQC_FIELD_ENCAPSULATION: u8 = 0x82;

/// PQC configuration stored in a vault's header
#[derive(Debug, Clone)]
pub struct PqcVaultConfig {
    pub algorithm: PqcAlgorithm,
    /// Recipient public key (stored in header, used to encapsulate on save)
    pub public_key: Vec<u8>,
    /// Encapsulated shared secret (stored in header, used to decapsulate on open)
    pub encapsulation: PqcEncapsulation,
}

impl Drop for PqcVaultConfig {
    fn drop(&mut self) {
        self.public_key.zeroize();
    }
}

/// Derive the final vault key by combining Argon2id output with PQC shared secret.
///
/// # Arguments
/// * `argon2_key` — 32-byte key from Argon2id KDF
/// * `pqc_shared` — 32-byte shared secret from PQC decapsulation
///
/// # Returns
/// 32-byte final vault key
pub fn derive_final_key(argon2_key: &[u8; 32], pqc_shared: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(argon2_key);
    hasher.update(pqc_shared);
    hasher.update(b"kpx-final-key-v1");
    hasher.finalize().into()
}

/// Generate PQC config for a new vault or when enabling PQC on an existing vault.
///
/// # Arguments
/// * `argon2_key` — 32-byte key from Argon2id (used to deterministically derive PQC keypair)
/// * `algorithm` — PQC algorithm to use
pub fn generate_pqc_config(
    argon2_key: &[u8; 32],
    algorithm: PqcAlgorithm,
) -> Result<(PqcVaultConfig, Vec<u8>)> {
    // Derive PQC keypair deterministically from the Argon2id key
    let keypair = derive_pqc_keypair(argon2_key, algorithm);

    // Encapsulate: generates a random shared secret + ciphertext
    let (shared_secret, encapsulation) = encapsulate(&keypair.public_key, algorithm)?;

    let config = PqcVaultConfig {
        algorithm,
        public_key: keypair.public_key.clone(),
        encapsulation,
    };

    Ok((config, shared_secret))
}

/// Recover the PQC shared secret from a vault's header config.
///
/// # Arguments
/// * `argon2_key` — 32-byte key from Argon2id (used to re-derive PQC private key)
/// * `config` — PQC config read from vault header
pub fn recover_pqc_shared(argon2_key: &[u8; 32], config: &PqcVaultConfig) -> Result<Vec<u8>> {
    let keypair = derive_pqc_keypair(argon2_key, config.algorithm);
    decapsulate(&keypair.private_key, &config.encapsulation)
}

/// Serialize PQC config to custom header fields.
/// Returns a list of (field_id, field_data) pairs to embed in the KDBX header.
pub fn serialize_pqc_header(config: &PqcVaultConfig) -> Vec<(u8, Vec<u8>)> {
    vec![
        (PQC_FIELD_ALGORITHM, vec![config.algorithm as u8]),
        (PQC_FIELD_PUBLIC_KEY, config.public_key.clone()),
        (PQC_FIELD_ENCAPSULATION, config.encapsulation.to_bytes()),
    ]
}

/// Parse PQC config from custom header fields.
/// Returns `None` if no PQC fields are present (classical vault).
pub fn parse_pqc_header(custom_fields: &[(u8, Vec<u8>)]) -> Result<Option<PqcVaultConfig>> {
    let algorithm_field = custom_fields
        .iter()
        .find(|(id, _)| *id == PQC_FIELD_ALGORITHM);

    let algorithm = match algorithm_field {
        None => return Ok(None), // No PQC — classical vault
        Some((_, data)) => {
            if data.is_empty() {
                return Ok(None);
            }
            match data[0] {
                0 => PqcAlgorithm::Classical,
                1 => PqcAlgorithm::HybridKyber768,
                v => {
                    return Err(KeePassExError::Other(format!(
                        "Unknown PQC algorithm: {}",
                        v
                    )))
                }
            }
        }
    };

    // Classical mode — no PQC key material needed
    if algorithm == PqcAlgorithm::Classical {
        return Ok(None);
    }

    let public_key = custom_fields
        .iter()
        .find(|(id, _)| *id == PQC_FIELD_PUBLIC_KEY)
        .map(|(_, data)| data.clone())
        .ok_or_else(|| KeePassExError::Other("PQC public key missing from header".into()))?;

    let encap_bytes = custom_fields
        .iter()
        .find(|(id, _)| *id == PQC_FIELD_ENCAPSULATION)
        .map(|(_, data)| data.clone())
        .ok_or_else(|| KeePassExError::Other("PQC encapsulation missing from header".into()))?;

    let encapsulation = PqcEncapsulation::from_bytes(&encap_bytes)?;

    Ok(Some(PqcVaultConfig {
        algorithm,
        public_key,
        encapsulation,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_config_roundtrip() {
        let argon2_key = [0x42u8; 32];
        let (config, shared1) =
            generate_pqc_config(&argon2_key, PqcAlgorithm::HybridKyber768).unwrap();

        // Serialize and parse back
        let fields = serialize_pqc_header(&config);
        let parsed = parse_pqc_header(&fields).unwrap().unwrap();

        assert_eq!(parsed.algorithm, config.algorithm);
        assert_eq!(parsed.public_key, config.public_key);
    }

    #[test]
    fn test_no_pqc_fields_returns_none() {
        let fields: Vec<(u8, Vec<u8>)> = vec![];
        let result = parse_pqc_header(&fields).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_derive_final_key_deterministic() {
        let argon2_key = [0x11u8; 32];
        let pqc_shared = vec![0x22u8; 32];
        let key1 = derive_final_key(&argon2_key, &pqc_shared);
        let key2 = derive_final_key(&argon2_key, &pqc_shared);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_final_key_differs_from_argon2_alone() {
        let argon2_key = [0x11u8; 32];
        let pqc_shared = vec![0x22u8; 32];
        let final_key = derive_final_key(&argon2_key, &pqc_shared);
        // Final key must differ from raw Argon2id key
        assert_ne!(final_key.as_slice(), argon2_key.as_slice());
    }
}
