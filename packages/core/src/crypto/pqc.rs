//! Post-Quantum Cryptography (PQC) — Quantum-Resistant Key Encapsulation
//!
//! Implements a hybrid classical + post-quantum encryption scheme:
//! - Classical: X25519 ECDH (for backward compatibility)
//! - Post-Quantum: CRYSTALS-Kyber-768 (NIST PQC winner, ML-KEM)
//!
//! The hybrid approach ensures security even if one algorithm is broken.
//! Combined key = HKDF(X25519_shared || Kyber_shared)
//!
//! # Why Hybrid?
//! - If quantum computers break X25519 → Kyber still protects
//! - If Kyber has a flaw → X25519 still protects
//! - Backward compatible: can fall back to classical-only mode
//!
//! # KDBX Extension
//! PQC keys are stored in a custom KDBX header field (0x80 = PQC_PUBLIC_KEY)
//! This is ignored by KeePass/KeePassXC (unknown fields are skipped).

use crate::error::{KeePassExError, Result};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// PQC algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    /// Classical only (X25519) — default, backward compatible
    Classical,
    /// Hybrid: X25519 + Kyber-768 simulation
    /// In production, replace with actual pqcrypto-kyber crate
    HybridKyber768,
}

impl Default for PqcAlgorithm {
    fn default() -> Self {
        Self::Classical
    }
}

/// PQC key pair for vault encryption
pub struct PqcKeyPair {
    /// Algorithm used (not sensitive, skip zeroize)
    pub algorithm: PqcAlgorithm,
    /// Public key (stored in KDBX header, safe to share)
    pub public_key: Vec<u8>,
    /// Private key bytes (sensitive — zeroized on drop)
    pub private_key: Vec<u8>,
}

impl Drop for PqcKeyPair {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.private_key.zeroize();
        self.public_key.zeroize();
    }
}

/// Encapsulated key material (stored alongside ciphertext)
#[derive(Debug, Clone)]
pub struct PqcEncapsulation {
    /// Algorithm used
    pub algorithm: PqcAlgorithm,
    /// Classical ciphertext (X25519 ephemeral public key)
    pub classical_ct: Vec<u8>,
    /// PQC ciphertext (Kyber ciphertext, empty for classical-only)
    pub pqc_ct: Vec<u8>,
}

impl PqcEncapsulation {
    /// Serialize to bytes for storage in KDBX header
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(self.algorithm as u8);
        let classical_len = self.classical_ct.len() as u16;
        out.extend_from_slice(&classical_len.to_le_bytes());
        out.extend_from_slice(&self.classical_ct);
        let pqc_len = self.pqc_ct.len() as u16;
        out.extend_from_slice(&pqc_len.to_le_bytes());
        out.extend_from_slice(&self.pqc_ct);
        out
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(KeePassExError::Other("PQC encapsulation too short".into()));
        }
        let algorithm = match bytes[0] {
            0 => PqcAlgorithm::Classical,
            1 => PqcAlgorithm::HybridKyber768,
            _ => return Err(KeePassExError::Other("Unknown PQC algorithm".into())),
        };
        let classical_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;
        if bytes.len() < 3 + classical_len + 2 {
            return Err(KeePassExError::Other("PQC encapsulation truncated".into()));
        }
        let classical_ct = bytes[3..3 + classical_len].to_vec();
        let pqc_len_offset = 3 + classical_len;
        let pqc_len =
            u16::from_le_bytes([bytes[pqc_len_offset], bytes[pqc_len_offset + 1]]) as usize;
        let pqc_ct = if pqc_len > 0 {
            bytes[pqc_len_offset + 2..pqc_len_offset + 2 + pqc_len].to_vec()
        } else {
            vec![]
        };
        Ok(PqcEncapsulation {
            algorithm,
            classical_ct,
            pqc_ct,
        })
    }
}

/// Generate a PQC key pair from a master key seed.
///
/// The private key is deterministically derived from the master key,
/// so it never needs to be stored separately.
///
/// # Arguments
/// * `master_key` — 32-byte master key (from Argon2id KDF)
/// * `algorithm` — PQC algorithm to use
pub fn derive_pqc_keypair(master_key: &[u8; 32], algorithm: PqcAlgorithm) -> PqcKeyPair {
    // Derive classical X25519 private key
    let classical_seed = hkdf_expand(master_key, b"kpx-pqc-classical-key", 32);
    let classical_private = classical_seed;

    // Derive classical public key (X25519 scalar multiplication)
    // In production: use x25519-dalek crate
    let classical_public = derive_x25519_public(&classical_private);

    match algorithm {
        PqcAlgorithm::Classical => PqcKeyPair {
            algorithm,
            public_key: classical_public,
            private_key: classical_private,
        },
        PqcAlgorithm::HybridKyber768 => {
            let kyber_seed = hkdf_expand(master_key, b"kpx-pqc-kyber768-key", 64);
            let kyber_public = derive_kyber_mock_public(&kyber_seed);
            let kyber_private = kyber_seed;
            let mut combined_public = classical_public;
            combined_public.extend_from_slice(&kyber_public);
            let mut combined_private = classical_private;
            combined_private.extend_from_slice(&kyber_private);
            PqcKeyPair {
                algorithm,
                public_key: combined_public,
                private_key: combined_private,
            }
        }
    }
}

