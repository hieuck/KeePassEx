//! KDBX 4.x file format reader/writer
//! Also supports reading KDBX 3.1 and KDB (legacy)

pub mod reader;
pub mod writer;
pub mod header;
pub mod xml;

pub use reader::KdbxReader;
pub use writer::KdbxWriter;

/// KDBX file signature
pub const KDBX_SIGNATURE_1: u32 = 0x9AA2D903;
pub const KDBX_SIGNATURE_2: u32 = 0xB54BFB67;

/// KDBX version constants
pub const KDBX_VERSION_3_1: u32 = 0x00030001;
pub const KDBX_VERSION_4_0: u32 = 0x00040000;
pub const KDBX_VERSION_4_1: u32 = 0x00040001;

/// Header field IDs (KDBX 4.x)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeaderFieldId {
    EndOfHeader = 0,
    Comment = 1,
    CipherId = 2,
    CompressionFlags = 3,
    MasterSeed = 4,
    EncryptionIv = 7,
    KdfParameters = 11,
    PublicCustomData = 12,
}

/// Inner header field IDs (KDBX 4.x)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InnerHeaderFieldId {
    EndOfHeader = 0,
    InnerRandomStreamId = 1,
    InnerRandomStreamKey = 2,
    Binary = 3,
}

/// Compression algorithm
#[derive(Debug, Clone, PartialEq)]
pub enum Compression {
    None,
    GZip,
}

impl Compression {
    pub fn from_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Compression::None),
            1 => Some(Compression::GZip),
            _ => None,
        }
    }

    pub fn id(&self) -> u32 {
        match self {
            Compression::None => 0,
            Compression::GZip => 1,
        }
    }
}
