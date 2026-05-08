//! HMAC-SHA256 block authentication for KDBX 4.x
//!
//! Per KeePassXC source (HmacBlockStream.cpp, KeePass2.cpp):
//! - Block key = SHA512(block_index_le64 || hmac_key_64bytes)  → full 64 bytes as HMAC key
//! - Header key = SHA512(0xFFFFFFFFFFFFFFFF_le64 || hmac_key_64bytes) → full 64 bytes as HMAC key

use crate::error::{KeePassExError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute HMAC-SHA256 for a data block
/// block_key = SHA512(block_index_le64 || hmac_key) — full 64 bytes used as HMAC key
pub fn compute_block_hmac(key: &[u8], block_index: u64, data: &[u8]) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha512};

    // Block key = SHA512(block_index_le64 || hmac_key) — 64 bytes
    let mut hasher = Sha512::new();
    hasher.update(block_index.to_le_bytes());
    hasher.update(key);
    let block_key = hasher.finalize(); // 64 bytes — use ALL of them

    // HMAC-SHA256 with full 64-byte block key
    let mut mac = HmacSha256::new_from_slice(&block_key)
        .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;

    mac.update(block_index.to_le_bytes().as_ref());
    mac.update(&(data.len() as u32).to_le_bytes());
    mac.update(data);

    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Ok(out)
}

/// Verify HMAC-SHA256 for a data block
pub fn verify_block_hmac(
    key: &[u8],
    block_index: u64,
    data: &[u8],
    expected: &[u8; 32],
) -> Result<()> {
    let computed = compute_block_hmac(key, block_index, data)?;
    if computed != *expected {
        return Err(KeePassExError::HmacVerificationFailed);
    }
    Ok(())
}

/// Compute header HMAC
/// header_key = SHA512(0xFFFFFFFFFFFFFFFF_le64 || hmac_key) — full 64 bytes used as HMAC key
pub fn compute_header_hmac(hmac_key: &[u8], header_data: &[u8]) -> Result<[u8; 32]> {
    use sha2::{Digest, Sha512};

    // Header HMAC key = SHA512(UINT64_MAX_le64 || hmac_key) — 64 bytes
    let mut hasher = Sha512::new();
    hasher.update(u64::MAX.to_le_bytes());
    hasher.update(hmac_key);
    let header_key = hasher.finalize(); // 64 bytes — use ALL of them

    // HMAC-SHA256 with full 64-byte header key
    let mut mac = HmacSha256::new_from_slice(&header_key)
        .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;
    mac.update(header_data);

    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Ok(out)
}
