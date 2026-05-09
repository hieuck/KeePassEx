//! Symmetric cipher implementations for KDBX payload encryption
//!
//! KeePassXC cipher mapping (from SymmetricCipher.cpp):
//! - CIPHER_CHACHA20 UUID → Botan "ChaCha20" = pure stream cipher, NOT AEAD
//!   Key: 32 bytes, IV: 12 bytes (nonce), no authentication tag
//! - CIPHER_AES256   UUID → AES-256-CBC, key: 32 bytes, IV: 16 bytes
//! - AES-256-GCM is a KeePassEx extension (not in KeePassXC)

use crate::error::{KeePassExError, Result};

// AES-256-CBC type aliases
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;

/// Supported cipher algorithms
#[derive(Debug, Clone, PartialEq)]
pub enum CipherAlgorithm {
    /// ChaCha20 stream cipher — KDBX 4.x default
    /// UUID: d6038a2b-8b6f-4cb5-a524-339a31dbb59a
    /// KeePassXC uses pure ChaCha20 stream cipher (Botan "ChaCha20"), NOT ChaCha20-Poly1305 AEAD
    ChaCha20Poly1305,
    /// AES-256-GCM — KeePassEx extension
    Aes256Gcm,
    /// AES-256-CBC — KDBX 3.1 and KDBX 4.x alternative
    /// UUID: 31c1f2e6-bf71-4350-be58-05216afc5aff
    Aes256Cbc,
    /// Twofish-CBC — KDBX 3.1 compat
    /// UUID: ad68f29f-576f-4bb9-a36a-d47af965346c
    TwofishCbc,
}

impl CipherAlgorithm {
    /// KDBX UUID bytes (from KeePassXC KeePass2.cpp)
    pub fn uuid_bytes(&self) -> [u8; 16] {
        match self {
            // CIPHER_AES256 = "31c1f2e6-bf71-4350-be58-05216afc5aff"
            CipherAlgorithm::Aes256Cbc => [
                0x31, 0xC1, 0xF2, 0xE6, 0xBF, 0x71, 0x43, 0x50, 0xBE, 0x58, 0x05, 0x21, 0x6A, 0xFC,
                0x5A, 0xFF,
            ],
            // CIPHER_TWOFISH = "ad68f29f-576f-4bb9-a36a-d47af965346c"
            CipherAlgorithm::TwofishCbc => [
                0xAD, 0x68, 0xF2, 0x9F, 0x57, 0x6F, 0x4B, 0xB9, 0xA3, 0x6A, 0xD4, 0x7A, 0xF9, 0x65,
                0x34, 0x6C,
            ],
            // CIPHER_CHACHA20 = "d6038a2b-8b6f-4cb5-a524-339a31dbb59a"
            CipherAlgorithm::ChaCha20Poly1305 => [
                0xD6, 0x03, 0x8A, 0x2B, 0x8B, 0x6F, 0x4C, 0xB5, 0xA5, 0x24, 0x33, 0x9A, 0x31, 0xDB,
                0xB5, 0x9A,
            ],
            CipherAlgorithm::Aes256Gcm => [
                0x72, 0xAE, 0x8A, 0xEF, 0x3F, 0x6A, 0x4F, 0x0D, 0xB0, 0xF4, 0x3A, 0x2D, 0x8C, 0x17,
                0x2D, 0x4F,
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
                // Pure ChaCha20 stream cipher — same operation for encrypt/decrypt
                chacha20_stream_crypt(&self.key, &self.iv, plaintext)
            }
            CipherAlgorithm::Aes256Gcm => {
                use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
                let key = Key::<Aes256Gcm>::from_slice(&self.key[..32]);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&self.iv[..12]);
                cipher
                    .encrypt(nonce, plaintext)
                    .map_err(|e| KeePassExError::EncryptionFailed(e.to_string()))
            }
            CipherAlgorithm::Aes256Cbc => {
                use aes::cipher::{block_padding::Pkcs7, BlockEncryptMut, KeyIvInit};
                let encryptor = Aes256CbcEnc::new_from_slices(&self.key[..32], &self.iv[..16])
                    .map_err(|_| KeePassExError::EncryptionFailed("AES-CBC init".into()))?;
                Ok(encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext))
            }
            CipherAlgorithm::TwofishCbc => Err(KeePassExError::EncryptionFailed(
                "Twofish not supported for encryption".into(),
            )),
        }
    }

    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        match &self.algorithm {
            CipherAlgorithm::ChaCha20Poly1305 => {
                // Pure ChaCha20 stream cipher — XOR with keystream, same as encrypt
                chacha20_stream_crypt(&self.key, &self.iv, ciphertext)
            }
            CipherAlgorithm::Aes256Gcm => {
                use aes_gcm::{aead::Aead, Aes256Gcm, Key, KeyInit, Nonce};
                let key = Key::<Aes256Gcm>::from_slice(&self.key[..32]);
                let cipher = Aes256Gcm::new(key);
                let nonce = Nonce::from_slice(&self.iv[..12]);
                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|_| KeePassExError::DecryptionFailed)
            }
            CipherAlgorithm::Aes256Cbc => {
                if self.key.len() < 32 || self.iv.len() < 16 {
                    return Err(KeePassExError::DecryptionFailed);
                }
                use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
                let decryptor = Aes256CbcDec::new_from_slices(&self.key[..32], &self.iv[..16])
                    .map_err(|_| KeePassExError::DecryptionFailed)?;
                // KDBX 3.1 block stream has no PKCS7 padding
                let mut buf = ciphertext.to_vec();
                decryptor
                    .decrypt_padded_mut::<NoPadding>(&mut buf)
                    .map(|s| s.to_vec())
                    .map_err(|_| KeePassExError::DecryptionFailed)
            }
            CipherAlgorithm::TwofishCbc => Err(KeePassExError::Other(
                "Twofish-CBC not supported; re-save vault with AES or ChaCha20".into(),
            )),
        }
    }
}

/// ChaCha20 pure stream cipher (no Poly1305 authentication tag)
///
/// KeePassXC uses Botan "ChaCha20" mode which is a raw stream cipher.
/// This is NOT ChaCha20-Poly1305 AEAD — there is no 16-byte auth tag.
/// Stream cipher: ciphertext = plaintext XOR keystream (encrypt == decrypt)
///
/// Key: 32 bytes, Nonce: 12 bytes (IETF ChaCha20 variant)
fn chacha20_stream_crypt(key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    use chacha20::cipher::{KeyIvInit, StreamCipher};
    use chacha20::ChaCha20;

    if key.len() < 32 {
        return Err(KeePassExError::EncryptionFailed(format!(
            "ChaCha20 key must be 32 bytes, got {}",
            key.len()
        )));
    }
    if iv.len() < 12 {
        return Err(KeePassExError::EncryptionFailed(format!(
            "ChaCha20 nonce must be 12 bytes, got {}",
            iv.len()
        )));
    }

    let key_arr: &[u8; 32] = key[..32].try_into().unwrap();
    let nonce_arr: &[u8; 12] = iv[..12].try_into().unwrap();

    let mut cipher = ChaCha20::new(key_arr.into(), nonce_arr.into());
    let mut output = data.to_vec();
    cipher.apply_keystream(&mut output);
    Ok(output)
}
