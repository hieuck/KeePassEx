//! Password generator tests

use crate::generator::{PasswordGenerator, PasswordStrength};
use crate::types::{PasswordGeneratorConfig, GeneratorMode, WordList};

#[test]
fn test_generate_random_password_length() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 20,
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: true,
        ..Default::default()
    };

    let password = PasswordGenerator::generate(&config).unwrap();
    assert_eq!(password.len(), 20);
}

#[test]
fn test_generate_respects_charset() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 100,
        use_uppercase: true,
        use_lowercase: false,
        use_digits: false,
        use_symbols: false,
        ..Default::default()
    };

    let password = PasswordGenerator::generate(&config).unwrap();
    assert!(password.chars().all(|c| c.is_uppercase()));
}

#[test]
fn test_generate_no_charset_fails() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 20,
        use_uppercase: false,
        use_lowercase: false,
        use_digits: false,
        use_symbols: false,
        ..Default::default()
    };

    let result = PasswordGenerator::generate(&config);
    assert!(result.is_err());
}

#[test]
fn test_generate_passphrase() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Passphrase,
        word_count: 4,
        word_separator: "-".to_string(),
        ..Default::default()
    };

    let passphrase = PasswordGenerator::generate(&config).unwrap();
    let words: Vec<&str> = passphrase.split('-').collect();
    assert_eq!(words.len(), 4);
    assert!(words.iter().all(|w| !w.is_empty()));
}

#[test]
fn test_generate_pronounceable() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Pronounceable,
        length: 12,
        ..Default::default()
    };

    let password = PasswordGenerator::generate(&config).unwrap();
    assert_eq!(password.len(), 12);
}

#[test]
fn test_entropy_calculation() {
    // All lowercase: 26^8 ≈ 38 bits
    let entropy = PasswordGenerator::estimate_entropy("abcdefgh");
    assert!(entropy > 30.0 && entropy < 50.0);

    // Mixed: higher entropy
    let entropy_mixed = PasswordGenerator::estimate_entropy("Abc123!@");
    assert!(entropy_mixed > entropy);
}

#[test]
fn test_strength_scoring() {
    assert_eq!(PasswordGenerator::score_strength("abc").score(), 0); // Very weak
    assert_eq!(PasswordGenerator::score_strength("password123").score(), 1); // Weak
    assert_eq!(PasswordGenerator::score_strength("P@ssw0rd123").score(), 2); // Fair
    assert_eq!(PasswordGenerator::score_strength("Xk9#mP2$vL7@nQ4!").score(), 3); // Strong
    assert_eq!(PasswordGenerator::score_strength("Xk9#mP2$vL7@nQ4!zR8^wS5%").score(), 4); // Very strong
}

#[test]
fn test_strength_labels_en() {
    assert_eq!(PasswordStrength::VeryWeak.label_en(), "Very Weak");
    assert_eq!(PasswordStrength::VeryStrong.label_en(), "Very Strong");
}

#[test]
fn test_strength_labels_vi() {
    assert_eq!(PasswordStrength::VeryWeak.label_vi(), "Rất yếu");
    assert_eq!(PasswordStrength::VeryStrong.label_vi(), "Rất mạnh");
}

#[test]
fn test_exclude_ambiguous() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 200,
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: false,
        exclude_ambiguous: true,
        ..Default::default()
    };

    let password = PasswordGenerator::generate(&config).unwrap();
    assert!(!password.contains('0'));
    assert!(!password.contains('O'));
    assert!(!password.contains('1'));
    assert!(!password.contains('l'));
    assert!(!password.contains('I'));
}

#[test]
fn test_minimum_character_requirements() {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 20,
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: true,
        min_uppercase: 2,
        min_lowercase: 2,
        min_digits: 2,
        min_symbols: 2,
        ..Default::default()
    };

    for _ in 0..10 {
        let password = PasswordGenerator::generate(&config).unwrap();
        let uppercase_count = password.chars().filter(|c| c.is_uppercase()).count();
        let lowercase_count = password.chars().filter(|c| c.is_lowercase()).count();
        let digit_count = password.chars().filter(|c| c.is_ascii_digit()).count();

        assert!(uppercase_count >= 2, "Not enough uppercase: {}", password);
        assert!(lowercase_count >= 2, "Not enough lowercase: {}", password);
        assert!(digit_count >= 2, "Not enough digits: {}", password);
    }
}

#[test]
fn test_passwords_are_random() {
    let config = PasswordGeneratorConfig::default();
    let p1 = PasswordGenerator::generate(&config).unwrap();
    let p2 = PasswordGenerator::generate(&config).unwrap();
    // Extremely unlikely to be equal
    assert_ne!(p1, p2);
}
