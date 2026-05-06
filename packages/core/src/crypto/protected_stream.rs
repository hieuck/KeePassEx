//! Inner protected stream for in-memory field encryption
//! Supports ChaCha20 (KDBX 4.x) and Salsa20 (KDBX 3.1 compat)

use crate::error::{KeePassExError, Result};
use sha2::{Sha256, Digest};

#[derive(Debug, Clone, PartialEq)]
pub enum ProtectedStreamAlgorithm {
    ChaCha20,
    Salsa20,
    ArcFourVariant, // Legacy
}

impl ProtectedStreamAlgorithm {
    pub fn id(&self) -> u32 {
        match self {
            ProtectedStreamAlgorithm::ArcFourVariant => 1,
            ProtectedStreamAlgorithm::Salsa20 => 2,
            ProtectedStreamAlgorithm::ChaCha20 => 3,
        }
    }

    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            1 => Some(ProtectedStreamAlgorithm::ArcFourVariant),
            2 => Some(ProtectedStreamAlgorithm::Salsa20),
            3 => Some(ProtectedStreamAlgorithm::ChaCha20),
            _ => None,
        }
    }
}

/// Protected stream for XOR-encrypting sensitive fields in memory
pub struct ProtectedStream {
    algorithm: ProtectedStreamAlgorithm,
    key: Vec<u8>,
    counter: u64,
}

impl ProtectedStream {
    pub fn new(algorithm: ProtectedStreamAlgorithm, key: &[u8]) -> Result<Self> {
        let derived_key = match &algorithm {
            ProtectedStreamAlgorithm::ChaCha20 => {
                // ChaCha20 key = SHA512(key)[..32], nonce = SHA512(key)[32..44]
                use sha2::Sha512;
                let mut hasher = Sha512::new();
                hasher.update(key);
                hasher.finalize().to_vec()
            }
            ProtectedStreamAlgorithm::Salsa20 => {
                let mut hasher = Sha256::new();
                hasher.update(key);
                hasher.finalize().to_vec()
            }
            _ => key.to_vec(),
        };

        Ok(Self {
            algorithm,
            key: derived_key,
            counter: 0,
        })
    }

    /// XOR-encrypt/decrypt a value (symmetric)
    pub fn process(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let keystream = self.generate_keystream(data.len())?;
        Ok(data.iter().zip(keystream.iter()).map(|(a, b)| a ^ b).collect())
    }

    fn generate_keystream(&mut self, length: usize) -> Result<Vec<u8>> {
        // Simplified keystream generation — production would use proper ChaCha20/Salsa20
        // This is a placeholder; real impl uses chacha20 crate streaming API
        let mut stream = Vec::with_capacity(length);
        let mut hasher = Sha256::new();
        hasher.update(&self.key);
        hasher.update(self.counter.to_le_bytes());
        let block = hasher.finalize();

        let mut pos = 0;
        let mut block_idx = 0u64;
        while stream.len() < length {
            if pos >= 32 {
                pos = 0;
                block_idx += 1;
                let mut h = Sha256::new();
                h.update(&self.key);
                h.update(block_idx.to_le_bytes());
                let b = h.finalize();
                stream.extend_from_slice(&b[..std::cmp::min(32, length - stream.len())]);
            } else {
                let remaining = length - stream.len();
                let available = 32 - pos;
                let take = std::cmp::min(remaining, available);
                stream.extend_from_slice(&block[pos..pos + take]);
                pos += take;
            }
        }

        self.counter += 1;
        Ok(stream[..length].to_vec())
    }
}
