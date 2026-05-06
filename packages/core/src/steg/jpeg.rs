//! JPEG Steganography — Embedding in EXIF/APP1 metadata
//!
//! Embeds vault data into a custom EXIF APP1 segment. JPEG compression
//! is lossy for pixel data, so we use metadata instead (lossless).
//!
//! # Capacity
//! EXIF APP1 segment max size: ~64KB (JPEG spec limit)
//!
//! # Format
//! We create a custom APP1 marker (0xFFE1) with identifier "KPX\0".
//! Standard JPEG readers will ignore unknown APP1 segments.

use crate::error::{KeePassExError, Result};

/// JPEG SOI (Start of Image) marker
const JPEG_SOI: [u8; 2] = [0xFF, 0xD8];
/// JPEG APP1 marker
const JPEG_APP1: u8 = 0xE1;
/// Custom APP1 identifier for KeePassEx
const KPX_APP1_ID: &[u8; 4] = b"KPX\0";

/// Embed payload into JPEG as a custom APP1 segment.
///
/// The segment is inserted immediately after the SOI marker.
pub fn embed_jpeg(jpeg_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    // Validate JPEG magic
    if jpeg_data.len() < 2 || &jpeg_data[..2] != JPEG_SOI {
        return Err(KeePassExError::Other("Not a valid JPEG file".into()));
    }

    // Check payload size (APP1 segment max = 65535 - 2 (marker) - 2 (length) - 4 (ID) = 65527)
    if payload.len() > 65_527 {
        return Err(KeePassExError::Other(
            "Payload too large for JPEG (max 64KB)".into(),
        ));
    }

    // Build APP1 segment: [0xFF 0xE1] [length(2 BE)] [ID(4)] [payload...]
    let segment_len = 2 + 4 + payload.len(); // length field + ID + payload
    let mut segment = Vec::with_capacity(2 + 2 + 4 + payload.len());
    segment.push(0xFF);
    segment.push(JPEG_APP1);
    segment.extend_from_slice(&(segment_len as u16).to_be_bytes());
    segment.extend_from_slice(KPX_APP1_ID);
    segment.extend_from_slice(payload);

    // Insert segment after SOI
    let mut output = Vec::with_capacity(jpeg_data.len() + segment.len());
    output.extend_from_slice(&JPEG_SOI);
    output.extend_from_slice(&segment);
    output.extend_from_slice(&jpeg_data[2..]); // Rest of JPEG

    Ok(output)
}

/// Extract payload from JPEG custom APP1 segment.
pub fn extract_jpeg(jpeg_data: &[u8]) -> Result<Vec<u8>> {
    if jpeg_data.len() < 2 || &jpeg_data[..2] != JPEG_SOI {
        return Err(KeePassExError::Other("Not a valid JPEG file".into()));
    }

    // Walk JPEG markers looking for our custom APP1
    let mut pos = 2; // Skip SOI
    while pos + 4 <= jpeg_data.len() {
        if jpeg_data[pos] != 0xFF {
            return Err(KeePassExError::Other("Invalid JPEG marker".into()));
        }

        let marker = jpeg_data[pos + 1];
        pos += 2;

        // Markers without length field (standalone)
        if marker == 0xD8 || marker == 0xD9 || (marker >= 0xD0 && marker <= 0xD7) {
            continue;
        }

        // Read segment length
        if pos + 2 > jpeg_data.len() {
            return Err(KeePassExError::Other("JPEG segment truncated".into()));
        }
        let seg_len = u16::from_be_bytes([jpeg_data[pos], jpeg_data[pos + 1]]) as usize;
        if seg_len < 2 {
            return Err(KeePassExError::Other("Invalid JPEG segment length".into()));
        }

        // Check if this is our custom APP1
        if marker == JPEG_APP1 && pos + seg_len <= jpeg_data.len() {
            let seg_data = &jpeg_data[pos + 2..pos + seg_len];
            if seg_data.len() >= 4 && &seg_data[..4] == KPX_APP1_ID {
                return Ok(seg_data[4..].to_vec());
            }
        }

        // Skip to next marker
        pos += seg_len;

        // Stop at SOS (Start of Scan) — image data follows
        if marker == 0xDA {
            break;
        }
    }

    Err(KeePassExError::Other(
        "No embedded vault found in JPEG".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_jpeg() -> Vec<u8> {
        // Minimal valid JPEG (1x1 white pixel)
        vec![
            // SOI
            0xFF, 0xD8, // APP0 (JFIF)
            0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01,
            0x00, 0x01, 0x00, 0x00, // SOF0 (Start of Frame)
            0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
            // DHT (Define Huffman Table)
            0xFF, 0xC4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x08, // SOS (Start of Scan)
            0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00, 0x3F, 0x00, 0xD2, 0xCF, 0x20,
            // EOI
            0xFF, 0xD9,
        ]
    }

    #[test]
    fn test_embed_extract_jpeg() {
        let jpeg = minimal_jpeg();
        let payload = b"KeePassEx vault in JPEG metadata";

        let modified = embed_jpeg(&jpeg, payload).unwrap();
        let extracted = extract_jpeg(&modified).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_embed_preserves_jpeg_structure() {
        let jpeg = minimal_jpeg();
        let payload = b"test payload";

        let modified = embed_jpeg(&jpeg, payload).unwrap();

        // Should still start with JPEG SOI
        assert_eq!(&modified[..2], &JPEG_SOI);
        // Should be larger than original
        assert!(modified.len() > jpeg.len());
    }

    #[test]
    fn test_extract_no_vault_fails() {
        let jpeg = minimal_jpeg();
        let result = extract_jpeg(&jpeg);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_jpeg_fails() {
        let not_jpeg = b"This is not a JPEG file";
        assert!(embed_jpeg(not_jpeg, b"payload").is_err());
        assert!(extract_jpeg(not_jpeg).is_err());
    }

    #[test]
    fn test_payload_too_large_fails() {
        let jpeg = minimal_jpeg();
        let large_payload = vec![0u8; 70_000]; // > 64KB
        let result = embed_jpeg(&jpeg, &large_payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_size_payload() {
        let jpeg = minimal_jpeg();
        let payload = vec![0xABu8; 65_527]; // Max size

        let modified = embed_jpeg(&jpeg, &payload).unwrap();
        let extracted = extract_jpeg(&modified).unwrap();

        assert_eq!(extracted, payload);
    }
}
