//! Key Derivation Functions

use crate::error::{KeePassExError, Result};
use argon2::{Algorithm, Argon2, Params, Version};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// KDF variant
#[derive(Debug, Clone)]
pub enum KdfParams {
    Argon2(ArgonParams),
    AesKdf(AesKdfParams),
}

/// Argon2 variant
#[derive(Debug, Clone, PartialEq)]
pub enum Argon2Variant {
    /// Argon2d — UUID: ef636ddf-8c29-444b-91f7-a9a403e30a0c
    Argon2d,
    /// Argon2id — UUID: 9e298b19-56db-4773-b23d-fc3ec6f0a1e6
    Argon2id,
}

/// Argon2 parameters (KDBX 4.x default)
#[derive(Debug, Clone)]
pub struct ArgonParams {
    pub variant: Argon2Variant,
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub memory_kb: u32,
    pub parallelism: u32,
    pub version: u32,
    pub secret_key: Option<Vec<u8>>,
    pub associated_data: Option<Vec<u8>>,
}

impl Default for ArgonParams {
    fn default() -> Self {
        Self {
            variant: Argon2Variant::Argon2id,
            salt: vec![0u8; 32],
            iterations: 2,
            memory_kb: 65536, // 64 MB
            parallelism: 2,
            version: 19,
            secret_key: None,
            associated_data: None,
        }
    }
}

/// AES-KDF parameters (KDBX 3.1 compat)
#[derive(Debug, Clone)]
pub struct AesKdfParams {
    pub seed: Vec<u8>,
    pub rounds: u64,
}

/// Key Derivation Function trait
pub trait Kdf {
    fn derive(&self, composite_key: &[u8]) -> Result<Vec<u8>>;
}

/// Argon2id KDF implementation
pub struct Argon2Kdf {
    pub params: ArgonParams,
}

impl Kdf for Argon2Kdf {
    fn derive(&self, composite_key: &[u8]) -> Result<Vec<u8>> {
        let params = Params::new(
            self.params.memory_kb,
            self.params.iterations,
            self.params.parallelism,
            Some(32),
        )
        .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;

        let algorithm = match self.params.variant {
            Argon2Variant::Argon2d => Algorithm::Argon2d,
            Argon2Variant::Argon2id => Algorithm::Argon2id,
        };

        let version = match self.params.version {
            0x10 => Version::V0x10,
            _ => Version::V0x13,
        };

        let argon2 = Argon2::new(algorithm, version, params);

        let mut output = vec![0u8; 32];
        argon2
            .hash_password_into(composite_key, &self.params.salt, &mut output)
            .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;

        Ok(output)
    }
}

/// AES-KDF implementation (for KDBX 3.1 compatibility)
/// KeePass uses AES-CBC(key=seed, iv=data_half) to encrypt zeros,
/// then takes the last 16 bytes as the new data half.
pub struct AesKdf {
    pub params: AesKdfParams,
}

impl Kdf for AesKdf {
    fn derive(&self, composite_key: &[u8]) -> Result<Vec<u8>> {
        use aes::cipher::{BlockEncrypt, KeyInit};
        use aes::Aes256;

        let cipher = Aes256::new_from_slice(&self.params.seed)
            .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;

        let mut key = [0u8; 32];
        key.copy_from_slice(&composite_key[..32]);

        // KeePass AES-KDF: encrypt each 16-byte half independently
        // using AES-ECB (equivalent to AES-CBC with zero IV on each block)
        let mut block_a = aes::Block::clone_from_slice(&key[..16]);
        let mut block_b = aes::Block::clone_from_slice(&key[16..]);

        for _ in 0..self.params.rounds {
            cipher.encrypt_block(&mut block_a);
            cipher.encrypt_block(&mut block_b);
        }

        key[..16].copy_from_slice(&block_a);
        key[16..].copy_from_slice(&block_b);

        // Final SHA-256 of the AES-KDF output (per KeePass spec)
        let mut hasher = Sha256::new();
        hasher.update(&key);
        let result = hasher.finalize().to_vec();

        key.zeroize();
        Ok(result)
    }
}

/// Derive master key from composite key using given KDF params
pub fn derive_master_key(composite_key: &[u8], params: &KdfParams) -> Result<Vec<u8>> {
    match params {
        KdfParams::Argon2(p) => {
            let kdf = Argon2Kdf { params: p.clone() };
            kdf.derive(composite_key)
        }
        KdfParams::AesKdf(p) => {
            let kdf = AesKdf { params: p.clone() };
            kdf.derive(composite_key)
        }
    }
}
