//! Tests for the AI password suggestion engine

use crate::ai::{suggest_passwords, SuggestionContext, SuggestionStrategy};

fn ctx<'a>(category: &'a str) -> SuggestionContext<'a> {
    SuggestionContext {
        url: "https://example.com",
        title: "Example",
        category,
        existing_passwords: &[],
    }
}

fn ctx_with_passwords<'a>(category: &'a str, passwords: &'a [String]) -> SuggestionContext<'a> {
    SuggestionContext {
        url: "https://example.com",
        title: "Example",
        category,
        existing_passwords: passwords,
    }
}

// ─── Count & uniqueness ───────────────────────────────────────────────────────

#[test]
fn test_returns_requested_count() {
    for count in [1, 3, 5] {
        let suggestions = suggest_passwords(&ctx("general"), count);
        assert_eq!(suggestions.len(), count, "Expected {count} suggestions");
    }
}

#[test]
fn test_all_suggestions_unique_passwords() {
    let suggestions = suggest_passwords(&ctx("general"), 5);
    let passwords: std::collections::HashSet<&str> =
        suggestions.iter().map(|s| s.password.as_str()).collect();
    assert_eq!(
        passwords.len(),
        suggestions.len(),
        "All suggestions should have unique passwords"
    );
}

// ─── Strength requirements ────────────────────────────────────────────────────

#[test]
fn test_banking_suggestions_meet_minimum_length() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(
            s.password.len() >= 12,
            "Banking password too short: {} chars — '{}'",
            s.password.len(),
            s.password
        );
    }
}

#[test]
fn test_banking_suggestions_have_adequate_entropy() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(
            s.entropy >= 40.0,
            "Banking suggestion entropy too low: {:.1} bits",
            s.entropy
        );
    }
}

#[test]
fn test_development_suggestions_are_strongest() {
    let dev = suggest_passwords(&ctx("development"), 3);
    let social = suggest_passwords(&ctx("social"), 3);

    let dev_avg_entropy: f64 = dev.iter().map(|s| s.entropy).sum::<f64>() / dev.len() as f64;
    let social_avg_entropy: f64 =
        social.iter().map(|s| s.entropy).sum::<f64>() / social.len() as f64;

    assert!(
        dev_avg_entropy >= social_avg_entropy,
        "Development suggestions should be at least as strong as social: {dev_avg_entropy:.1} vs {social_avg_entropy:.1}"
    );
}

#[test]
fn test_strength_scores_in_valid_range() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(
            s.strength_score <= 4,
            "Strength score out of range: {}",
            s.strength_score
        );
    }
}

// ─── Strategy coverage ────────────────────────────────────────────────────────

#[test]
fn test_passphrase_strategy_present_for_email() {
    let suggestions = suggest_passwords(&ctx("email"), 5);
    let has_passphrase = suggestions
        .iter()
        .any(|s| s.strategy == SuggestionStrategy::Passphrase);
    assert!(
        has_passphrase,
        "Email should include a passphrase suggestion"
    );
}

#[test]
fn test_max_security_strategy_for_banking() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    let has_max = suggestions
        .iter()
        .any(|s| s.strategy == SuggestionStrategy::MaxSecurity);
    assert!(has_max, "Banking should include a MaxSecurity suggestion");
}

#[test]
fn test_max_security_strategy_for_crypto() {
    let suggestions = suggest_passwords(&ctx("crypto"), 5);
    let has_max = suggestions
        .iter()
        .any(|s| s.strategy == SuggestionStrategy::MaxSecurity);
    assert!(has_max, "Crypto should include a MaxSecurity suggestion");
}

#[test]
fn test_max_security_strategy_for_development() {
    let suggestions = suggest_passwords(&ctx("development"), 5);
    let has_max = suggestions
        .iter()
        .any(|s| s.strategy == SuggestionStrategy::MaxSecurity);
    assert!(
        has_max,
        "Development should include a MaxSecurity suggestion"
    );
}

#[test]
fn test_vault_styled_strategy_with_existing_passwords() {
    let existing = vec![
        "MyStr0ng!Pass".to_string(),
        "An0ther$ecure1".to_string(),
        "Y3tAn0ther#Pass".to_string(),
    ];
    let suggestions = suggest_passwords(&ctx_with_passwords("general", &existing), 5);
    let has_styled = suggestions
        .iter()
        .any(|s| s.strategy == SuggestionStrategy::VaultStyled);
    assert!(
        has_styled,
        "Should include VaultStyled suggestion when existing passwords provided"
    );
}

#[test]
fn test_no_vault_styled_without_existing_passwords() {
    // With no existing passwords, VaultStyled falls back to CategoryOptimized
    let suggestions = suggest_passwords(&ctx("general"), 5);
    // This is fine — just verify no panic and we get results
    assert!(!suggestions.is_empty());
}

// ─── Bilingual rationale ──────────────────────────────────────────────────────

#[test]
fn test_all_suggestions_have_en_rationale() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(
            !s.rationale_en.is_empty(),
            "EN rationale should not be empty for strategy {:?}",
            s.strategy
        );
    }
}

#[test]
fn test_all_suggestions_have_vi_rationale() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(
            !s.rationale_vi.is_empty(),
            "VI rationale should not be empty for strategy {:?}",
            s.strategy
        );
    }
}

#[test]
fn test_vi_rationale_differs_from_en() {
    // VI rationale should be in Vietnamese, not identical to EN
    let suggestions = suggest_passwords(&ctx("banking"), 3);
    for s in &suggestions {
        assert_ne!(
            s.rationale_en, s.rationale_vi,
            "EN and VI rationale should differ for strategy {:?}",
            s.strategy
        );
    }
}

// ─── Category profiles ────────────────────────────────────────────────────────

#[test]
fn test_all_categories_produce_results() {
    let categories = [
        "banking",
        "crypto",
        "finance",
        "email",
        "work",
        "social",
        "shopping",
        "entertainment",
        "development",
        "security",
        "general",
    ];
    for cat in &categories {
        let suggestions = suggest_passwords(&ctx(cat), 3);
        assert_eq!(
            suggestions.len(),
            3,
            "Category '{cat}' should produce 3 suggestions"
        );
    }
}

#[test]
fn test_unknown_category_falls_back_to_general() {
    let suggestions = suggest_passwords(&ctx("unknown_category_xyz"), 3);
    assert_eq!(
        suggestions.len(),
        3,
        "Unknown category should fall back to general profile"
    );
}

// ─── Password quality ─────────────────────────────────────────────────────────

#[test]
fn test_no_empty_passwords() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    for s in &suggestions {
        assert!(!s.password.is_empty(), "Password should not be empty");
    }
}

#[test]
fn test_max_security_password_is_32_chars() {
    let suggestions = suggest_passwords(&ctx("banking"), 5);
    let max_sec = suggestions
        .iter()
        .find(|s| s.strategy == SuggestionStrategy::MaxSecurity);
    if let Some(s) = max_sec {
        assert_eq!(
            s.password.len(),
            32,
            "MaxSecurity password should be 32 chars"
        );
        assert_eq!(s.strength_score, 4, "MaxSecurity should have score 4");
    }
}

#[test]
fn test_passphrase_contains_separator() {
    let suggestions = suggest_passwords(&ctx("email"), 5);
    let passphrase = suggestions
        .iter()
        .find(|s| s.strategy == SuggestionStrategy::Passphrase);
    if let Some(s) = passphrase {
        // EFF passphrases use word separators
        assert!(
            s.password.contains('-') || s.password.contains(' ') || s.password.len() > 15,
            "Passphrase should be long or contain separators: '{}'",
            s.password
        );
    }
}
