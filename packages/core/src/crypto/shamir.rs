//! Shamir's Secret Sharing — Distributed Vault Key Sharding
//!
//! Splits a vault master key into N shards where any M shards can reconstruct
//! the original secret. Uses GF(256) finite field arithmetic.
//!
//! # Example
//! ```no_run
//! use keepassex_core::crypto::{split_secret, combine_shards};
//! let secret = b"my-32-byte-master-key-here-12345";
//! let shards = split_secret(secret, 3, 5).unwrap(); // 3-of-5
//! let recovered = combine_shards(&shards[..3]).unwrap();
//! assert_eq!(secret, recovered.as_slice());
//! ```

use crate::error::{KeePassExError, Result};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// A single shard of a split secret.
#[derive(Debug, Clone, ZeroizeOnDrop)]
pub struct SecretShard {
    /// Shard index (1-based, never 0)
    pub index: u8,
    /// Shard data (same length as original secret)
    pub data: Vec<u8>,
    /// Total shards (N)
    pub total: u8,
    /// Threshold required (M)
    pub threshold: u8,
    /// Optional label for this shard
    pub label: Option<String>,
}

impl SecretShard {
    /// Serialize shard to bytes for storage/transport.
    /// Format: [index(1)] [total(1)] [threshold(1)] [data_len(2 LE)] [data...]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(5 + self.data.len());
        out.push(self.index);
        out.push(self.total);
        out.push(self.threshold);
        let len = self.data.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&self.data);
        out
    }

    /// Deserialize shard from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 5 {
            return Err(KeePassExError::Other("Shard too short".into()));
        }
        let index = bytes[0];
        let total = bytes[1];
        let threshold = bytes[2];
        let data_len = u16::from_le_bytes([bytes[3], bytes[4]]) as usize;
        if bytes.len() < 5 + data_len {
            return Err(KeePassExError::Other("Shard data truncated".into()));
        }
        Ok(SecretShard {
            index,
            total,
            threshold,
            data: bytes[5..5 + data_len].to_vec(),
            label: None,
        })
    }
}

/// Split a secret into `total` shards where any `threshold` shards can
/// reconstruct the original.
///
/// # Arguments
/// * `secret` — The secret bytes to split (e.g., 32-byte vault key)
/// * `threshold` — Minimum shards needed to reconstruct (M)
/// * `total` — Total shards to generate (N), must be >= threshold
///
/// # Errors
/// Returns error if threshold > total, or threshold < 2, or total > 255.
pub fn split_secret(secret: &[u8], threshold: u8, total: u8) -> Result<Vec<SecretShard>> {
    if threshold < 2 {
        return Err(KeePassExError::Other("Threshold must be at least 2".into()));
    }
    if total < threshold {
        return Err(KeePassExError::Other(
            "Total shards must be >= threshold".into(),
        ));
    }
    if total > 255 {
        return Err(KeePassExError::Other(
            "Total shards cannot exceed 255".into(),
        ));
    }
    if secret.is_empty() {
        return Err(KeePassExError::Other("Secret cannot be empty".into()));
    }

    let mut rng = rand::thread_rng();
    let mut shards: Vec<SecretShard> = (1..=total)
        .map(|i| SecretShard {
            index: i,
            total,
            threshold,
            data: vec![0u8; secret.len()],
            label: None,
        })
        .collect();

    // Process each byte of the secret independently over GF(256)
    for (byte_idx, &secret_byte) in secret.iter().enumerate() {
        // Generate random polynomial coefficients a_1..a_{threshold-1}
        // The polynomial is: f(x) = secret_byte + a_1*x + a_2*x^2 + ... + a_{t-1}*x^{t-1}
        let mut coeffs = vec![0u8; threshold as usize];
        coeffs[0] = secret_byte;
        rng.fill_bytes(&mut coeffs[1..]);

        // Evaluate polynomial at each shard index
        for shard in shards.iter_mut() {
            shard.data[byte_idx] = gf256_poly_eval(&coeffs, shard.index);
        }

        // Zeroize coefficients after use
        coeffs.zeroize();
    }

    Ok(shards)
}

