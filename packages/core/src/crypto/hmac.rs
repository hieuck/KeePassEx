//! HMAC-SHA256 block authentication for KDBX 4.x

use crate::error::{KeePassExError, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute HMAC-SHA256 for a data block
pub fn compute_block_hmac(key: &[u8], block_index: u64, data: &[u8]) -> Result<[u8; 32]> {
    // Block key = SHA512(block_index_le || hmac_key)[..32]
    use sha2::{Sha512, Digest};
    let mut hasher = Sha512::new();
    hasher.update(block_index.to_le_bytes());
    hasher.update(key);
    let block_key = hasher.finalize();

    let mut mac = HmacSha256::new_from_slice(&block_key[..32])
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
pub fn compute_header_hmac(hmac_key: &[u8], header_data: &[u8]) -> Result<[u8; 32]> {
    use sha2::{Sha512, Digest};
    // Header HMAC key = SHA512(0xFFFFFFFFFFFFFFFF || hmac_key)
    let mut hasher = Sha512::new();
    hasher.update(u64::MAX.to_le_bytes());
    hasher.update(hmac_key);
    let header_key = hasher.finalize();

    let mut mac = HmacSha256::new_from_slice(&header_key[..32])
        .map_err(|e| KeePassExError::KdfFailed(e.to_string()))?;
    mac.update(header_data);

    let result = mac.finalize().into_bytes();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    Ok(out)
}
