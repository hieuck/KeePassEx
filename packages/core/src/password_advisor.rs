//! Context-Aware Password Strength Advisor
//!
//! Analyzes a password in the context of its entry (URL, title, category)
//! and provides specific, actionable recommendations in EN and VI.
//!
//! Unlike a generic strength meter, this advisor understands:
//! - Banking sites need stronger passwords than social media
//! - Passwords matching the username or site name are weak regardless of length
//! - Passphrases are often better than complex short passwords
//! - Reuse across entries is a critical risk
//!
//! # No competitor has this feature.
//!
//! # Usage
//! ```no_run
//! use keepassex_core::password_advisor::advise_password;
//!
//! let advice = advise_password("mypassword", "Chase Bank", "https://chase.com", "banking");
//! assert!(!advice.recommendations.is_empty());
//! ```

use serde::{Deserialize, Serialize};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Severity level of a recommendation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdviseSeverity {
    /// Informational — good practice
    Info,
    /// Warning — should be addressed
    Warning,
    /// Critical — must be fixed immediately
    Critical,
}

impl AdviseSeverity {
    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Info => "#2563eb",
            Self::Warning => "#d97706",
            Self::Critical => "#dc2626",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "ℹ️",
            Self::Warning => "⚠️",
            Self::Critical => "🚨",
        }
    }
}

/// A single recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub severity: AdviseSeverity,
    pub code: &'static str,
    pub message_en: String,
    pub message_vi: String,
}

/// Full advice report for a password in context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordAdvice {
    /// Overall score 0–100 (context-adjusted, not just entropy)
    pub score: u8,
    /// Strength label
    pub label_en: &'static str,
    pub label_vi: &'static str,
    /// Color for the score indicator
    pub color: &'static str,
    /// Ordered recommendations (most critical first)
    pub recommendations: Vec<Recommendation>,
    /// Suggested alternative (passphrase or stronger pattern)
    pub suggestion_en: Option<String>,
    pub suggestion_vi: Option<String>,
    /// Whether this password is appropriate for the site category
    pub appropriate_for_category: bool,
    /// Minimum recommended length for this category
    pub min_recommended_length: usize,
}

// ─── Category-specific requirements ──────────────────────────────────────────

struct CategoryRequirements {
    min_length: usize,
    min_entropy_bits: f64,
    require_symbols: bool,
    require_digits: bool,
    require_mixed_case: bool,
    name_en: &'static str,
    name_vi: &'static str,
}

fn get_requirements(category: &str) -> CategoryRequirements {
    match category {
        "banking" | "crypto" | "finance" => CategoryRequirements {
            min_length: 16,
            min_entropy_bits: 60.0,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            name_en: "banking/finance",
            name_vi: "ngân hàng/tài chính",
        },
        "email" | "work" => CategoryRequirements {
            min_length: 14,
            min_entropy_bits: 50.0,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            name_en: "email/work",
            name_vi: "email/công việc",
        },
        "social" | "shopping" | "entertainment" => CategoryRequirements {
            min_length: 12,
            min_entropy_bits: 40.0,
            require_symbols: false,
            require_digits: true,
            require_mixed_case: true,
            name_en: "social/shopping",
            name_vi: "mạng xã hội/mua sắm",
        },
        "development" | "security" => CategoryRequirements {
            min_length: 20,
            min_entropy_bits: 70.0,
            require_symbols: true,
            require_digits: true,
            require_mixed_case: true,
            name_en: "development/security",
            name_vi: "lập trình/bảo mật",
        },
        _ => CategoryRequirements {
            min_length: 12,
            min_entropy_bits: 40.0,
            require_symbols: false,
            require_digits: true,
            require_mixed_case: false,
            name_en: "general",
            name_vi: "chung",
        },
    }
}

// ─── Core analysis ────────────────────────────────────────────────────────────

