//! Cryptographic primitives for KeePassEx
//!
//! Supports:
//! - Argon2id (default KDF)
//! - AES-KDF (KDBX 3.1 compat)
//! - ChaCha20-Poly1305 (default cipher)
//! - AES-256-GCM (alternative cipher)
//! - HMAC-SHA256 (integrity)
//! - Salsa20 (inner stream, KDBX 3.1 compat)

pub mod cipher;
pub mod hmac;
pub mod kdf;
pub mod keys;
pub mod pqc;
pub mod protected_stream;
pub mod shamir;

pub use cipher::{Cipher, CipherAlgorithm};
pub use kdf::{AesKdfParams, ArgonParams, Kdf, KdfParams};
pub use keys::{CompositeKey, KeyFile, MasterKey};
pub use pqc::{
    decapsulate, derive_pqc_keypair, encapsulate, PqcAlgorithm, PqcEncapsulation, PqcKeyPair,
};
pub use shamir::{combine_shards, split_secret, SecretShard};
