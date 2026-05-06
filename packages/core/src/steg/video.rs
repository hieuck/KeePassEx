//! Video Steganography — Embedding in MP4/AVI metadata
//!
//! Embeds vault data into custom metadata atoms/chunks:
//! - MP4: Custom 'kpxv' atom in the 'moov' container
//! - AVI: Custom 'KPX ' chunk in the RIFF header
//!
//! # Capacity
//! Essentially unlimited (metadata can be arbitrarily large)

use crate::error::{KeePassExError, Result};

/// MP4 'kpxv' atom type (KeePassEx Vault)
const MP4_KPX_ATOM: &[u8; 4] = b"kpxv";

/// AVI 'KPX ' chunk ID
const AVI_KPX_CHUNK: &[u8; 4] = b"KPX ";

/// Embed payload into MP4 as a custom atom in the 'moov' container.
pub fn embed_mp4(mp4_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    // Validate MP4 magic (ftyp box)
    if mp4_data.len() < 8 {
        return Err(KeePassExError::Other("MP4 file too short".into()));
    }
    if &mp4_data[4..8] != b"ftyp" {
        return Err(KeePassExError::Other("Not a valid MP4 file".into()));
    }

    // Find 'moov' atom
    let moov_pos = find_mp4_atom(mp4_data, b"moov")
        .ok_or_else(|| KeePassExError::Other("MP4 missing 'moov' atom".into()))?;

    // Build custom 'kpxv' atom: [size(4 BE)] [type(4)] [payload...]
    let atom_size = 8 + payload.len();
    let mut atom = Vec::with_capacity(atom_size);
    atom.extend_from_slice(&(atom_size as u32).to_be_bytes());
    atom.extend_from_slice(MP4_KPX_ATOM);
    atom.extend_from_slice(payload);

    // Insert atom at the end of 'moov' (before next top-level atom)
    let moov_size = u32::from_be_bytes([
        mp4_data[moov_pos],
        mp4_data[moov_pos + 1],
        mp4_data[moov_pos + 2],
        mp4_data[moov_pos + 3],
    ]) as usize;
    let moov_end = moov_pos + moov_size;

    let mut output = Vec::with_capacity(mp4_data.len() + atom.len());
    output.extend_from_slice(&mp4_data[..moov_end]);
    output.extend_from_slice(&atom);
    output.extend_from_slice(&mp4_data[moov_end..]);

    // Update 'moov' size
    let new_moov_size = moov_size + atom.len();
    output[moov_pos..moov_pos + 4].copy_from_slice(&(new_moov_size as u32).to_be_bytes());

    Ok(output)
}

/// Extract payload from MP4 custom atom.
pub fn extract_mp4(mp4_data: &[u8]) -> Result<Vec<u8>> {
    if mp4_data.len() < 8 || &mp4_data[4..8] != b"ftyp" {
        return Err(KeePassExError::Other("Not a valid MP4 file".into()));
    }

    // Find 'kpxv' atom
    let kpxv_pos = find_mp4_atom(mp4_data, MP4_KPX_ATOM)
        .ok_or_else(|| KeePassExError::Other("No embedded vault found in MP4".into()))?;

    let atom_size = u32::from_be_bytes([
        mp4_data[kpxv_pos],
        mp4_data[kpxv_pos + 1],
        mp4_data[kpxv_pos + 2],
        mp4_data[kpxv_pos + 3],
    ]) as usize;

    if kpxv_pos + atom_size > mp4_data.len() {
        return Err(KeePassExError::Other("MP4 kpxv atom truncated".into()));
    }

    // Payload starts after size(4) + type(4)
    Ok(mp4_data[kpxv_pos + 8..kpxv_pos + atom_size].to_vec())
}

/// Find an MP4 atom by type — searches top-level AND inside 'moov'
fn find_mp4_atom(data: &[u8], atom_type: &[u8; 4]) -> Option<usize> {
    // First pass: top-level atoms
    let mut pos = 0;
    while pos + 8 <= data.len() {
        let size =
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        let atype = &data[pos + 4..pos + 8];

        if atype == atom_type {
            return Some(pos);
        }

        // Search inside 'moov' container
        if atype == b"moov" && size >= 8 {
            let moov_end = (pos + size).min(data.len());
            let mut inner_pos = pos + 8; // skip moov header
            while inner_pos + 8 <= moov_end {
                let inner_size = u32::from_be_bytes([
                    data[inner_pos],
                    data[inner_pos + 1],
                    data[inner_pos + 2],
                    data[inner_pos + 3],
                ]) as usize;
                let inner_type = &data[inner_pos + 4..inner_pos + 8];
                if inner_type == atom_type {
                    return Some(inner_pos);
                }
                if inner_size < 8 {
                    break;
                }
                inner_pos += inner_size;
            }
        }

        if size < 8 {
            break;
        }
        pos += size;
    }
    None
}