/// Reconstruct the original secret from at least `threshold` shards.
///
/// # Arguments
/// * `shards` — At least `threshold` shards (can be any subset)
///
/// # Errors
/// Returns error if fewer than threshold shards provided, or shards are invalid.
pub fn combine_shards(shards: &[SecretShard]) -> Result<Vec<u8>> {
    if shards.is_empty() {
        return Err(KeePassExError::Other("No shards provided".into()));
    }

    let threshold = shards[0].threshold as usize;
    let secret_len = shards[0].data.len();

    if shards.len() < threshold {
        return Err(KeePassExError::Other(format!(
            "Need at least {} shards, got {}",
            threshold,
            shards.len()
        )));
    }

    // Validate all shards have same length and threshold
    for shard in shards {
        if shard.data.len() != secret_len {
            return Err(KeePassExError::Other("Shard length mismatch".into()));
        }
        if shard.threshold as usize != threshold {
            return Err(KeePassExError::Other("Shard threshold mismatch".into()));
        }
        if shard.index == 0 {
            return Err(KeePassExError::Other("Shard index cannot be 0".into()));
        }
    }

    // Use only the first `threshold` shards
    let active_shards = &shards[..threshold];
    let mut secret = vec![0u8; secret_len];

    // Lagrange interpolation over GF(256) for each byte
    for byte_idx in 0..secret_len {
        let points: Vec<(u8, u8)> = active_shards
            .iter()
            .map(|s| (s.index, s.data[byte_idx]))
            .collect();
        secret[byte_idx] = gf256_lagrange_interpolate(&points);
    }

    Ok(secret)
}

// ─── GF(256) Arithmetic ───────────────────────────────────────────────────────

/// GF(256) multiplication using the irreducible polynomial x^8 + x^4 + x^3 + x + 1
/// (same as AES, 0x11b)
fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result = 0u8;
    while b > 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        let high_bit = a & 0x80;
        a <<= 1;
        if high_bit != 0 {
            a ^= 0x1b; // x^8 + x^4 + x^3 + x + 1 (mod x^8)
        }
        b >>= 1;
    }
    result
}

/// GF(256) division: a / b
fn gf256_div(a: u8, b: u8) -> u8 {
    if b == 0 {
        panic!("GF(256) division by zero");
    }
    if a == 0 {
        return 0;
    }
    gf256_mul(a, gf256_inv(b))
}

/// GF(256) multiplicative inverse using extended Euclidean algorithm
fn gf256_inv(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    // Use lookup: a^254 = a^{-1} in GF(256) (Fermat's little theorem)
    let mut result = a;
    for _ in 0..6 {
        result = gf256_mul(result, result);
        result = gf256_mul(result, a);
    }
    // result = a^127, need a^254 = (a^127)^2
    gf256_mul(result, result)
}

/// Evaluate polynomial f(x) = coeffs[0] + coeffs[1]*x + ... over GF(256)
fn gf256_poly_eval(coeffs: &[u8], x: u8) -> u8 {
    // Horner's method: f(x) = c0 + x*(c1 + x*(c2 + ...))
    let mut result = 0u8;
    for &coeff in coeffs.iter().rev() {
        result = gf256_mul(result, x) ^ coeff;
    }
    result
}

