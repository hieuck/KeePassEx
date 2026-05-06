//! Steganography — Hide encrypted vault inside image/video files
//!
//! Embeds an encrypted KDBX vault into the LSB (Least Significant Bits)
//! of PNG/JPEG images or MP4 video metadata. The resulting file is
//! visually indistinguishable from the original.
//!
//! # Security
//! - The vault is encrypted with ChaCha20-Poly1305 before embedding
//! - The embedding key is derived from a separate steganography password
//! - Without the steg password, the vault cannot be extracted
//! - The carrier file remains fully functional (viewable as image/video)
//!
//! # Supported Formats
//! - PNG: LSB embedding in pixel data (lossless, recommended)
//! - JPEG: Embedding in EXIF/APP1 metadata (lossy compression safe)
//! - MP4/AVI: Embedding in custom metadata atoms/chunks
//!
//! # Capacity
//! - PNG: ~1 bit per pixel channel → 1MB image ≈ 125KB capacity
//! - JPEG: EXIF APP1 segment ≈ 64KB capacity
//! - MP4: Custom 'kpxv' atom ≈ unlimited capacity

pub mod jpeg;
pub mod png;
pub mod video;

use crate::error::{KeePassExError, Result};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use rand::RngCore;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// Steganography carrier format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StegFormat {
    Png,
    Jpeg,
    Mp4,
    Avi,
}

impl StegFormat {
    /// Detect format from file magic bytes
    pub fn detect(data: &[u8]) -> Option<Self> {
        if data.len() < 4 {
            return None;
        }
        match &data[..4] {
            [0x89, 0x50, 0x4E, 0x47] => Some(Self::Png), // PNG magic
            [0xFF, 0xD8, 0xFF, _] => Some(Self::Jpeg),   // JPEG SOI
            [0x00, 0x00, 0x00, _] if data.len() >= 8 && &data[4..8] == b"ftyp" => Some(Self::Mp4),
            [0x52, 0x49, 0x46, 0x46] => Some(Self::Avi), // RIFF
            _ => None,
        }
    }
}

/// Magic header for embedded vault data
const STEG_MAGIC: &[u8; 8] = b"KPX\x00STG\x01";

/// Embed an encrypted vault into a carrier file.
///
/// # Arguments
/// * `carrier` — Original image/video bytes
/// * `vault_data` — Raw KDBX vault bytes to embed
/// * `steg_password` — Password for the steganography layer (separate from vault password)
///
/// # Returns
/// Modified carrier file bytes with vault embedded
pub fn embed(carrier: &[u8], vault_data: &[u8], steg_password: &str) -> Result<Vec<u8>> {
    let format = StegFormat::detect(carrier)
        .ok_or_else(|| KeePassExError::Other("Unsupported carrier format".into()))?;

    // Encrypt vault data with steg password
    let encrypted = encrypt_payload(vault_data, steg_password)?;

    // Build payload: [magic(8)] [payload_len(4 LE)] [encrypted_data...]
    let mut payload = Vec::with_capacity(12 + encrypted.len());
    payload.extend_from_slice(STEG_MAGIC);
    payload.extend_from_slice(&(encrypted.len() as u32).to_le_bytes());
    payload.extend_from_slice(&encrypted);

    // Embed into carrier based on format
    match format {
        StegFormat::Png => png::embed_png(carrier, &payload),
        StegFormat::Jpeg => jpeg::embed_jpeg(carrier, &payload),
        StegFormat::Mp4 => video::embed_mp4(carrier, &payload),
        StegFormat::Avi => video::embed_avi(carrier, &payload),
    }
}

/// Extract and decrypt a vault from a carrier file.
///
/// # Arguments
/// * `carrier` — Carrier file bytes (with embedded vault)
/// * `steg_password` — Steganography password used during embedding
///
/// # Returns
/// Original KDBX vault bytes
pub fn extract(carrier: &[u8], steg_password: &str) -> Result<Vec<u8>> {
    let format = StegFormat::detect(carrier)
        .ok_or_else(|| KeePassExError::Other("Unsupported carrier format".into()))?;

    // Extract raw payload from carrier
    let payload = match format {
        StegFormat::Png => png::extract_png(carrier)?,
        StegFormat::Jpeg => jpeg::extract_jpeg(carrier)?,
        StegFormat::Mp4 => video::extract_mp4(carrier)?,
        StegFormat::Avi => video::extract_avi(carrier)?,
    };

    // Validate magic header
    if payload.len() < 12 || &payload[..8] != STEG_MAGIC {
        return Err(KeePassExError::Other(
            "No KeePassEx vault found in this file".into(),
        ));
    }

    let payload_len =
        u32::from_le_bytes([payload[8], payload[9], payload[10], payload[11]]) as usize;
    if payload.len() < 12 + payload_len {
        return Err(KeePassExError::Other(
            "Embedded vault data is truncated".into(),
        ));
    }

    let encrypted = &payload[12..12 + payload_len];
    decrypt_payload(encrypted, steg_password)
}