/// Embed payload into AVI as a custom RIFF chunk.
pub fn embed_avi(avi_data: &[u8], payload: &[u8]) -> Result<Vec<u8>> {
    // Validate AVI magic (RIFF...AVI )
    if avi_data.len() < 12 {
        return Err(KeePassExError::Other("AVI file too short".into()));
    }
    if &avi_data[..4] != b"RIFF" || &avi_data[8..12] != b"AVI " {
        return Err(KeePassExError::Other("Not a valid AVI file".into()));
    }

    // Build custom 'KPX ' chunk: [ID(4)] [size(4 LE)] [payload...]
    let chunk_size = payload.len();
    let mut chunk = Vec::with_capacity(8 + chunk_size);
    chunk.extend_from_slice(AVI_KPX_CHUNK);
    chunk.extend_from_slice(&(chunk_size as u32).to_le_bytes());
    chunk.extend_from_slice(payload);

    // Pad to even size (RIFF requirement)
    if chunk_size % 2 != 0 {
        chunk.push(0);
    }

    // Insert chunk after RIFF header (after 'AVI ' type)
    let mut output = Vec::with_capacity(avi_data.len() + chunk.len());
    output.extend_from_slice(&avi_data[..12]);
    output.extend_from_slice(&chunk);
    output.extend_from_slice(&avi_data[12..]);

    // Update RIFF size
    let riff_size = output.len() - 8;
    output[4..8].copy_from_slice(&(riff_size as u32).to_le_bytes());

    Ok(output)
}

/// Extract payload from AVI custom chunk.
pub fn extract_avi(avi_data: &[u8]) -> Result<Vec<u8>> {
    if avi_data.len() < 12 || &avi_data[..4] != b"RIFF" || &avi_data[8..12] != b"AVI " {
        return Err(KeePassExError::Other("Not a valid AVI file".into()));
    }

    // Find 'KPX ' chunk
    let mut pos = 12; // Skip RIFF header
    while pos + 8 <= avi_data.len() {
        let chunk_id = &avi_data[pos..pos + 4];
        let chunk_size = u32::from_le_bytes([
            avi_data[pos + 4],
            avi_data[pos + 5],
            avi_data[pos + 6],
            avi_data[pos + 7],
        ]) as usize;

        if chunk_id == AVI_KPX_CHUNK {
            if pos + 8 + chunk_size > avi_data.len() {
                return Err(KeePassExError::Other("AVI KPX chunk truncated".into()));
            }
            return Ok(avi_data[pos + 8..pos + 8 + chunk_size].to_vec());
        }

        // Skip to next chunk (size + padding)
        let padded_size = if chunk_size % 2 == 0 {
            chunk_size
        } else {
            chunk_size + 1
        };
        pos += 8 + padded_size;
    }

    Err(KeePassExError::Other(
        "No embedded vault found in AVI".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_mp4() -> Vec<u8> {
        // Minimal valid MP4 structure
        vec![
            // ftyp atom
            0x00, 0x00, 0x00, 0x18, // size = 24
            b'f', b't', b'y', b'p', // type
            b'i', b's', b'o', b'm', // major brand
            0x00, 0x00, 0x02, 0x00, // minor version
            b'i', b's', b'o', b'm', // compatible brand
            b'm', b'p', b'4', b'2', // compatible brand
            // moov atom (empty)
            0x00, 0x00, 0x00, 0x08, // size = 8
            b'm', b'o', b'o', b'v', // type
        ]
    }

    fn minimal_avi() -> Vec<u8> {
        // Minimal valid AVI structure
        vec![
            // RIFF header
            b'R', b'I', b'F', b'F', // ID
            0x20, 0x00, 0x00, 0x00, // size (32 bytes)
            b'A', b'V', b'I', b' ', // type
            // hdrl LIST (header)
            b'L', b'I', b'S', b'T', // ID
            0x14, 0x00, 0x00, 0x00, // size (20 bytes)
            b'h', b'd', b'r', b'l', // type
            // avih chunk (AVI header)
            b'a', b'v', b'i', b'h', // ID
            0x08, 0x00, 0x00, 0x00, // size (8 bytes)
            0x00, 0x00, 0x00, 0x00, // dummy data
            0x00, 0x00, 0x00, 0x00,
        ]
    }

    #[test]
    fn test_embed_extract_mp4() {
        let mp4 = minimal_mp4();
        let payload = b"KeePassEx vault in MP4 metadata";

        let modified = embed_mp4(&mp4, payload).unwrap();
        let extracted = extract_mp4(&modified).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_embed_extract_avi() {
        let avi = minimal_avi();
        let payload = b"KeePassEx vault in AVI chunk";

        let modified = embed_avi(&avi, payload).unwrap();
        let extracted = extract_avi(&modified).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_mp4_no_vault_fails() {
        let mp4 = minimal_mp4();
        let result = extract_mp4(&mp4);
        assert!(result.is_err());
    }

    #[test]
    fn test_avi_no_vault_fails() {
        let avi = minimal_avi();
        let result = extract_avi(&avi);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mp4_fails() {
        let not_mp4 = b"Not an MP4 file";
        assert!(embed_mp4(not_mp4, b"payload").is_err());
        assert!(extract_mp4(not_mp4).is_err());
    }

    #[test]
    fn test_invalid_avi_fails() {
        let not_avi = b"Not an AVI file";
        assert!(embed_avi(not_avi, b"payload").is_err());
        assert!(extract_avi(not_avi).is_err());
    }

    #[test]
    fn test_large_payload_mp4() {
        let mp4 = minimal_mp4();
        let payload = vec![0xABu8; 10_000];

        let modified = embed_mp4(&mp4, &payload).unwrap();
        let extracted = extract_mp4(&modified).unwrap();

        assert_eq!(extracted, payload);
    }

    #[test]
    fn test_large_payload_avi() {
        let avi = minimal_avi();
        let payload = vec![0xCDu8; 10_000];

        let modified = embed_avi(&avi, &payload).unwrap();
        let extracted = extract_avi(&modified).unwrap();

        assert_eq!(extracted, payload);
    }
}