/// Lagrange interpolation at x=0 over GF(256)
/// Given points (x_i, y_i), compute f(0)
fn gf256_lagrange_interpolate(points: &[(u8, u8)]) -> u8 {
    let mut secret = 0u8;
    let n = points.len();

    for i in 0..n {
        let (x_i, y_i) = points[i];
        let mut numerator = 1u8;
        let mut denominator = 1u8;

        for j in 0..n {
            if i == j {
                continue;
            }
            let (x_j, _) = points[j];
            // numerator *= (0 - x_j) = x_j in GF(256) (since -x = x)
            numerator = gf256_mul(numerator, x_j);
            // denominator *= (x_i - x_j) = x_i XOR x_j in GF(256)
            denominator = gf256_mul(denominator, x_i ^ x_j);
        }

        let lagrange_coeff = gf256_div(numerator, denominator);
        secret ^= gf256_mul(y_i, lagrange_coeff);
    }

    secret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_combine_2_of_3() {
        let secret = b"my-super-secret-32-byte-key-here";
        let shards = split_secret(secret, 2, 3).unwrap();
        assert_eq!(shards.len(), 3);

        // Any 2 shards should reconstruct
        let recovered = combine_shards(&shards[..2]).unwrap();
        assert_eq!(recovered, secret);

        let recovered2 = combine_shards(&[shards[0].clone(), shards[2].clone()]).unwrap();
        assert_eq!(recovered2, secret);

        let recovered3 = combine_shards(&[shards[1].clone(), shards[2].clone()]).unwrap();
        assert_eq!(recovered3, secret);
    }

    #[test]
    fn test_split_combine_3_of_5() {
        let secret = b"another-secret-key-for-testing!!";
        let shards = split_secret(secret, 3, 5).unwrap();
        assert_eq!(shards.len(), 5);

        // Any 3 shards should reconstruct
        let recovered = combine_shards(&shards[..3]).unwrap();
        assert_eq!(recovered, secret);

        // Different combination
        let subset = vec![shards[0].clone(), shards[2].clone(), shards[4].clone()];
        let recovered2 = combine_shards(&subset).unwrap();
        assert_eq!(recovered2, secret);
    }

    #[test]
    fn test_insufficient_shards_fails() {
        let secret = b"test-secret-key-for-shamir-split";
        let shards = split_secret(secret, 3, 5).unwrap();

        // Only 2 shards when 3 needed — should fail
        let result = combine_shards(&shards[..2]);
        assert!(result.is_err());
    }

    #[test]
    fn test_shard_serialization() {
        let secret = b"serialize-test-secret-key-here!!";
        let shards = split_secret(secret, 2, 3).unwrap();

        // Serialize and deserialize
        let bytes = shards[0].to_bytes();
        let restored = SecretShard::from_bytes(&bytes).unwrap();
        assert_eq!(restored.index, shards[0].index);
        assert_eq!(restored.data, shards[0].data);
        assert_eq!(restored.threshold, shards[0].threshold);
        assert_eq!(restored.total, shards[0].total);
    }

    #[test]
    fn test_threshold_equals_total() {
        // 3-of-3: all shards required
        let secret = b"all-shards-required-test-key-123";
        let shards = split_secret(secret, 3, 3).unwrap();
        let recovered = combine_shards(&shards).unwrap();
        assert_eq!(recovered, secret);
    }

    #[test]
    fn test_invalid_threshold() {
        let secret = b"test";
        assert!(split_secret(secret, 1, 3).is_err()); // threshold < 2
        assert!(split_secret(secret, 4, 3).is_err()); // threshold > total
    }

    #[test]
    fn test_gf256_mul_identity() {
        // a * 1 = a
        for a in 0u8..=255 {
            assert_eq!(gf256_mul(a, 1), a);
        }
    }

    #[test]
    fn test_gf256_mul_zero() {
        // a * 0 = 0
        for a in 0u8..=255 {
            assert_eq!(gf256_mul(a, 0), 0);
        }
    }

    #[test]
    fn test_gf256_inv() {
        // a * inv(a) = 1 for all non-zero a
        for a in 1u8..=255 {
            assert_eq!(gf256_mul(a, gf256_inv(a)), 1);
        }
    }

    #[test]
    fn test_arbitrary_length_secret() {
        // Test with various secret lengths
        for len in [1, 16, 32, 64, 128, 256] {
            let secret: Vec<u8> = (0..len).map(|i| i as u8).collect();
            let shards = split_secret(&secret, 3, 5).unwrap();
            let recovered = combine_shards(&shards[..3]).unwrap();
            assert_eq!(recovered, secret, "Failed for length {}", len);
        }
    }
}