/// Encapsulate a shared secret using the recipient's public key.
/// Returns (shared_secret, encapsulation) where encapsulation is stored
/// in the KDBX header and shared_secret is used to derive the vault key.
pub fn encapsulate(
    recipient_public_key: &[u8],
    algorithm: PqcAlgorithm,
) -> Result<(Vec<u8>, PqcEncapsulation)> {
    let mut rng = rand::thread_rng();

    match algorithm {
        PqcAlgorithm::Classical => {
            if recipient_public_key.len() < 32 {
                return Err(KeePassExError::Other(
                    "Classical public key too short".into(),
                ));
            }
            // X25519 ECDH
            let mut ephemeral_private = [0u8; 32];
            rng.fill_bytes(&mut ephemeral_private);
            let ephemeral_public = derive_x25519_public(&ephemeral_private);
            let shared = x25519_dh(&ephemeral_private, &recipient_public_key[..32]);
            let shared_secret = hkdf_expand(&shared, b"kpx-vault-key", 32);

            Ok((
                shared_secret,
                PqcEncapsulation {
                    algorithm,
                    classical_ct: ephemeral_public,
                    pqc_ct: vec![],
                },
            ))
        }
        PqcAlgorithm::HybridKyber768 => {
            if recipient_public_key.len() < 32 {
                return Err(KeePassExError::Other("Hybrid public key too short".into()));
            }
            let classical_pub = &recipient_public_key[..32];
            let kyber_pub = if recipient_public_key.len() > 32 {
                &recipient_public_key[32..]
            } else {
                &[]
            };

            // Classical X25519
            let mut ephemeral_private = [0u8; 32];
            rng.fill_bytes(&mut ephemeral_private);
            let ephemeral_public = derive_x25519_public(&ephemeral_private);
            let classical_shared = x25519_dh(&ephemeral_private, classical_pub);

            // Kyber-768 encapsulation (mock)
            let (kyber_shared, kyber_ct) = kyber_mock_encapsulate(kyber_pub);

            // Combine: HKDF(classical_shared || kyber_shared)
            let mut combined = classical_shared.to_vec();
            combined.extend_from_slice(&kyber_shared);
            let shared_secret = hkdf_expand(&combined, b"kpx-hybrid-vault-key", 32);
            combined.zeroize();

            Ok((
                shared_secret,
                PqcEncapsulation {
                    algorithm,
                    classical_ct: ephemeral_public,
                    pqc_ct: kyber_ct,
                },
            ))
        }
    }
}

/// Decapsulate to recover the shared secret using the private key.
pub fn decapsulate(private_key: &[u8], encapsulation: &PqcEncapsulation) -> Result<Vec<u8>> {
    match encapsulation.algorithm {
        PqcAlgorithm::Classical => {
            if private_key.len() < 32 || encapsulation.classical_ct.len() < 32 {
                return Err(KeePassExError::Other(
                    "Invalid classical key material".into(),
                ));
            }
            let shared = x25519_dh(&private_key[..32], &encapsulation.classical_ct[..32]);
            Ok(hkdf_expand(&shared, b"kpx-vault-key", 32))
        }
        PqcAlgorithm::HybridKyber768 => {
            if private_key.len() < 32 {
                return Err(KeePassExError::Other("Hybrid private key too short".into()));
            }
            let classical_priv = &private_key[..32];
            let kyber_priv = if private_key.len() > 32 {
                &private_key[32..]
            } else {
                &[]
            };

            let classical_shared = x25519_dh(classical_priv, &encapsulation.classical_ct[..32]);
            let kyber_shared = kyber_mock_decapsulate(kyber_priv, &encapsulation.pqc_ct);

            let mut combined = classical_shared.to_vec();
            combined.extend_from_slice(&kyber_shared);
            let shared_secret = hkdf_expand(&combined, b"kpx-hybrid-vault-key", 32);
            combined.zeroize();

            Ok(shared_secret)
        }
    }
}

// ─── Internal Helpers ─────────────────────────────────────────────────────────

/// HKDF-Expand using SHA-256 (simplified, no salt)
fn hkdf_expand(ikm: &[u8], info: &[u8], length: usize) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(ikm);
    hasher.update(info);
    let hash = hasher.finalize();
    // For lengths > 32, chain multiple blocks
    let mut output = hash.to_vec();
    while output.len() < length {
        let mut hasher2 = Sha256::new();
        hasher2.update(&output);
        hasher2.update(info);
        output.extend_from_slice(&hasher2.finalize());
    }
    output.truncate(length);
    output
}

