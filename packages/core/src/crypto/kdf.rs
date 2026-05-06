//! Key Derivation Functions

use crate::error::{KeePassExError, Result};
use argon2::{Argon2, Algorithm, Version, Params};
use sha2::{Sha256, Digest};
use zeroize::Zeroize;

/// KDF variant
#[derive(Debug, Clone)]
pub enum KdfParams {
    Argon2(ArgonParams),
    AesKdf(AesKdfParams),
}

/// Argon2id parameters (KDBX 4.x default)
#[derive(Debug, Clone)]
pub struct ArgonParams {
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

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut output = vec![0u8; 32];
        argon2
            .hash_password_into(composite_key, &self.params.salt, &mut output)
            .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;

        Ok(output)
    }
}

/// AES-KDF implementation (for KDBX 3.1 compatibility)
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

        // Split into two 16-byte blocks and encrypt `rounds` times
        let mut block_a = aes::Block::clone_from_slice(&key[..16]);
        let mut block_b = aes::Block::clone_from_slice(&key[16..]);

        for _ in 0..self.params.rounds {
            cipher.encrypt_block(&mut block_a);
            cipher.encrypt_block(&mut block_b);
        }

        key[..16].copy_from_slice(&block_a);
        key[16..].copy_from_slice(&block_b);

        // Final SHA-256
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
