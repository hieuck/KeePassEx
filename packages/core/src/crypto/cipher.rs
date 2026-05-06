//! Symmetric cipher implementations for KDBX payload encryption

use crate::error::{KeePassExError, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce,
};
use aes_gcm::{
    aead::Aead as AesAead,
    Aes256Gcm, Key as AesKey, Nonce as AesNonce,
};

/// Supported cipher algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum CipherAlgorithm {
    /// ChaCha20-Poly1305 (KDBX 4.x default, recommended)
    ChaCha20Poly1305,
    /// AES-256-GCM (KDBX 4.x alternative)
    Aes256Gcm,
    /// AES-256-CBC (KDBX 3.1 compat)
    Aes256Cbc,
    /// Twofish-CBC (KDBX 3.1 compat)
    TwofishCbc,
}

impl CipherAlgorithm {
    /// KDBX UUID identifiers
    pub fn uuid_bytes(&self) -> [u8; 16] {
        match self {
            CipherAlgorithm::Aes256Cbc => [
                0x31, 0xC1, 0xF2, 0xE6, 0xBF, 0x71, 0x43, 0x50,
                0xBE, 0x58, 0x05, 0x21, 0x6A, 0xFC, 0x5A, 0xFF,
            ],
            CipherAlgorithm::TwofishCbc => [
                0xAD, 0x68, 0xF2, 0x9F, 0x57, 0x6F, 0x4B, 0xB9,
                0xA3, 0x6A, 0xD4, 0x7A, 0xF9, 0x65, 0x34, 0x6C,
            ],
            CipherAlgorithm::ChaCha20Poly1305 => [
                0xD6, 0x03, 0x8A, 0x2B, 0x8B, 0x6F, 0x4C, 0xB5,
                0xA5, 0x24, 0x33, 0x9A, 0x31, 0xDB, 0xB5, 0x9A,
            ],
            CipherAlgorithm::Aes256Gcm => [
                0x72, 0xAE, 0x8A, 0xEF, 0x3F, 0x6A, 0x4F, 0x0D,
                0xB0, 0xF4, 0x3A, 0x2D, 0x8C, 0x17, 0x2D, 0x4F,
            ],
        }
    }
}

/// Cipher abstraction
pub struct Cipher {
    algorithm: CipherAlgorithm,
    key: Vec<u8>,
    iv: Vec<u8>,
}

impl Cipher {
    pub fn new(algorithm: CipherAlgorithm, key: Vec<u8>, iv: Vec<u8>) -> Self {
        Self { algorithm, key, iv }
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        match &self.algorithm {
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&self.key[..32]);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(&self.iv[..12]);
                cipher
                    .encrypt(nonce, plaintext)
                    .map_err(|e| KeePassExError::EncryptionFailed(e.to_string()))
            }
            CipherAlgorithm::Aes256Gcm => {
                let key = AesKey::<Aes256Gcm>::from_slice(&self.key[..32]);
                let cipher = Aes256Gcm::new(key);
                let nonce = AesNonce::from_slice(&self.iv[..12]);
                cipher
                    .encrypt(nonce, plaintext)
                    .map_err(|e| KeePassExError::EncryptionFailed(e.to_string()))
            }
            _ => Err(KeePassExError::EncryptionFailed(
                "CBC ciphers use streaming mode".into(),
            )),
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        match &self.algorithm {
            CipherAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&self.key[..32]);
                let cipher = ChaCha20Poly1305::new(key);
                let nonce = ChaChaNonce::from_slice(&self.iv[..12]);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|_| KeePassExError::DecryptionFailed)
            }
            CipherAlgorithm::Aes256Gcm => {
                let key = AesKey::<Aes256Gcm>::from_slice(&self.key[..32]);
                let cipher = Aes256Gcm::new(key);
                let nonce = AesNonce::from_slice(&self.iv[..12]);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|_| KeePassExError::DecryptionFailed)
            }
            _ => Err(KeePassExError::DecryptionFailed),
        }
    }
}
