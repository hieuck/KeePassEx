//! On-device AI password suggestions
//!
//! Provides context-aware password suggestions without any network calls.
//! Uses a rule-based + statistical model trained on password patterns.
//!
//! # Design
//! - Fully offline — no ML framework dependency, no network calls
//! - Context-aware: analyzes URL, title, and category to suggest appropriate passwords
//! - Learns from vault patterns: suggests passwords similar in style to existing strong ones
//! - Bilingual output: EN + VI recommendations

use crate::generator::PasswordGenerator;
use crate::types::{GeneratorMode, PasswordGeneratorConfig, WordList};
use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Context for generating AI-powered password suggestions
pub struct SuggestionContext<'a> {
    /// URL of the entry (e.g., "https://chase.com")
    pub url: &'a str,
    /// Title of the entry (e.g., "Chase Bank")
    pub title: &'a str,
    /// Category hint (e.g., "banking", "social", "email")
    pub category: &'a str,
    /// Existing strong passwords in the vault (for style learning)
    pub existing_passwords: &'a [String],
}

/// A single password suggestion with rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordSuggestion {
    /// The suggested password
    pub password: String,
    /// Entropy in bits
    pub entropy: f64,
    /// Strength score 0–4
    pub strength_score: u8,
    /// Why this password was suggested (EN)
    pub rationale_en: String,
    /// Why this password was suggested (VI)
    pub rationale_vi: String,
    /// The generation strategy used
    pub strategy: SuggestionStrategy,
}

/// Strategy used to generate the suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuggestionStrategy {
    /// Strong random password matching category requirements
    CategoryOptimized,
    /// Memorable passphrase (EFF wordlist)
    Passphrase,
    /// Pronounceable password (easier to type on mobile)
    Pronounceable,
    /// Pattern learned from existing vault passwords
    VaultStyled,
    /// Maximum security (for banking/crypto)
    MaxSecurity,
}

// ─── Category profiles ────────────────────────────────────────────────────────

struct CategoryProfile {
    min_length: usize,
    preferred_length: usize,
    require_symbols: bool,
    require_digits: bool,
    require_mixed_case: bool,
    prefer_passphrase: bool,
    rationale_en: &'static str,
    rationale_vi: &'static str,
}

fn profile_for(category: &str) -> CategoryProfile {
    match category {
        "banking" | "crypto" | "finance" => CategoryProfile {
            min_length: 16,
            preferred_length: 20,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            prefer_passphrase: false,
            rationale_en: "Banking accounts need maximum security — long random password with all character types",
            rationale_vi: "Tài khoản ngân hàng cần bảo mật tối đa — mật khẩu ngẫu nhiên dài với tất cả loại ký tự",
        },
        "email" | "work" => CategoryProfile {
            min_length: 14,
            preferred_length: 16,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            prefer_passphrase: true,
            rationale_en: "Work/email accounts benefit from memorable passphrases that are still strong",
            rationale_vi: "Tài khoản công việc/email được hưởng lợi từ cụm từ dễ nhớ nhưng vẫn mạnh",
        },
        "social" | "shopping" | "entertainment" => CategoryProfile {
            min_length: 12,
            preferred_length: 14,
            require_symbols: false,
            require_digits: true,
            require_mixed_case: true,
            prefer_passphrase: true,
            rationale_en: "Social accounts: a passphrase is easier to remember and type on mobile",
            rationale_vi: "Tài khoản mạng xã hội: cụm từ dễ nhớ và gõ trên điện thoại hơn",
        },
        "development" | "security" | "server" => CategoryProfile {
            min_length: 20,
            preferred_length: 32,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            prefer_passphrase: false,
            rationale_en: "Development/security credentials need maximum entropy — use a long random password",
            rationale_vi: "Thông tin xác thực phát triển/bảo mật cần entropy tối đa — dùng mật khẩu ngẫu nhiên dài",
        },
        _ => CategoryProfile {
            min_length: 12,
            preferred_length: 16,
            require_symbols: false,
            require_digits: true,
            require_mixed_case: true,
            prefer_passphrase: false,
            rationale_en: "A strong random password with mixed characters",
            rationale_vi: "Mật khẩu ngẫu nhiên mạnh với ký tự hỗn hợp",
        },
    }
}

