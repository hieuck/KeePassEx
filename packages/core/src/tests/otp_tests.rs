//! OTP tests

use crate::otp::{parse_otp_uri, to_otp_uri, generate_hotp};
use crate::types::{OtpConfig, OtpAlgorithm, OtpType, ProtectedString};

fn make_totp_config(secret: &str) -> OtpConfig {
    OtpConfig {
        secret: ProtectedString::new(secret),
        algorithm: OtpAlgorithm::Sha1,
        digits: 6,
        period: 30,
        counter: 0,
        otp_type: OtpType::Totp,
        issuer: Some("Test".to_string()),
        account: Some("user@example.com".to_string()),
    }
}

#[test]
fn test_hotp_rfc4226_test_vectors() {
    // RFC 4226 test vectors with secret "12345678901234567890"
    let config = OtpConfig {
        secret: ProtectedString::new("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"), // base32 of "12345678901234567890"
        algorithm: OtpAlgorithm::Sha1,
        digits: 6,
        period: 30,
        counter: 0,
        otp_type: OtpType::Hotp,
        issuer: None,
        account: None,
    };

    // RFC 4226 expected values for counter 0-9
    let expected = ["755224", "287082", "359152", "969429", "338314",
                    "254676", "287922", "162583", "399871", "520489"];

    for (i, expected_code) in expected.iter().enumerate() {
        let code = generate_hotp(&config, i as u64).unwrap();
        assert_eq!(&code, expected_code, "Counter {}: expected {}, got {}", i, expected_code, code);
    }
}

#[test]
fn test_parse_totp_uri() {
    let uri = "otpauth://totp/Example%3Auser%40example.com?secret=JBSWY3DPEHPK3PXP&issuer=Example&algorithm=SHA1&digits=6&period=30";
    let config = parse_otp_uri(uri).unwrap();

    assert_eq!(config.otp_type, OtpType::Totp);
    assert_eq!(config.secret.get(), "JBSWY3DPEHPK3PXP");
    assert_eq!(config.algorithm, OtpAlgorithm::Sha1);
    assert_eq!(config.digits, 6);
    assert_eq!(config.period, 30);
    assert_eq!(config.issuer.as_deref(), Some("Example"));
}

#[test]
fn test_parse_hotp_uri() {
    let uri = "otpauth://hotp/Example?secret=JBSWY3DPEHPK3PXP&counter=5";
    let config = parse_otp_uri(uri).unwrap();

    assert_eq!(config.otp_type, OtpType::Hotp);
    assert_eq!(config.counter, 5);
}

#[test]
fn test_parse_invalid_uri() {
    let result = parse_otp_uri("https://example.com");
    assert!(result.is_err());

    let result = parse_otp_uri("otpauth://totp/test"); // no secret
    assert!(result.is_err());
}

#[test]
fn test_otp_uri_roundtrip() {
    let config = make_totp_config("JBSWY3DPEHPK3PXP");
    let uri = to_otp_uri(&config);
    let parsed = parse_otp_uri(&uri).unwrap();

    assert_eq!(parsed.secret.get(), config.secret.get());
    assert_eq!(parsed.digits, config.digits);
    assert_eq!(parsed.period, config.period);
}

#[test]
fn test_otp_sha256_algorithm() {
    let config = OtpConfig {
        secret: ProtectedString::new("JBSWY3DPEHPK3PXP"),
        algorithm: OtpAlgorithm::Sha256,
        digits: 6,
        period: 30,
        counter: 0,
        otp_type: OtpType::Hotp,
        issuer: None,
        account: None,
    };

    let code = generate_hotp(&config, 0).unwrap();
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn test_otp_8_digits() {
    let config = OtpConfig {
        secret: ProtectedString::new("JBSWY3DPEHPK3PXP"),
        algorithm: OtpAlgorithm::Sha1,
        digits: 8,
        period: 30,
        counter: 0,
        otp_type: OtpType::Hotp,
        issuer: None,
        account: None,
    };

    let code = generate_hotp(&config, 0).unwrap();
    assert_eq!(code.len(), 8);
}