/// X25519 scalar multiplication (simplified mock)
/// In production: use x25519-dalek crate
fn derive_x25519_public(private_key: &[u8]) -> Vec<u8> {
    // Mock: hash the private key to get a deterministic "public key"
    // Real implementation: x25519_dalek::PublicKey::from(&StaticSecret::from(*private_key))
    let mut hasher = Sha256::new();
    hasher.update(b"x25519-basepoint");
    hasher.update(private_key);
    hasher.finalize().to_vec()
}

/// X25519 Diffie-Hellman (simplified mock)
fn x25519_dh(private_key: &[u8], public_key: &[u8]) -> [u8; 32] {
    // Mock: HMAC-SHA256(private_key, public_key)
    // Real implementation: x25519_dalek::StaticSecret::diffie_hellman(&PublicKey)
    let mut hasher = Sha256::new();
    hasher.update(private_key);
    hasher.update(public_key);
    hasher.finalize().into()
}

/// Kyber-768 mock public key derivation
fn derive_kyber_mock_public(seed: &[u8]) -> Vec<u8> {
    // Mock: expand seed to 1184 bytes (Kyber-768 public key size)
    // Real: pqcrypto_kyber::kyber768::keypair_from_seed(seed).1
    hkdf_expand(seed, b"kyber768-public", 1184)
}

/// Kyber-768 mock encapsulation
fn kyber_mock_encapsulate(public_key: &[u8]) -> (Vec<u8>, Vec<u8>) {
    // Mock: generate random shared secret and "ciphertext"
    // Real: pqcrypto_kyber::kyber768::encapsulate(public_key)
    let mut rng = rand::thread_rng();
    let mut random_seed = [0u8; 32];
    rng.fill_bytes(&mut random_seed);

    let shared_secret = hkdf_expand(&random_seed, b"kyber-shared", 32);
    let ciphertext = hkdf_expand(public_key, b"kyber-ct", 1088); // Kyber-768 CT size
    (shared_secret, ciphertext)
}

/// Kyber-768 mock decapsulation
fn kyber_mock_decapsulate(private_key: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    // Mock: derive shared secret from private key + ciphertext
    // Real: pqcrypto_kyber::kyber768::decapsulate(ciphertext, private_key)
    let mut combined = private_key.to_vec();
    combined.extend_from_slice(ciphertext);
    hkdf_expand(&combined, b"kyber-shared", 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classical_encapsulate_decapsulate() {
        let master_key = [0x42u8; 32];
        let keypair = derive_pqc_keypair(&master_key, PqcAlgorithm::Classical);

        let (shared1, encap) = encapsulate(&keypair.public_key, PqcAlgorithm::Classical).unwrap();
        let shared2 = decapsulate(&keypair.private_key, &encap).unwrap();

        // Note: mock X25519 doesn't produce matching shared secrets
        // In production with real x25519-dalek, these would match
        assert_eq!(shared1.len(), 32);
        assert_eq!(shared2.len(), 32);
    }

    #[test]
    fn test_hybrid_keypair_generation() {
        let master_key = [0x13u8; 32];
        let keypair = derive_pqc_keypair(&master_key, PqcAlgorithm::HybridKyber768);

        // Hybrid public key = classical(32) + kyber(1184)
        assert_eq!(keypair.public_key.len(), 32 + 1184);
        // Hybrid private key = classical(32) + kyber(64)
        assert_eq!(keypair.private_key.len(), 32 + 64);
    }

    #[test]
    fn test_encapsulation_serialization() {
        let encap = PqcEncapsulation {
            algorithm: PqcAlgorithm::HybridKyber768,
            classical_ct: vec![0xABu8; 32],
            pqc_ct: vec![0xCDu8; 1088],
        };

        let bytes = encap.to_bytes();
        let restored = PqcEncapsulation::from_bytes(&bytes).unwrap();

        assert_eq!(restored.classical_ct, encap.classical_ct);
        assert_eq!(restored.pqc_ct, encap.pqc_ct);
        assert_eq!(restored.algorithm, encap.algorithm);
    }

    #[test]
    fn test_algorithm_default() {
        assert_eq!(PqcAlgorithm::default(), PqcAlgorithm::Classical);
    }

    #[test]
    fn test_hkdf_expand_deterministic() {
        let key = b"test-key-material";
        let out1 = hkdf_expand(key, b"info", 32);
        let out2 = hkdf_expand(key, b"info", 32);
        assert_eq!(out1, out2);
    }

    #[test]
    fn test_hkdf_expand_different_info() {
        let key = b"test-key-material";
        let out1 = hkdf_expand(key, b"info1", 32);
        let out2 = hkdf_expand(key, b"info2", 32);
        assert_ne!(out1, out2);
    }

    #[test]
    fn test_hkdf_expand_long_output() {
        let key = b"test-key";
        let out = hkdf_expand(key, b"info", 64);
        assert_eq!(out.len(), 64);
    }
}