/// Check if a carrier file contains an embedded vault (without decrypting).
pub fn has_embedded_vault(carrier: &[u8]) -> bool {
    let format = match StegFormat::detect(carrier) {
        Some(f) => f,
        None => return false,
    };

    let payload = match format {
        StegFormat::Png => png::extract_png(carrier),
        StegFormat::Jpeg => jpeg::extract_jpeg(carrier),
        StegFormat::Mp4 => video::extract_mp4(carrier),
        StegFormat::Avi => video::extract_avi(carrier),
    };

    match payload {
        Ok(p) => p.len() >= 8 && &p[..8] == STEG_MAGIC,
        Err(_) => false,
    }
}

/// Calculate maximum vault size that can be embedded in a carrier.
pub fn max_capacity(carrier: &[u8]) -> Option<usize> {
    let format = StegFormat::detect(carrier)?;
    match format {
        StegFormat::Png => {
            // PNG: 1 bit per channel, 3 channels per pixel
            // Rough estimate: image_size / 8 bytes capacity
            // Subtract header overhead
            Some(carrier.len() / 8)
        }
        StegFormat::Jpeg => {
            // JPEG: EXIF APP1 segment max ~64KB
            Some(65_536)
        }
        StegFormat::Mp4 | StegFormat::Avi => {
            // Video: custom atom, essentially unlimited
            Some(usize::MAX / 2)
        }
    }
}

// ─── Encryption ───────────────────────────────────────────────────────────────

/// Encrypt payload with ChaCha20-Poly1305 using steg password
fn encrypt_payload(data: &[u8], password: &str) -> Result<Vec<u8>> {
    let mut key_bytes = derive_steg_key(password);
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, data)
        .map_err(|_| KeePassExError::Other("Steg encryption failed".into()))?;

    // Output: [nonce(12)] [ciphertext...]
    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    key_bytes.zeroize();
    Ok(out)
}

/// Decrypt payload with ChaCha20-Poly1305 using steg password
fn decrypt_payload(data: &[u8], password: &str) -> Result<Vec<u8>> {
    if data.len() < 12 {
        return Err(KeePassExError::Other("Encrypted payload too short".into()));
    }

    let mut key_bytes = derive_steg_key(password);
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);

    let nonce = Nonce::from_slice(&data[..12]);
    let plaintext = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|_| KeePassExError::Other("Steg decryption failed — wrong password?".into()))?;

    key_bytes.zeroize();
    Ok(plaintext)
}

/// Derive 32-byte key from steganography password using SHA-256
fn derive_steg_key(password: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"kpx-steg-v1:");
    hasher.update(password.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steg_format_detect_png() {
        let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        assert_eq!(StegFormat::detect(&png_magic), Some(StegFormat::Png));
    }

    #[test]
    fn test_steg_format_detect_jpeg() {
        let jpeg_magic = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
        assert_eq!(StegFormat::detect(&jpeg_magic), Some(StegFormat::Jpeg));
    }

    #[test]
    fn test_steg_format_detect_mp4() {
        let mp4_magic = [0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70];
        assert_eq!(StegFormat::detect(&mp4_magic), Some(StegFormat::Mp4));
    }

    #[test]
    fn test_steg_format_detect_unknown() {
        let unknown = [0x00, 0x01, 0x02, 0x03];
        assert_eq!(StegFormat::detect(&unknown), None);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"This is a test vault payload for steganography";
        let password = "steg-password-123";

        let encrypted = encrypt_payload(data, password).unwrap();
        let decrypted = decrypt_payload(&encrypted, password).unwrap();

        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_decrypt_wrong_password_fails() {
        let data = b"Secret vault data";
        let encrypted = encrypt_payload(data, "correct-password").unwrap();
        let result = decrypt_payload(&encrypted, "wrong-password");
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_steg_key_deterministic() {
        let key1 = derive_steg_key("my-password");
        let key2 = derive_steg_key("my-password");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_steg_key_different_passwords() {
        let key1 = derive_steg_key("password1");
        let key2 = derive_steg_key("password2");
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_magic_header_validation() {
        // Payload without magic should fail extraction
        let fake_payload = b"not-a-kpx-vault";
        let result = extract(
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], // PNG magic
            "password",
        );
        // Should fail (no embedded data in minimal PNG)
        assert!(result.is_err());
        let _ = fake_payload; // suppress unused warning
    }
}