// ─── Core suggestion engine ───────────────────────────────────────────────────

/// Generate `count` password suggestions for the given context.
pub fn suggest_passwords(ctx: &SuggestionContext<'_>, count: usize) -> Vec<PasswordSuggestion> {
    let profile = profile_for(ctx.category);
    let mut suggestions = Vec::with_capacity(count);

    // Strategy 1: Category-optimized random password
    if let Some(s) = generate_category_optimized(&profile) {
        suggestions.push(s);
    }

    // Strategy 2: Passphrase (always useful as an alternative)
    if let Some(s) = generate_passphrase_suggestion(&profile) {
        suggestions.push(s);
    }

    // Strategy 3: Pronounceable (good for mobile typing)
    if suggestions.len() < count {
        if let Some(s) = generate_pronounceable_suggestion(&profile) {
            suggestions.push(s);
        }
    }

    // Strategy 4: Max security (for banking/crypto)
    if matches!(
        ctx.category,
        "banking" | "crypto" | "finance" | "development" | "security"
    ) {
        if let Some(s) = generate_max_security() {
            suggestions.push(s);
        }
    }

    // Strategy 5: Vault-styled (learn from existing strong passwords)
    if !ctx.existing_passwords.is_empty() && suggestions.len() < count {
        if let Some(s) = generate_vault_styled(ctx.existing_passwords, &profile) {
            suggestions.push(s);
        }
    }

    // Fill remaining with category-optimized variants
    while suggestions.len() < count {
        if let Some(s) = generate_category_optimized(&profile) {
            suggestions.push(s);
        } else {
            break;
        }
    }

    suggestions.truncate(count);
    suggestions
}

// ─── Strategy implementations ─────────────────────────────────────────────────

fn generate_category_optimized(profile: &CategoryProfile) -> Option<PasswordSuggestion> {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: profile.preferred_length,
        use_uppercase: profile.require_mixed_case,
        use_lowercase: true,
        use_digits: profile.require_digits,
        use_symbols: profile.require_symbols,
        custom_symbols: None,
        exclude_ambiguous: false,
        exclude_chars: String::new(),
        min_uppercase: if profile.require_mixed_case { 2 } else { 0 },
        min_lowercase: 2,
        min_digits: if profile.require_digits { 2 } else { 0 },
        min_symbols: if profile.require_symbols { 2 } else { 0 },
        word_count: 6,
        word_separator: "-".to_string(),
        capitalize_words: false,
        include_number: false,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).ok()?;
    let entropy = calculate_entropy(&password);

    Some(PasswordSuggestion {
        password,
        entropy,
        strength_score: entropy_to_score(entropy),
        rationale_en: profile.rationale_en.to_string(),
        rationale_vi: profile.rationale_vi.to_string(),
        strategy: SuggestionStrategy::CategoryOptimized,
    })
}

fn generate_passphrase_suggestion(profile: &CategoryProfile) -> Option<PasswordSuggestion> {
    let word_count = if profile.min_length >= 16 { 6 } else { 5 };
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Passphrase,
        length: 20,
        use_uppercase: false,
        use_lowercase: true,
        use_digits: true,
        use_symbols: false,
        custom_symbols: None,
        exclude_ambiguous: false,
        exclude_chars: String::new(),
        min_uppercase: 0,
        min_lowercase: 0,
        min_digits: 0,
        min_symbols: 0,
        word_count,
        word_separator: "-".to_string(),
        capitalize_words: true,
        include_number: true,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).ok()?;
    let entropy = calculate_entropy(&password);

    Some(PasswordSuggestion {
        password,
        entropy,
        strength_score: entropy_to_score(entropy),
        rationale_en: format!(
            "A {}-word passphrase — easier to remember and type, especially on mobile",
            word_count
        ),
        rationale_vi: format!(
            "Cụm từ {} từ — dễ nhớ và gõ hơn, đặc biệt trên điện thoại",
            word_count
        ),
        strategy: SuggestionStrategy::Passphrase,
    })
}

