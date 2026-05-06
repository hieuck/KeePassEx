//! Breach monitor tests

use crate::breach::{sha1_hex, check_password_offline, parse_hibp_response};

#[test]
fn test_sha1_hex_known_value() {
    // SHA-1 of "password" = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
    let hash = sha1_hex("password");
    assert_eq!(hash, "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8");
}

#[test]
fn test_sha1_hex_empty_string() {
    // SHA-1 of "" = DA39A3EE5E6B4B0D3255BFEF95601890AFD80709
    let hash = sha1_hex("");
    assert_eq!(hash, "DA39A3EE5E6B4B0D3255BFEF95601890AFD80709");
}

#[test]
fn test_sha1_hex_123456() {
    let hash = sha1_hex("123456");
    assert_eq!(hash, "7C4A8D09CA3762AF61E59520943DC26494F8941B");
}

#[test]
fn test_offline_check_common_passwords() {
    assert!(check_password_offline("password"));
    assert!(check_password_offline("123456"));
    assert!(check_password_offline("qwerty"));
}

#[test]
fn test_offline_check_strong_password() {
    assert!(!check_password_offline("Xk9#mP2$vL7@nQ4!zR8^wS5%"));
    assert!(!check_password_offline("correct-horse-battery-staple-42"));
}

#[test]
fn test_hibp_prefix_extraction() {
    let hash = sha1_hex("password");
    let prefix = &hash[..5];
    let suffix = &hash[5..];

    assert_eq!(prefix, "5BAA6");
    assert_eq!(suffix.len(), 35);
}
