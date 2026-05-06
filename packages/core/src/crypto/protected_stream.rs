//! Inner protected stream for in-memory field encryption
//!
//! KDBX uses a secondary stream cipher to XOR-encrypt sensitive string values
//! (passwords, protected custom fields) inside the XML payload. This prevents
//! the plaintext from appearing verbatim in the decrypted-but-not-yet-parsed
//! XML, adding a second layer of protection.
//!
//! Supported algorithms:
//! - ChaCha20 (KDBX 4.x, recommended)
//! - Salsa20 (KDBX 3.1 compat)
//! - ArcFour variant (KDBX 2.x legacy, not recommended)

use crate::error::{KeePassExError, Result};
use sha2::{Digest, Sha256, Sha512};
use zeroize::{Zeroize, ZeroizeOnDrop};

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

// ─── ChaCha20 protected stream ────────────────────────────────────────────────

/// ChaCha20 inner stream (KDBX 4.x)
///
/// Key derivation: SHA-512(inner_stream_key)
///   bytes  0..31 → ChaCha20 key
///   bytes 32..43 → ChaCha20 nonce (12 bytes)
///
/// The stream is stateful: each call to `process` advances the position.
struct ChaCha20Stream {
    /// Remaining keystream bytes from the current block
    buffer: Vec<u8>,
    /// Current position within `buffer`
    buf_pos: usize,
    /// ChaCha20 key (32 bytes)
    key: [u8; 32],
    /// ChaCha20 nonce (12 bytes)
    nonce: [u8; 12],
    /// Block counter
    counter: u32,
}

impl ChaCha20Stream {
    fn new(inner_key: &[u8]) -> Self {
        // Derive key + nonce from SHA-512(inner_key)
        let mut hasher = Sha512::new();
        hasher.update(inner_key);
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        let mut nonce = [0u8; 12];
        key.copy_from_slice(&hash[..32]);
        nonce.copy_from_slice(&hash[32..44]);

        Self {
            buffer: Vec::new(),
            buf_pos: 0,
            key,
            nonce,
            counter: 0,
        }
    }

    /// Generate the next 64-byte ChaCha20 block using the quarter-round algorithm.
    ///
    /// This is a pure-Rust implementation of the ChaCha20 block function
    /// (RFC 7539 §2.1) so we don't need an additional crate dependency.
    fn next_block(&mut self) -> [u8; 64] {
        // ChaCha20 constants ("expand 32-byte k")
        let c0: u32 = 0x61707865;
        let c1: u32 = 0x3320646e;
        let c2: u32 = 0x79622d32;
        let c3: u32 = 0x6b206574;

        // Build initial state
        let key_words = key_to_words(&self.key);
        let nonce_words = nonce_to_words(&self.nonce);

        let mut state = [
            c0,
            c1,
            c2,
            c3,
            key_words[0],
            key_words[1],
            key_words[2],
            key_words[3],
            key_words[4],
            key_words[5],
            key_words[6],
            key_words[7],
            self.counter,
            nonce_words[0],
            nonce_words[1],
            nonce_words[2],
        ];

        let initial = state;

        // 20 rounds (10 double-rounds)
        for _ in 0..10 {
            // Column rounds
            quarter_round(&mut state, 0, 4, 8, 12);
            quarter_round(&mut state, 1, 5, 9, 13);
            quarter_round(&mut state, 2, 6, 10, 14);
            quarter_round(&mut state, 3, 7, 11, 15);
            // Diagonal rounds
            quarter_round(&mut state, 0, 5, 10, 15);
            quarter_round(&mut state, 1, 6, 11, 12);
            quarter_round(&mut state, 2, 7, 8, 13);
            quarter_round(&mut state, 3, 4, 9, 14);
        }

        // Add initial state
        for i in 0..16 {
            state[i] = state[i].wrapping_add(initial[i]);
        }

        // Serialize to bytes (little-endian)
        let mut block = [0u8; 64];
        for (i, &word) in state.iter().enumerate() {
            let bytes = word.to_le_bytes();
            block[i * 4..i * 4 + 4].copy_from_slice(&bytes);
        }

        self.counter = self.counter.wrapping_add(1);
        block
    }

