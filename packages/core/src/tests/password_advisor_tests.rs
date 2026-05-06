//! Password Advisor integration tests
//! Tests the context-aware password strength advisor

use crate::password_advisor::{advise_password, calculate_entropy, AdviseSeverity};

#[test]
fn test_empty_password() {
    let advice = advise_password("", "GitHub", "https://github.com", "development");
    assert_eq!(advice.score, 0);
    assert_eq!(advice.recommendations[0].code, "empty");
    assert_eq!(advice.recommendations[0].severity, AdviseSeverity::Critical);
    assert!(!advice.appropriate_for_category);
}

#[test]
fn test_password_contains_site_name() {
    let advice = advise_password("github123!", "GitHub", "https://github.com", "development");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"contains_site_name"));
}

#[test]
fn test_password_contains_domain() {
    let advice = advise_password("chase2024!", "Chase Bank", "https://chase.com", "banking");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"contains_domain") || codes.contains(&"contains_site_name"));
}

#[test]
fn test_banking_requires_16_chars() {
    let advice = advise_password("MyPass123!", "Chase", "https://chase.com", "banking");
    assert_eq!(advice.min_recommended_length, 16);
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"below_category_minimum"));
}

#[test]
fn test_strong_banking_password() {
    let advice = advise_password(
        "X9#mK2$pL7@nQ4!vR8^wZ",
        "Chase Bank",
        "https://chase.com",
        "banking",
    );
    assert!(advice.score >= 60);
    assert!(advice.appropriate_for_category);
    let critical: Vec<_> = advice
        .recommendations
        .iter()
        .filter(|r| r.severity == AdviseSeverity::Critical)
        .collect();
    assert!(critical.is_empty());
}

#[test]
fn test_keyboard_walk_qwerty() {
    let advice = advise_password("qwerty123", "Gmail", "https://gmail.com", "email");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"keyboard_walk"));
    assert_eq!(advice.recommendations[0].severity, AdviseSeverity::Critical);
}

#[test]
fn test_keyboard_walk_password_word() {
    let advice = advise_password("password1", "Test", "https://test.com", "general");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"keyboard_walk"));
}

#[test]
fn test_repeated_chars() {
    let advice = advise_password("Passaaa123!", "Test", "https://test.com", "general");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(codes.contains(&"repeated_chars"));
}

#[test]
fn test_no_repeated_chars_for_normal_password() {
    let advice = advise_password("MyStr0ng!Pass", "Test", "https://test.com", "general");
    let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
    assert!(!codes.contains(&"repeated_chars"));
}

#[test]
fn test_social_category_lower_requirements() {
    // 12-char password should be appropriate for social
    let advice = advise_password("MyPass123!XY", "Twitter", "https://twitter.com", "social");
    assert_eq!(advice.min_recommended_length, 12);
}

#[test]
fn test_development_category_highest_requirements() {
    let advice = advise_password("short", "GitHub", "https://github.com", "development");
    assert_eq!(advice.min_recommended_length, 20);
}

#[test]
fn test_score_is_bounded_0_to_100() {
    let cases = [
        ("", "Test", "https://test.com", "general"),
        ("a", "Test", "https://test.com", "general"),
        (
            "X9#mK2$pL7@nQ4!vR8^wZ1",
            "Test",
            "https://test.com",
            "general",
        ),
        ("qwerty", "Test", "https://test.com", "banking"),
    ];
    for (pwd, title, url, cat) in &cases {
        let advice = advise_password(pwd, title, url, cat);
        assert!(
            advice.score <= 100,
            "Score {} > 100 for password '{}'",
            advice.score,
            pwd
        );
    }
}

#[test]
fn test_recommendations_sorted_critical_first() {
    let advice = advise_password("qwerty", "GitHub", "https://github.com", "development");
    let severities: Vec<u8> = advice
        .recommendations
        .iter()
        .map(|r| match r.severity {
            AdviseSeverity::Critical => 0,
            AdviseSeverity::Warning => 1,
            AdviseSeverity::Info => 2,
        })
        .collect();
    // Should be non-decreasing (critical before warning before info)
    for i in 1..severities.len() {
        assert!(severities[i - 1] <= severities[i]);
    }
}

#[test]
fn test_vi_messages_populated() {
    let advice = advise_password("weak", "Test", "https://test.com", "banking");
    for rec in &advice.recommendations {
        assert!(
            !rec.message_vi.is_empty(),
            "VI message empty for code: {}",
            rec.code
        );
        assert!(
            !rec.message_en.is_empty(),
            "EN message empty for code: {}",
            rec.code
        );
    }
}

#[test]
fn test_passphrase_suggestion_for_weak() {
    let advice = advise_password("short", "Bank", "https://bank.com", "banking");
    assert!(advice.suggestion_en.is_some());
    assert!(advice.suggestion_vi.is_some());
}

#[test]
fn test_no_suggestion_for_strong_password() {
    let advice = advise_password(
        "X9#mK2$pL7@nQ4!vR8^wZ1",
        "Test",
        "https://test.com",
        "general",
    );
    // Strong password should not need a suggestion
    assert!(advice.suggestion_en.is_none());
}

#[test]
fn test_entropy_all_lowercase() {
    let e = calculate_entropy("abcdefgh");
    // 26^8 → 8 * log2(26) ≈ 37.6 bits
    assert!((e - 37.6).abs() < 1.0, "Expected ~37.6 bits, got {}", e);
}

#[test]
fn test_entropy_increases_with_variety() {
    let e_lower = calculate_entropy("abcdefgh");
    let e_mixed = calculate_entropy("Abc123!@");
    assert!(
        e_mixed > e_lower,
        "Mixed charset should have higher entropy"
    );
}

#[test]
fn test_entropy_increases_with_length() {
    let e_short = calculate_entropy("Abc1!");
    let e_long = calculate_entropy("Abc1!Abc1!Abc1!");
    assert!(e_long > e_short);
}

#[test]
fn test_strong_password_info_message() {
    let advice = advise_password("X9#mK2$pL7@nQ4!vR8^", "Test", "https://test.com", "general");
    // Should have at least one info message saying it's strong
    let has_info = advice
        .recommendations
        .iter()
        .any(|r| r.severity == AdviseSeverity::Info);
    assert!(has_info, "Strong password should have an info message");
}