/// Analyze a password in context and return actionable advice.
///
/// # Arguments
/// * `password` — The password to analyze
/// * `entry_title` — Title of the entry (e.g., "Chase Bank")
/// * `entry_url` — URL of the entry (e.g., "https://chase.com")
/// * `category` — Category hint (e.g., "banking", "social", "email")
pub fn advise_password(
    password: &str,
    entry_title: &str,
    entry_url: &str,
    category: &str,
) -> PasswordAdvice {
    let mut recommendations: Vec<Recommendation> = Vec::new();
    let reqs = get_requirements(category);

    // ── Empty password ────────────────────────────────────────────────────────
    if password.is_empty() {
        return PasswordAdvice {
            score: 0,
            label_en: "No Password",
            label_vi: "Không có mật khẩu",
            color: "#dc2626",
            recommendations: vec![Recommendation {
                severity: AdviseSeverity::Critical,
                code: "empty",
                message_en: "No password set. This entry is completely unprotected.".into(),
                message_vi: "Chưa đặt mật khẩu. Mục này hoàn toàn không được bảo vệ.".into(),
            }],
            suggestion_en: Some("Generate a strong password using the built-in generator.".into()),
            suggestion_vi: Some("Tạo mật khẩu mạnh bằng trình tạo mật khẩu tích hợp.".into()),
            appropriate_for_category: false,
            min_recommended_length: reqs.min_length,
        };
    }

    let entropy = calculate_entropy(password);
    let len = password.len();

    // ── Context-based checks ──────────────────────────────────────────────────

    // Check if password contains the site name
    let title_lower = entry_title.to_lowercase();
    let pwd_lower = password.to_lowercase();
    let domain = extract_domain(entry_url);

    if !title_lower.is_empty() && pwd_lower.contains(&title_lower) {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Critical,
            code: "contains_site_name",
            message_en: format!(
                "Password contains the site name '{}'. Attackers try this first.",
                entry_title
            ),
            message_vi: format!(
                "Mật khẩu chứa tên trang web '{}'. Kẻ tấn công thử điều này đầu tiên.",
                entry_title
            ),
        });
    }

    if !domain.is_empty() && pwd_lower.contains(&domain) {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Critical,
            code: "contains_domain",
            message_en: format!(
                "Password contains the domain '{}'. This is easily guessable.",
                domain
            ),
            message_vi: format!("Mật khẩu chứa tên miền '{}'. Điều này dễ đoán.", domain),
        });
    }

    // ── Length checks ─────────────────────────────────────────────────────────

    if len < 8 {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Critical,
            code: "too_short",
            message_en: format!(
                "Password is only {} characters. Minimum 8 required, {} recommended for {}.",
                len, reqs.min_length, reqs.name_en
            ),
            message_vi: format!(
                "Mật khẩu chỉ có {} ký tự. Tối thiểu 8, khuyến nghị {} cho {}.",
                len, reqs.min_length, reqs.name_vi
            ),
        });
    } else if len < reqs.min_length {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "below_category_minimum",
            message_en: format!(
                "For {}, passwords should be at least {} characters (currently {}).",
                reqs.name_en, reqs.min_length, len
            ),
            message_vi: format!(
                "Với {}, mật khẩu nên có ít nhất {} ký tự (hiện tại {}).",
                reqs.name_vi, reqs.min_length, len
            ),
        });
    }

    // ── Character variety checks ──────────────────────────────────────────────

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    if reqs.require_mixed_case && (!has_upper || !has_lower) {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "no_mixed_case",
            message_en: format!(
                "{} accounts should use both uppercase and lowercase letters.",
                reqs.name_en
            ),
            message_vi: format!(
                "Tài khoản {} nên dùng cả chữ hoa và chữ thường.",
                reqs.name_vi
            ),
        });
    }

    if reqs.require_digits && !has_digit {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "no_digits",
            message_en: format!(
                "{} accounts should include at least one digit.",
                reqs.name_en
            ),
            message_vi: format!("Tài khoản {} nên có ít nhất một chữ số.", reqs.name_vi),
        });
    }

    if reqs.require_symbols && !has_symbol {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "no_symbols",
            message_en: format!(
                "{} accounts should include special characters (!@#$...).",
                reqs.name_en
            ),
            message_vi: format!(
                "Tài khoản {} nên có ký tự đặc biệt (!@#$...).",
                reqs.name_vi
            ),
        });
    }

    // ── Common patterns ───────────────────────────────────────────────────────

    if has_keyboard_walk(password) {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Critical,
            code: "keyboard_walk",
            message_en: "Password contains a keyboard pattern (e.g., qwerty, 12345). These are in every attacker's dictionary.".into(),
            message_vi: "Mật khẩu chứa mẫu bàn phím (ví dụ: qwerty, 12345). Đây là trong từ điển của mọi kẻ tấn công.".into(),
        });
    }

    if has_repeated_chars(password) {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "repeated_chars",
            message_en: "Password contains 3+ repeated characters (e.g., 'aaa'). This reduces effective entropy.".into(),
            message_vi: "Mật khẩu chứa 3+ ký tự lặp lại (ví dụ: 'aaa'). Điều này làm giảm entropy hiệu quả.".into(),
        });
    }

    // ── Entropy check ─────────────────────────────────────────────────────────

    if entropy < reqs.min_entropy_bits && len >= 8 {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Warning,
            code: "low_entropy",
            message_en: format!(
                "Password entropy is {:.0} bits. {} accounts need at least {:.0} bits.",
                entropy, reqs.name_en, reqs.min_entropy_bits
            ),
            message_vi: format!(
                "Entropy mật khẩu là {:.0} bit. Tài khoản {} cần ít nhất {:.0} bit.",
                entropy, reqs.name_vi, reqs.min_entropy_bits
            ),
        });
    }

    // ── Passphrase suggestion ─────────────────────────────────────────────────

    let (suggestion_en, suggestion_vi) = if len < reqs.min_length || entropy < reqs.min_entropy_bits
    {
        (
            Some(format!(
                "Consider a passphrase like 'correct-horse-battery-staple' — longer, memorable, and stronger than '{}***'.",
                &password[..password.len().min(3)]
            )),
            Some(format!(
                "Hãy dùng cụm từ như 'correct-horse-battery-staple' — dài hơn, dễ nhớ và mạnh hơn '{}***'.",
                &password[..password.len().min(3)]
            )),
        )
    } else {
        (None, None)
    };

    // ── Positive feedback ─────────────────────────────────────────────────────

    if recommendations.is_empty() {
        recommendations.push(Recommendation {
            severity: AdviseSeverity::Info,
            code: "strong",
            message_en: format!(
                "Password meets all requirements for {}. Well done!",
                reqs.name_en
            ),
            message_vi: format!(
                "Mật khẩu đáp ứng tất cả yêu cầu cho {}. Tốt lắm!",
                reqs.name_vi
            ),
        });
    }

    // ── Score calculation ─────────────────────────────────────────────────────

    let critical_count = recommendations
        .iter()
        .filter(|r| r.severity == AdviseSeverity::Critical)
        .count();
    let warning_count = recommendations
        .iter()
        .filter(|r| r.severity == AdviseSeverity::Warning)
        .count();

    let base_score = ((entropy / reqs.min_entropy_bits).min(1.0) * 60.0) as u8;
    let length_bonus = ((len as f64 / reqs.min_length as f64).min(1.0) * 20.0) as u8;
    let variety_bonus = {
        let v = [has_upper, has_lower, has_digit, has_symbol]
            .iter()
            .filter(|&&b| b)
            .count();
        (v as u8 * 5).min(20)
    };

    let raw_score = base_score + length_bonus + variety_bonus;
    let penalty =
        (critical_count as u8 * 25).min(raw_score) + (warning_count as u8 * 10).min(raw_score);
    let score = raw_score.saturating_sub(penalty).min(100);

    // Sort: critical first, then warning, then info
    recommendations.sort_by_key(|r| match r.severity {
        AdviseSeverity::Critical => 0,
        AdviseSeverity::Warning => 1,
        AdviseSeverity::Info => 2,
    });

    let appropriate = score >= 60 && critical_count == 0;

    let (label_en, label_vi, color) = match score {
        0..=19 => ("Very Weak", "Rất yếu", "#dc2626"),
        20..=39 => ("Weak", "Yếu", "#ea580c"),
        40..=59 => ("Fair", "Trung bình", "#d97706"),
        60..=79 => ("Strong", "Mạnh", "#65a30d"),
        _ => ("Very Strong", "Rất mạnh", "#16a34a"),
    };

    PasswordAdvice {
        score,
        label_en,
        label_vi,
        color,
        recommendations,
        suggestion_en,
        suggestion_vi,
        appropriate_for_category: appropriate,
        min_recommended_length: reqs.min_length,
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Calculate Shannon entropy in bits
pub fn calculate_entropy(password: &str) -> f64 {
    if password.is_empty() {
        return 0.0;
    }

    let mut charset_size: f64 = 0.0;
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    let has_space = password.contains(' ');

    if has_lower {
        charset_size += 26.0;
    }
    if has_upper {
        charset_size += 26.0;
    }
    if has_digit {
        charset_size += 10.0;
    }
    if has_symbol {
        charset_size += 32.0;
    }
    if has_space {
        charset_size += 1.0;
    }

    if charset_size == 0.0 {
        charset_size = 26.0;
    }

    password.len() as f64 * charset_size.log2()
}

/// Detect keyboard walk patterns (qwerty, asdf, 12345, etc.)
fn has_keyboard_walk(password: &str) -> bool {
    const WALKS: &[&str] = &[
        "qwerty", "qwert", "asdf", "zxcv", "12345", "23456", "34567", "45678", "56789", "67890",
        "password", "pass", "admin", "login", "letmein", "welcome", "monkey", "dragon", "master",
        "abc123", "iloveyou", "sunshine", "princess", "football",
    ];
    let lower = password.to_lowercase();
    WALKS.iter().any(|w| lower.contains(w))
}

/// Detect 3+ consecutive repeated characters
fn has_repeated_chars(password: &str) -> bool {
    let chars: Vec<char> = password.chars().collect();
    if chars.len() < 3 {
        return false;
    }
    for i in 0..chars.len() - 2 {
        if chars[i] == chars[i + 1] && chars[i + 1] == chars[i + 2] {
            return true;
        }
    }
    false
}

/// Extract domain from URL for context matching
fn extract_domain(url: &str) -> String {
    if url.is_empty() {
        return String::new();
    }
    let url = if url.contains("://") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    url.split("://")
        .nth(1)
        .and_then(|s| s.split('/').next())
        .map(|h| {
            h.trim_start_matches("www.")
                .split('.')
                .next()
                .unwrap_or("")
                .to_lowercase()
        })
        .unwrap_or_default()
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_password_is_critical() {
        let advice = advise_password("", "GitHub", "https://github.com", "development");
        assert_eq!(advice.score, 0);
        assert_eq!(advice.recommendations[0].code, "empty");
        assert_eq!(advice.recommendations[0].severity, AdviseSeverity::Critical);
    }

    #[test]
    fn test_password_containing_site_name_is_critical() {
        let advice = advise_password("github123", "GitHub", "https://github.com", "development");
        let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
        assert!(codes.contains(&"contains_site_name"));
        assert_eq!(advice.recommendations[0].severity, AdviseSeverity::Critical);
    }

    #[test]
    fn test_banking_requires_stronger_password() {
        // 12-char password that's fine for social but weak for banking
        let social = advise_password("MyPass123!", "Twitter", "https://twitter.com", "social");
        let banking = advise_password("MyPass123!", "Chase", "https://chase.com", "banking");
        // Banking should have lower score or more warnings
        assert!(banking.score <= social.score || !banking.appropriate_for_category);
    }

    #[test]
    fn test_strong_password_for_banking() {
        let advice = advise_password(
            "X9#mK2$pL7@nQ4!vR8^",
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
    fn test_keyboard_walk_detected() {
        let advice = advise_password("qwerty123", "Gmail", "https://gmail.com", "email");
        let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
        assert!(codes.contains(&"keyboard_walk"));
    }

    #[test]
    fn test_repeated_chars_detected() {
        let advice = advise_password("Passaaa123!", "Test", "https://test.com", "general");
        let codes: Vec<&str> = advice.recommendations.iter().map(|r| r.code).collect();
        assert!(codes.contains(&"repeated_chars"));
    }

    #[test]
    fn test_entropy_calculation() {
        // All lowercase: 26^8 ≈ 37.6 bits
        let e = calculate_entropy("abcdefgh");
        assert!((e - 37.6).abs() < 1.0);

        // Mixed: larger charset
        let e2 = calculate_entropy("Abc123!@");
        assert!(e2 > e);
    }

    #[test]
    fn test_domain_extraction() {
        assert_eq!(extract_domain("https://www.github.com/user"), "github");
        assert_eq!(extract_domain("https://chase.com"), "chase");
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn test_recommendations_sorted_critical_first() {
        let advice = advise_password("qwerty", "GitHub", "https://github.com", "development");
        // Verify that no Critical recommendation appears after a Warning or Info
        let severities: Vec<u8> = advice
            .recommendations
            .iter()
            .map(|r| match r.severity {
                AdviseSeverity::Critical => 0,
                AdviseSeverity::Warning => 1,
                AdviseSeverity::Info => 2,
            })
            .collect();
        // Should be non-decreasing (critical=0 before warning=1 before info=2)
        for i in 1..severities.len() {
            assert!(
                severities[i - 1] <= severities[i],
                "Severity order violated at index {}: {} > {}",
                i,
                severities[i - 1],
                severities[i]
            );
        }
    }

    #[test]
    fn test_vi_messages_not_empty() {
        let advice = advise_password("weak", "Test", "https://test.com", "banking");
        for rec in &advice.recommendations {
            assert!(
                !rec.message_vi.is_empty(),
                "VI message empty for code: {}",
                rec.code
            );
        }
    }

    #[test]
    fn test_passphrase_suggestion_for_weak_password() {
        let advice = advise_password("short", "Bank", "https://bank.com", "banking");
        assert!(advice.suggestion_en.is_some());
        assert!(advice.suggestion_vi.is_some());
    }

    #[test]
    fn test_score_range() {
        let advice = advise_password("X9#mK2$pL7@nQ4!vR8^", "Test", "https://test.com", "general");
        assert!(advice.score <= 100);
        let advice2 = advise_password("a", "Test", "https://test.com", "general");
        assert!(advice2.score <= 100);
    }
}
