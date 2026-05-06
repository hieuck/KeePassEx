//! Steganography tests — uses the public steg::embed() and steg::extract() APIs

use crate::steg::{embed, extract, has_embedded_vault, max_capacity, StegFormat};

/// Minimal valid 1×1 PNG bytes
fn minimal_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR length + type
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1 dimensions
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, // bit depth, color type
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, // IDAT
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21,
        0xBC, 0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, // IEND
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ]
}

#[test]
fn test_steg_format_detect_png() {
    let png_magic = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    assert_eq!(StegFormat::detect(&png_magic), Some(StegFormat::Png));
}

#[test]
fn test_steg_format_detect_jpeg() {
    let jpeg_magic = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10];
    assert_eq!(StegFormat::detect(&jpeg_magic), Some(StegFormat::Jpeg));
}

#[test]
fn test_steg_format_detect_mp4() {
    let mp4_magic = [0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70];
    assert_eq!(StegFormat::detect(&mp4_magic), Some(StegFormat::Mp4));
}

#[test]
fn test_steg_format_detect_unknown() {
    let unknown = [0x00, 0x01, 0x02, 0x03];
    assert_eq!(StegFormat::detect(&unknown), None);
}

#[test]
fn test_steg_format_detect_too_short() {
    let short = [0x89, 0x50];
    assert_eq!(StegFormat::detect(&short), None);
}

#[test]
fn test_extract_from_clean_png_fails() {
    let carrier = minimal_png();
    let result = extract(&carrier, "test_password");
    assert!(result.is_err(), "Extracting from clean PNG should fail");
}

#[test]
fn test_has_embedded_vault_clean_image_returns_false() {
    let carrier = minimal_png();
    assert!(!has_embedded_vault(&carrier));
}

#[test]
fn test_max_capacity_png() {
    let carrier = minimal_png();
    let cap = max_capacity(&carrier);
    assert!(cap.is_some());
    // PNG capacity is carrier.len() / 8
    assert_eq!(cap.unwrap(), carrier.len() / 8);
}

#[test]
fn test_max_capacity_jpeg() {
    let jpeg_magic = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
    let cap = max_capacity(&jpeg_magic);
    assert!(cap.is_some());
    assert_eq!(cap.unwrap(), 65_536);
}

#[test]
fn test_max_capacity_unknown_returns_none() {
    let unknown = vec![0x00, 0x01, 0x02, 0x03];
    assert!(max_capacity(&unknown).is_none());
}

#[test]
fn test_extract_from_unsupported_format_fails() {
    let unknown = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let result = extract(&unknown, "password");
    assert!(result.is_err());
}

#[test]
fn test_embed_into_unsupported_format_fails() {
    let unknown = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
    let result = embed(&unknown, b"vault_data", "password");
    assert!(result.is_err());
}