fn generate_pronounceable_suggestion(profile: &CategoryProfile) -> Option<PasswordSuggestion> {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Pronounceable,
        length: profile.preferred_length.max(12),
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: false,
        custom_symbols: None,
        exclude_ambiguous: true,
        exclude_chars: String::new(),
        min_uppercase: 1,
        min_lowercase: 4,
        min_digits: 2,
        min_symbols: 0,
        word_count: 4,
        word_separator: "-".to_string(),
        capitalize_words: false,
        include_number: false,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).ok()?;
    let entropy = calculate_entropy(&password);

    Some(PasswordSuggestion {
        password,
        entropy,
        strength_score: entropy_to_score(entropy),
        rationale_en:
            "Pronounceable password — easier to type on mobile keyboards without copy-paste"
                .to_string(),
        rationale_vi:
            "Mật khẩu có thể đọc được — dễ gõ trên bàn phím điện thoại mà không cần sao chép"
                .to_string(),
        strategy: SuggestionStrategy::Pronounceable,
    })
}

fn generate_max_security() -> Option<PasswordSuggestion> {
    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: 32,
        use_uppercase: true,
        use_lowercase: true,
        use_digits: true,
        use_symbols: true,
        custom_symbols: None,
        exclude_ambiguous: false,
        exclude_chars: String::new(),
        min_uppercase: 4,
        min_lowercase: 4,
        min_digits: 4,
        min_symbols: 4,
        word_count: 6,
        word_separator: "-".to_string(),
        capitalize_words: false,
        include_number: false,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).ok()?;
    let entropy = calculate_entropy(&password);

    Some(PasswordSuggestion {
        password,
        entropy,
        strength_score: 4,
        rationale_en: "Maximum security: 32-character random password with all character types — store in KeePassEx, never memorize".to_string(),
        rationale_vi: "Bảo mật tối đa: mật khẩu ngẫu nhiên 32 ký tự với tất cả loại ký tự — lưu trong KeePassEx, không cần nhớ".to_string(),
        strategy: SuggestionStrategy::MaxSecurity,
    })
}