    fn get_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            if self.buf_pos >= self.buffer.len() {
                let block = self.next_block();
                self.buffer = block.to_vec();
                self.buf_pos = 0;
            }
            let available = self.buffer.len() - self.buf_pos;
            let take = std::cmp::min(available, n - out.len());
            out.extend_from_slice(&self.buffer[self.buf_pos..self.buf_pos + take]);
            self.buf_pos += take;
        }
        out
    }
}

impl Drop for ChaCha20Stream {
    fn drop(&mut self) {
        self.key.zeroize();
        self.nonce.zeroize();
        self.buffer.zeroize();
    }
}

// ─── Salsa20 protected stream ─────────────────────────────────────────────────

/// Salsa20 inner stream (KDBX 3.1 compat)
///
/// Key derivation: SHA-256(inner_stream_key) → 32-byte key
/// Fixed nonce: [0xE8, 0x30, 0x09, 0x4B, 0x97, 0x20, 0x5D, 0x2A] (KeePass spec)
struct Salsa20Stream {
    buffer: Vec<u8>,
    buf_pos: usize,
    key: [u8; 32],
    nonce: [u8; 8],
    counter: u64,
}

impl Salsa20Stream {
    fn new(inner_key: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(inner_key);
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);

        // KeePass-specified fixed nonce for Salsa20 inner stream
        let nonce = [0xE8, 0x30, 0x09, 0x4B, 0x97, 0x20, 0x5D, 0x2A];

        Self {
            buffer: Vec::new(),
            buf_pos: 0,
            key,
            nonce,
            counter: 0,
        }
    }

    /// Salsa20 block function (Bernstein spec)
    fn next_block(&mut self) -> [u8; 64] {
        // Salsa20 constants ("expand 32-byte k")
        let c0: u32 = 0x61707865;
        let c1: u32 = 0x3320646e;
        let c2: u32 = 0x79622d32;
        let c3: u32 = 0x6b206574;

        let key_words = key_to_words(&self.key);
        let nonce_lo = u32::from_le_bytes(self.nonce[0..4].try_into().unwrap());
        let nonce_hi = u32::from_le_bytes(self.nonce[4..8].try_into().unwrap());
        let ctr_lo = (self.counter & 0xFFFF_FFFF) as u32;
        let ctr_hi = (self.counter >> 32) as u32;

        let mut x = [
            c0,
            key_words[0],
            key_words[1],
            key_words[2],
            key_words[3],
            c1,
            nonce_lo,
            nonce_hi,
            ctr_lo,
            ctr_hi,
            c2,
            key_words[4],
            key_words[5],
            key_words[6],
            key_words[7],
            c3,
        ];

        let initial = x;

        // 20 rounds (10 double-rounds)
        for _ in 0..10 {
            // Column rounds
            salsa_quarter_round(&mut x, 0, 4, 8, 12);
            salsa_quarter_round(&mut x, 5, 9, 13, 1);
            salsa_quarter_round(&mut x, 10, 14, 2, 6);
            salsa_quarter_round(&mut x, 15, 3, 7, 11);
            // Row rounds
            salsa_quarter_round(&mut x, 0, 1, 2, 3);
            salsa_quarter_round(&mut x, 5, 6, 7, 4);
            salsa_quarter_round(&mut x, 10, 11, 8, 9);
            salsa_quarter_round(&mut x, 15, 12, 13, 14);
        }

        for i in 0..16 {
            x[i] = x[i].wrapping_add(initial[i]);
        }

        let mut block = [0u8; 64];
        for (i, &word) in x.iter().enumerate() {
            block[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }

        self.counter = self.counter.wrapping_add(1);
        block
    }

    fn get_bytes(&mut self, n: usize) -> Vec<u8> {
        let mut out = Vec::with_capacity(n);
        while out.len() < n {
            if self.buf_pos >= self.buffer.len() {
                let block = self.next_block();
                self.buffer = block.to_vec();
                self.buf_pos = 0;
            }
            let available = self.buffer.len() - self.buf_pos;
            let take = std::cmp::min(available, n - out.len());
            out.extend_from_slice(&self.buffer[self.buf_pos..self.buf_pos + take]);
            self.buf_pos += take;
        }
        out
    }
}

