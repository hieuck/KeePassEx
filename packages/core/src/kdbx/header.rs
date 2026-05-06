//! KDBX header structures and parsing helpers

use crate::kdbx::{HeaderFieldId, InnerHeaderFieldId, Compression};
use crate::crypto::cipher::CipherAlgorithm;
use crate::crypto::kdf::KdfParams;

/// Parsed outer header of a KDBX 4.x file
#[derive(Debug)]
pub struct OuterHeader {
    pub cipher_algorithm: CipherAlgorithm,
    pub compression: Compression,
    pub master_seed: Vec<u8>,
    pub encryption_iv: Vec<u8>,
    pub kdf_params: KdfParams,
    pub public_custom_data: Vec<(String, Vec<u8>)>,
}

/// Parsed inner header of a KDBX 4.x file
#[derive(Debug)]
pub struct InnerHeader {
    pub inner_stream_algorithm: u32,
    pub inner_stream_key: Vec<u8>,
    pub binaries: Vec<BinaryEntry>,
}

/// A binary attachment stored in the inner header
#[derive(Debug, Clone)]
pub struct BinaryEntry {
    pub flags: u8,
    pub data: Vec<u8>,
}

impl BinaryEntry {
    /// Whether this binary is protected (XOR'd with inner stream)
    pub fn is_protected(&self) -> bool {
        self.flags & 0x01 != 0
    }
}

/// KDBX file version info
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KdbxVersion {
    pub major: u16,
    pub minor: u16,
}

impl KdbxVersion {
    pub fn new(raw: u32) -> Self {
        Self {
            minor: (raw & 0xFFFF) as u16,
            major: ((raw >> 16) & 0xFFFF) as u16,
        }
    }

    pub fn is_kdbx4(&self) -> bool {
        self.major >= 4
    }

    pub fn is_kdbx3(&self) -> bool {
        self.major == 3
    }
}

impl std::fmt::Display for KdbxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}
