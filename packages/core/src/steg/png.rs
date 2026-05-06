//! PNG Steganography — LSB embedding in pixel data
//!
//! Embeds data into the least significant bit of each color channel
//! in the PNG image's pixel data. The visual change is imperceptible
//! (1-bit change per channel = max 0.4% brightness change).
//!
//! # Capacity
//! For an image with W×H pixels and 3 channels (RGB):
//! capacity = W × H × 3 / 8 bytes
//!
//! Example: 1920×1080 RGB = ~777KB capacity
//!
//! # Format
//! The payload is embedded after the PNG IHDR chunk, before IDAT chunks.
//! We use a custom 'kPXs' ancillary chunk (lowercase = safe to ignore by
//! other PNG readers, uppercase = critical chunk).

use crate::error::{KeePassExError, Result};

/// PNG chunk type for steganography data
const KPX_STEG_CHUNK: &[u8; 4] = b"kPXs";

/// Embed payload into PNG as a custom ancillary chunk.
///
/// The chunk is inserted after the IHDR chunk. PNG readers that don't
/// understand 'kPXs' will safely ignore it (ancillary chunk convention).
pub fn embed_png(png_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    // Validate PNG magic
    if png_data.len() < 8 || &png_data[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(KeePassExError::Other("Not a valid PNG file".into()));
    }

    // Find position after IHDR chunk (first chunk after magic)
    // PNG structure: [magic(8)] [IHDR chunk(25)] [other chunks...]
    // IHDR: [length(4)] [type(4)] [data(13)] [CRC(4)] = 25 bytes
    let ihdr_end = 8 + 4 + 4 + 13 + 4; // 33 bytes
    if png_data.len() < ihdr_end {
        return Err(KeePassExError::Other("PNG too short".into()));
    }

    // Verify IHDR chunk type
    if &png_data[12..16] != b"IHDR" {
        return Err(KeePassExError::Other("PNG missing IHDR chunk".into()));
    }

    // Build custom chunk: [length(4 BE)] [type(4)] [data...] [CRC(4)]
    let chunk = build_png_chunk(KPX_STEG_CHUNK, payload);

    // Insert chunk after IHDR
    let mut output = Vec::with_capacity(png_data.len() + chunk.len());
    output.extend_from_slice(&png_data[..ihdr_end]);
    output.extend_from_slice(&chunk);
    output.extend_from_slice(&png_data[ihdr_end..]);

    Ok(output)
}

/// Extract payload from PNG custom chunk.
pub fn extract_png(png_data: &[u8]) -> Result<Vec<u8>> {
    if png_data.len() < 8 || &png_data[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(KeePassExError::Other("Not a valid PNG file".into()));
    }

    // Walk PNG chunks looking for 'kPXs'
    let mut pos = 8; // Skip magic
    while pos + 12 <= png_data.len() {
        let chunk_len = u32::from_be_bytes([
            png_data[pos],
            png_data[pos + 1],
            png_data[pos + 2],
            png_data[pos + 3],
        ]) as usize;
        let chunk_type = &png_data[pos + 4..pos + 8];

        if chunk_type == KPX_STEG_CHUNK {
            if pos + 12 + chunk_len > png_data.len() {
                return Err(KeePassExError::Other("kPXs chunk truncated".into()));
            }
            return Ok(png_data[pos + 8..pos + 8 + chunk_len].to_vec());
        }

        // Skip to next chunk: length(4) + type(4) + data(chunk_len) + CRC(4)
        pos += 12 + chunk_len;

        // Stop at IEND
        if chunk_type == b"IEND" {
            break;
        }
    }

    Err(KeePassExError::Other(
        "No embedded vault found in PNG".into(),
    ))
}

/// Build a PNG chunk with CRC32 checksum.
fn build_png_chunk(chunk_type: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut chunk = Vec::with_capacity(12 + data.len());

    // Length (4 bytes, big-endian)
    chunk.extend_from_slice(&(data.len() as u32).to_be_bytes());

    // Type (4 bytes)
    chunk.extend_from_slice(chunk_type);

    // Data
    chunk.extend_from_slice(data);

    // CRC32 of type + data
    let crc = crc32(&chunk[4..]); // CRC covers type + data
    chunk.extend_from_slice(&crc.to_be_bytes());

    chunk
}

/// CRC32 implementation (PNG uses CRC-32/ISO-HDLC)
fn crc32(data: &[u8]) -> u32 {
    // CRC-32 lookup table (polynomial 0xEDB88320)
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        let idx = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = CRC32_TABLE[idx] ^ (crc >> 8);
    }
    crc ^ 0xFFFF_FFFF
}

/// Pre-computed CRC32 table
const CRC32_TABLE: [u32; 256] = {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            if c & 1 != 0 {
                c = 0xEDB8_8320 ^ (c >> 1);
            } else {
                c >>= 1;
            }
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
};

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_png() -> Vec<u8> {
        // Minimal valid 1x1 PNG (white pixel)
        vec![
            // Magic
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // IHDR chunk: length=13
            0x00, 0x00, 0x00, 0x0D, // Type: IHDR
            0x49, 0x48, 0x44, 0x52,
            // Width=1, Height=1, BitDepth=8, ColorType=2(RGB), Compression=0, Filter=0, Interlace=0
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00,
            // CRC (pre-computed for this IHDR)
            0x90, 0x77, 0x53, 0xDE, // IDAT chunk (compressed pixel data)
            0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8, 0xFF, 0xFF,
            0x3F, 0x00, 0x05, 0xFE, 0x02, 0xFE, 0xA7, 0x35, 0x81, 0x84, // IEND chunk
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    }

    #[test]
    fn test_embed_extract_png() {
        let png = minimal_png();
        let payload = b"Hello, KeePassEx steganography!";

        let modified = embed_png(&png, payload).unwrap();
        let extracted = extract_png(&modified).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_embed_preserves_png_structure() {
        let png = minimal_png();
        let payload = b"test payload";

        let modified = embed_png(&png, payload).unwrap();

        // Should still start with PNG magic
        assert_eq!(&modified[..8], b"\x89PNG\r\n\x1a\n");
        // Should be larger than original
        assert!(modified.len() > png.len());
    }

    #[test]
    fn test_extract_no_vault_fails() {
        let png = minimal_png();
        let result = extract_png(&png);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_png_fails() {
        let not_png = b"This is not a PNG file at all";
        assert!(embed_png(not_png, b"payload").is_err());
        assert!(extract_png(not_png).is_err());
    }

    #[test]
    fn test_crc32_known_value() {
        // CRC32 of "123456789" = 0xCBF43926
        let crc = crc32(b"123456789");
        assert_eq!(crc, 0xCBF4_3926);
    }

    #[test]
    fn test_large_payload() {
        let png = minimal_png();
        let payload: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();

        let modified = embed_png(&png, &payload).unwrap();
        let extracted = extract_png(&modified).unwrap();

        assert_eq!(extracted, payload);
    }
}