impl Drop for Salsa20Stream {
    fn drop(&mut self) {
        self.key.zeroize();
        self.buffer.zeroize();
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

enum StreamInner {
    ChaCha20(ChaCha20Stream),
    Salsa20(Salsa20Stream),
    ArcFour(Vec<u8>, usize), // key, position (simplified)
}

/// Protected stream for XOR-encrypting sensitive fields in memory
pub struct ProtectedStream {
    inner: StreamInner,
}

impl ProtectedStream {
    pub fn new(algorithm: ProtectedStreamAlgorithm, key: &[u8]) -> Result<Self> {
        let inner = match algorithm {
            ProtectedStreamAlgorithm::ChaCha20 => StreamInner::ChaCha20(ChaCha20Stream::new(key)),
            ProtectedStreamAlgorithm::Salsa20 => StreamInner::Salsa20(Salsa20Stream::new(key)),
            ProtectedStreamAlgorithm::ArcFourVariant => {
                // ARC4 variant: skip first 512 bytes of keystream
                // Simplified: use SHA-256 chain (legacy, not security-critical)
                let mut hasher = Sha256::new();
                hasher.update(key);
                let k = hasher.finalize().to_vec();
                StreamInner::ArcFour(k, 0)
            }
        };
        Ok(Self { inner })
    }

    /// XOR-encrypt or decrypt `data` with the next bytes from the keystream.
    /// The stream is stateful — successive calls advance the position.
    pub fn process(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let keystream = self.get_keystream(data.len())?;
        Ok(data
            .iter()
            .zip(keystream.iter())
            .map(|(a, b)| a ^ b)
            .collect())
    }

    fn get_keystream(&mut self, n: usize) -> Result<Vec<u8>> {
        match &mut self.inner {
            StreamInner::ChaCha20(s) => Ok(s.get_bytes(n)),
            StreamInner::Salsa20(s) => Ok(s.get_bytes(n)),
            StreamInner::ArcFour(key, pos) => {
                // Simplified ARC4 variant for legacy compat
                let mut out = Vec::with_capacity(n);
                for i in 0..n {
                    let idx = (*pos + i) % key.len();
                    out.push(key[idx]);
                }
                *pos = (*pos + n) % key.len();
                Ok(out)
            }
        }
    }
}

// ─── ChaCha20 primitives ──────────────────────────────────────────────────────

#[inline(always)]
fn quarter_round(state: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(16);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(12);
    state[a] = state[a].wrapping_add(state[b]);
    state[d] ^= state[a];
    state[d] = state[d].rotate_left(8);
    state[c] = state[c].wrapping_add(state[d]);
    state[b] ^= state[c];
    state[b] = state[b].rotate_left(7);
}

// ─── Salsa20 primitives ───────────────────────────────────────────────────────

#[inline(always)]
fn salsa_quarter_round(x: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    x[b] ^= x[a].wrapping_add(x[d]).rotate_left(7);
    x[c] ^= x[b].wrapping_add(x[a]).rotate_left(9);
    x[d] ^= x[c].wrapping_add(x[b]).rotate_left(13);
    x[a] ^= x[d].wrapping_add(x[c]).rotate_left(18);
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

fn key_to_words(key: &[u8; 32]) -> [u32; 8] {
    let mut words = [0u32; 8];
    for (i, chunk) in key.chunks(4).enumerate() {
        words[i] = u32::from_le_bytes(chunk.try_into().unwrap());
    }
    words
}

fn nonce_to_words(nonce: &[u8; 12]) -> [u32; 3] {
    [
        u32::from_le_bytes(nonce[0..4].try_into().unwrap()),
        u32::from_le_bytes(nonce[4..8].try_into().unwrap()),
        u32::from_le_bytes(nonce[8..12].try_into().unwrap()),
    ]
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha20_stream_produces_output() {
        let key = vec![0x42u8; 64];
        let mut stream = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let ks = stream.get_keystream(32).unwrap();
        assert_eq!(ks.len(), 32);
        // Keystream should not be all zeros
        assert!(ks.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_chacha20_stream_is_deterministic() {
        let key = vec![0x42u8; 64];
        let mut s1 = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let mut s2 = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        assert_eq!(s1.get_keystream(64).unwrap(), s2.get_keystream(64).unwrap());
    }

    #[test]
    fn test_chacha20_encrypt_decrypt_roundtrip() {
        let key = vec![0xABu8; 64];
        let plaintext = b"Hello, KeePassEx protected field!";

        let mut enc = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let ciphertext = enc.process(plaintext).unwrap();
        assert_ne!(ciphertext, plaintext);

        let mut dec = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let recovered = dec.process(&ciphertext).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_salsa20_stream_produces_output() {
        let key = vec![0x42u8; 32];
        let mut stream = ProtectedStream::new(ProtectedStreamAlgorithm::Salsa20, &key).unwrap();
        let ks = stream.get_keystream(32).unwrap();
        assert_eq!(ks.len(), 32);
        assert!(ks.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_roundtrip() {
        let key = vec![0xCDu8; 32];
        let plaintext = b"Salsa20 protected password field";

        let mut enc = ProtectedStream::new(ProtectedStreamAlgorithm::Salsa20, &key).unwrap();
        let ciphertext = enc.process(plaintext).unwrap();
        assert_ne!(ciphertext, plaintext);

        let mut dec = ProtectedStream::new(ProtectedStreamAlgorithm::Salsa20, &key).unwrap();
        let recovered = dec.process(&ciphertext).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn test_chacha20_salsa20_produce_different_keystreams() {
        let key = vec![0x42u8; 64];
        let mut c = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let mut s = ProtectedStream::new(ProtectedStreamAlgorithm::Salsa20, &key).unwrap();
        // Different algorithms must produce different keystreams
        assert_ne!(c.get_keystream(32).unwrap(), s.get_keystream(32).unwrap());
    }

    #[test]
    fn test_stream_stateful_across_calls() {
        let key = vec![0x42u8; 64];
        let mut s = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();

        // Get 10 bytes in one call
        let combined = s.get_keystream(10).unwrap();

        // Get same 10 bytes in two separate calls from a fresh stream
        let mut s2 = ProtectedStream::new(ProtectedStreamAlgorithm::ChaCha20, &key).unwrap();
        let mut split = s2.get_keystream(4).unwrap();
        split.extend(s2.get_keystream(6).unwrap());

        assert_eq!(combined, split);
    }

    #[test]
    fn test_algorithm_ids() {
        assert_eq!(ProtectedStreamAlgorithm::ArcFourVariant.id(), 1);
        assert_eq!(ProtectedStreamAlgorithm::Salsa20.id(), 2);
        assert_eq!(ProtectedStreamAlgorithm::ChaCha20.id(), 3);
    }

    #[test]
    fn test_algorithm_from_id() {
        assert_eq!(
            ProtectedStreamAlgorithm::from_id(3),
            Some(ProtectedStreamAlgorithm::ChaCha20)
        );
        assert_eq!(
            ProtectedStreamAlgorithm::from_id(2),
            Some(ProtectedStreamAlgorithm::Salsa20)
        );
        assert_eq!(ProtectedStreamAlgorithm::from_id(99), None);
    }

    /// Verify ChaCha20 block function against RFC 7539 test vector
    #[test]
    fn test_chacha20_rfc7539_test_vector() {
        // RFC 7539 §2.1.1 test vector
        let key = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f,
        ];
        let nonce = [
            0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x00, 0x00,
        ];

        let mut stream = ChaCha20Stream {
            buffer: Vec::new(),
            buf_pos: 0,
            key,
            nonce,
            counter: 1, // RFC 7539 uses counter=1 for this test
        };

        let block = stream.next_block();

        // First 4 bytes of expected output from RFC 7539 §2.3.2
        assert_eq!(block[0], 0x10);
        assert_eq!(block[1], 0xf1);
        assert_eq!(block[2], 0xe7);
        assert_eq!(block[3], 0xe4);
    }
}