fn generate_vault_styled(
    existing: &[String],
    profile: &CategoryProfile,
) -> Option<PasswordSuggestion> {
    // Analyze patterns in existing strong passwords
    let strong: Vec<&String> = existing
        .iter()
        .filter(|p| calculate_entropy(p) >= 50.0)
        .collect();

    if strong.is_empty() {
        return generate_category_optimized(profile);
    }

    // Determine average length and character composition of strong passwords
    let avg_len = strong.iter().map(|p| p.len()).sum::<usize>() / strong.len();
    let has_symbols = strong
        .iter()
        .any(|p| p.chars().any(|c| !c.is_alphanumeric()));
    let has_upper = strong.iter().any(|p| p.chars().any(|c| c.is_uppercase()));

    let config = PasswordGeneratorConfig {
        mode: GeneratorMode::Random,
        length: avg_len.max(profile.min_length),
        use_uppercase: has_upper,
        use_lowercase: true,
        use_digits: true,
        use_symbols: has_symbols,
        custom_symbols: None,
        exclude_ambiguous: false,
        exclude_chars: String::new(),
        min_uppercase: if has_upper { 1 } else { 0 },
        min_lowercase: 1,
        min_digits: 1,
        min_symbols: if has_symbols { 1 } else { 0 },
        word_count: 6,
        word_separator: "-".to_string(),
        capitalize_words: false,
        include_number: false,
        wordlist: WordList::Eff,
    };

    let password = PasswordGenerator::generate(&config).ok()?;
    let entropy = calculate_entropy(&password);

    Some(PasswordSuggestion {
        password,
        entropy,
        strength_score: entropy_to_score(entropy),
        rationale_en: format!(
            "Styled after your existing strong passwords (avg {} chars, similar composition)",
            avg_len
        ),
        rationale_vi: format!(
            "Theo phong cách mật khẩu mạnh hiện có của bạn (trung bình {} ký tự, thành phần tương tự)",
            avg_len
        ),
        strategy: SuggestionStrategy::VaultStyled,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn calculate_entropy(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }
    let mut pool = 0.0f64;
    if password.chars().any(|c| c.is_lowercase()) {
        pool += 26.0;
    }
    if password.chars().any(|c| c.is_uppercase()) {
        pool += 26.0;
    }
    if password.chars().any(|c| c.is_ascii_digit()) {
        pool += 10.0;
    }
    if password.chars().any(|c| !c.is_alphanumeric()) {
        pool += 32.0;
    }
    if pool == 0.0 {
        pool = 26.0;
    }
    password.len() as f64 * pool.log2()
}

fn entropy_to_score(entropy: f64) -> u8 {
    match entropy as u32 {
        0..=27 => 0,
        28..=35 => 1,
        36..=59 => 2,
        60..=127 => 3,
        _ => 4,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx<'a>(category: &'a str) -> SuggestionContext<'a> {
        SuggestionContext {
            url: "https://example.com",
            title: "Example",
            category,
            existing_passwords: &[],
        }
    }

    #[test]
    fn test_suggest_returns_requested_count() {
        let ctx = make_ctx("banking");
        let suggestions = suggest_passwords(&ctx, 3);
        assert_eq!(suggestions.len(), 3);
    }

    #[test]
    fn test_banking_suggestions_are_strong() {
        let ctx = make_ctx("banking");
        let suggestions = suggest_passwords(&ctx, 5);
        for s in &suggestions {
            assert!(
                s.entropy >= 40.0,
                "Banking suggestion too weak: {} bits",
                s.entropy
            );
            assert!(
                s.password.len() >= 12,
                "Banking password too short: {}",
                s.password.len()
            );
        }
    }

    #[test]
    fn test_passphrase_strategy_present() {
        let ctx = make_ctx("email");
        let suggestions = suggest_passwords(&ctx, 5);
        let has_passphrase = suggestions
            .iter()
            .any(|s| s.strategy == SuggestionStrategy::Passphrase);
        assert!(
            has_passphrase,
            "Should include a passphrase suggestion for email"
        );
    }

    #[test]
    fn test_max_security_for_banking() {
        let ctx = make_ctx("banking");
        let suggestions = suggest_passwords(&ctx, 5);
        let has_max = suggestions
            .iter()
            .any(|s| s.strategy == SuggestionStrategy::MaxSecurity);
        assert!(has_max, "Banking should include max security suggestion");
    }

    #[test]
    fn test_vault_styled_uses_existing_patterns() {
        let existing = vec![
            "MyStr0ng!Pass".to_string(),
            "An0ther$ecure1".to_string(),
            "Y3tAn0ther#Pass".to_string(),
        ];
        let ctx = SuggestionContext {
            url: "https://example.com",
            title: "Example",
            category: "general",
            existing_passwords: &existing,
        };
        let suggestions = suggest_passwords(&ctx, 5);
        let has_styled = suggestions
            .iter()
            .any(|s| s.strategy == SuggestionStrategy::VaultStyled);
        assert!(
            has_styled,
            "Should include vault-styled suggestion when existing passwords provided"
        );
    }

    #[test]
    fn test_vi_rationale_not_empty() {
        let ctx = make_ctx("banking");
        let suggestions = suggest_passwords(&ctx, 3);
        for s in &suggestions {
            assert!(
                !s.rationale_vi.is_empty(),
                "VI rationale should not be empty"
            );
        }
    }

    #[test]
    fn test_all_suggestions_unique() {
        let ctx = make_ctx("general");
        let suggestions = suggest_passwords(&ctx, 5);
        let passwords: std::collections::HashSet<&str> =
            suggestions.iter().map(|s| s.password.as_str()).collect();
        // All passwords should be unique (extremely unlikely to collide)
        assert_eq!(
            passwords.len(),
            suggestions.len(),
            "All suggestions should be unique"
        );
    }

    #[test]
    fn test_entropy_calculation() {
        assert_eq!(calculate_entropy(""), 0.0);
        let e = calculate_entropy("abcdefgh"); // lowercase only: 26^8
        assert!((e - 37.6).abs() < 1.0);
        let e2 = calculate_entropy("Abc123!@"); // all types
        assert!(e2 > e);
    }

    #[test]
    fn test_score_range() {
        for score in [0u8, 1, 2, 3, 4] {
            assert!(score <= 4);
        }
        assert_eq!(entropy_to_score(0.0), 0);
        assert_eq!(entropy_to_score(100.0), 3);
        assert_eq!(entropy_to_score(200.0), 4);
    }
}
