//! Natural Language Query Parser
//!
//! Tokenizes and classifies natural language queries into structured intents.
//! Supports English and Vietnamese without external NLP dependencies.

use std::collections::HashMap;

/// Parsed natural language query
#[derive(Debug, Clone, PartialEq)]
pub struct NlQuery {
    /// Primary intent
    pub intent: NlIntent,
    /// Optional group filter
    pub group: Option<String>,
    /// Optional tag filter
    pub tags: Vec<String>,
    /// Optional time constraint
    pub time_filter: Option<TimeFilter>,
    /// Optional strength filter
    pub strength_filter: Option<StrengthFilter>,
    /// Optional feature filter (OTP, passkey, SSH, etc.)
    pub features: Vec<EntryFeature>,
    /// Free-text search term (remaining tokens)
    pub text: Option<String>,
    /// Detected language
    pub lang: Lang,
}

/// Primary search intent
#[derive(Debug, Clone, PartialEq)]
pub enum NlIntent {
    /// Show all entries (default)
    All,
    /// Entries with expired passwords
    Expired,
    /// Entries expiring soon
    ExpiringSoon { days: u32 },
    /// Entries with weak passwords
    Weak,
    /// Entries with reused passwords
    Reused,
    /// Entries with no password
    NoPassword,
    /// Entries found in breaches
    Breached,
    /// Entries with specific feature (OTP, passkey, SSH)
    WithFeature(EntryFeature),
    /// Entries created in time range
    CreatedIn(TimeFilter),
    /// Entries modified in time range
    ModifiedIn(TimeFilter),
    /// Entries not accessed in time range
    NotUsedIn(TimeFilter),
    /// Entries matching free text
    Search(String),
    /// Favorites
    Favorites,
    /// Recently used
    Recent,
}

/// Time filter
#[derive(Debug, Clone, PartialEq)]
pub enum TimeFilter {
    Today,
    Yesterday,
    ThisWeek,
    LastWeek,
    ThisMonth,
    LastMonth,
    LastNDays(u32),
    LastNMonths(u32),
    LastNYears(u32),
}

/// Password strength filter
#[derive(Debug, Clone, PartialEq)]
pub enum StrengthFilter {
    VeryWeak,
    Weak,
    Fair,
    Strong,
    VeryStrong,
}

/// Entry feature filter
#[derive(Debug, Clone, PartialEq)]
pub enum EntryFeature {
    Otp,
    Passkey,
    SshKey,
    Attachment,
    CustomField,
}

/// Detected language
#[derive(Debug, Clone, PartialEq)]
pub enum Lang {
    En,
    Vi,
    Unknown,
}

/// Parse a natural language query string into a structured `NlQuery`.
///
/// # Arguments
/// * `input` — Raw query string from user
///
/// # Returns
/// Parsed `NlQuery` with intent, filters, and metadata
pub fn parse_nl_query(input: &str) -> NlQuery {
    let normalized = input.trim().to_lowercase();
    let lang = detect_language(&normalized);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();

    let mut query = NlQuery {
        intent: NlIntent::All,
        group: None,
        tags: vec![],
        time_filter: None,
        strength_filter: None,
        features: vec![],
        text: None,
        lang: lang.clone(),
    };

    // Extract group filter: "in [group]" / "trong nhóm [group]"
    query.group = extract_group(&normalized, &lang);

    // Extract tag filter: "tagged [tag]" / "có thẻ [tag]"
    query.tags = extract_tags(&normalized, &lang);

    // Extract time filter
    query.time_filter = extract_time_filter(&normalized, &lang);

    // Extract feature filter
    query.features = extract_features(&normalized, &lang);

    // Extract strength filter
    query.strength_filter = extract_strength(&normalized, &lang);

    // Remaining text (after removing known keywords)
    let remaining = extract_remaining_text(&normalized, &lang);
    if !remaining.is_empty() {
        query.text = Some(remaining.clone());
    }

    // Determine primary intent (after setting text so it's available)
    query.intent = determine_intent(&normalized, &tokens, &lang, &query);

    query
}

// ─── Language Detection ───────────────────────────────────────────────────────

fn detect_language(input: &str) -> Lang {
    // Vietnamese indicators
    let vi_words = [
        "tìm",
        "mật khẩu",
        "kho",
        "mục",
        "nhóm",
        "hết hạn",
        "yếu",
        "tháng",
        "tuần",
        "ngày",
        "gần đây",
        "yêu thích",
        "có",
        "không",
        "trong",
    ];
    // English indicators
    let en_words = [
        "find", "show", "list", "password", "entry", "entries", "group", "expired", "weak",
        "month", "week", "day", "recent", "favorite", "with", "without", "in", "not",
    ];

    let vi_score: usize = vi_words.iter().filter(|&&w| input.contains(w)).count();
    let en_score: usize = en_words.iter().filter(|&&w| input.contains(w)).count();

    if vi_score > en_score {
        Lang::Vi
    } else if en_score > 0 {
        Lang::En
    } else {
        Lang::Unknown
    }
}

// ─── Intent Detection ─────────────────────────────────────────────────────────

fn determine_intent(input: &str, _tokens: &[&str], lang: &Lang, query: &NlQuery) -> NlIntent {
    // Check for specific intents in priority order

    // Expired
    if matches_any(input, &["expired", "hết hạn", "quá hạn"]) {
        return NlIntent::Expired;
    }

    // Expiring soon
    if matches_any(input, &["expiring", "sắp hết hạn", "sắp hạn"]) {
        let days = extract_number(input).unwrap_or(7);
        return NlIntent::ExpiringSoon { days };
    }

    // Weak passwords
    if matches_any(input, &["weak password", "mật khẩu yếu", "yếu"]) {
        return NlIntent::Weak;
    }

    // Reused passwords
    if matches_any(
        input,
        &[
            "reused",
            "duplicate password",
            "mật khẩu trùng",
            "trùng lặp",
        ],
    ) {
        return NlIntent::Reused;
    }

    // No password
    if matches_any(
        input,
        &[
            "no password",
            "without password",
            "không có mật khẩu",
            "không mật khẩu",
        ],
    ) {
        return NlIntent::NoPassword;
    }

    // Breached
    if matches_any(
        input,
        &["breached", "breach", "compromised", "rò rỉ", "bị lộ"],
    ) {
        return NlIntent::Breached;
    }

    // Favorites
    if matches_any(input, &["favorite", "starred", "yêu thích", "đánh dấu"]) {
        return NlIntent::Favorites;
    }

    // Recent
    if matches_any(input, &["recent", "recently used", "gần đây", "mới dùng"]) {
        return NlIntent::Recent;
    }

    // Feature-based
    if !query.features.is_empty() {
        return NlIntent::WithFeature(query.features[0].clone());
    }

    // Time-based: created
    if let Some(tf) = &query.time_filter {
        if matches_any(
            input,
            &["created", "added", "new", "tạo", "thêm", "mới tạo"],
        ) {
            return NlIntent::CreatedIn(tf.clone());
        }
        if matches_any(
            input,
            &[
                "modified",
                "changed",
                "updated",
                "sửa",
                "thay đổi",
                "cập nhật",
            ],
        ) {
            return NlIntent::ModifiedIn(tf.clone());
        }
        if matches_any(input, &["not used", "unused", "chưa dùng", "không dùng"]) {
            return NlIntent::NotUsedIn(tf.clone());
        }
        // Default time intent: created
        return NlIntent::CreatedIn(tf.clone());
    }

    // Free text search
    if let Some(text) = &query.text {
        if !text.is_empty() {
            return NlIntent::Search(text.clone());
        }
    }

    // Default: show all
    NlIntent::All
}

// ─── Extractors ───────────────────────────────────────────────────────────────

fn extract_group(input: &str, _lang: &Lang) -> Option<String> {
    // "in [group] group" / "in group [group]" / "trong nhóm [group]"
    let patterns = [
        ("in group ", 9),
        ("in the group ", 13),
        ("trong nhóm ", 11),
        ("nhóm ", 5),
    ];
    for (pattern, skip) in &patterns {
        if let Some(pos) = input.find(pattern) {
            let rest = &input[pos + skip..];
            let group = rest
                .split_whitespace()
                .take(3)
                .collect::<Vec<_>>()
                .join(" ");
            if !group.is_empty() {
                return Some(group);
            }
        }
    }
    None
}

fn extract_tags(input: &str, _lang: &Lang) -> Vec<String> {
    let mut tags = Vec::new();
    let patterns = ["tagged ", "with tag ", "có thẻ ", "thẻ "];
    for pattern in &patterns {
        if let Some(pos) = input.find(pattern) {
            let rest = &input[pos + pattern.len()..];
            let tag = rest.split_whitespace().next().unwrap_or("").to_string();
            if !tag.is_empty() {
                tags.push(tag);
            }
        }
    }
    tags
}

fn extract_time_filter(input: &str, _lang: &Lang) -> Option<TimeFilter> {
    // Today
    if matches_any(input, &["today", "hôm nay"]) {
        return Some(TimeFilter::Today);
    }
    // Yesterday
    if matches_any(input, &["yesterday", "hôm qua"]) {
        return Some(TimeFilter::Yesterday);
    }
    // This week
    if matches_any(input, &["this week", "tuần này"]) {
        return Some(TimeFilter::ThisWeek);
    }
    // Last week
    if matches_any(input, &["last week", "tuần trước"]) {
        return Some(TimeFilter::LastWeek);
    }
    // This month
    if matches_any(input, &["this month", "tháng này"]) {
        return Some(TimeFilter::ThisMonth);
    }
    // Last month
    if matches_any(input, &["last month", "tháng trước"]) {
        return Some(TimeFilter::LastMonth);
    }
    // Last N days
    if let Some(n) = extract_number_before(input, &["day", "days", "ngày"]) {
        return Some(TimeFilter::LastNDays(n));
    }
    // Last N months
    if let Some(n) = extract_number_before(input, &["month", "months", "tháng"]) {
        return Some(TimeFilter::LastNMonths(n));
    }
    // Last N years
    if let Some(n) = extract_number_before(input, &["year", "years", "năm"]) {
        return Some(TimeFilter::LastNYears(n));
    }
    None
}

fn extract_features(input: &str, _lang: &Lang) -> Vec<EntryFeature> {
    let mut features = Vec::new();
    if matches_any(
        input,
        &["otp", "totp", "2fa", "two-factor", "xác thực 2 bước"],
    ) {
        features.push(EntryFeature::Otp);
    }
    if matches_any(input, &["passkey", "webauthn", "fido2", "khóa thông hành"]) {
        features.push(EntryFeature::Passkey);
    }
    if matches_any(input, &["ssh", "ssh key", "khóa ssh"]) {
        features.push(EntryFeature::SshKey);
    }
    if matches_any(input, &["attachment", "file", "tệp đính kèm"]) {
        features.push(EntryFeature::Attachment);
    }
    features
}

fn extract_strength(input: &str, _lang: &Lang) -> Option<StrengthFilter> {
    if matches_any(input, &["very weak", "rất yếu"]) {
        return Some(StrengthFilter::VeryWeak);
    }
    if matches_any(input, &["weak", "yếu"]) {
        return Some(StrengthFilter::Weak);
    }
    if matches_any(input, &["fair", "medium", "trung bình"]) {
        return Some(StrengthFilter::Fair);
    }
    if matches_any(input, &["strong", "mạnh"]) {
        return Some(StrengthFilter::Strong);
    }
    if matches_any(input, &["very strong", "rất mạnh"]) {
        return Some(StrengthFilter::VeryStrong);
    }
    None
}

fn extract_remaining_text(input: &str, _lang: &Lang) -> String {
    // Remove known keywords and return remaining meaningful text
    let stop_words = [
        "show",
        "find",
        "list",
        "get",
        "display",
        "all",
        "entries",
        "entry",
        "passwords",
        "password",
        "with",
        "without",
        "in",
        "the",
        "a",
        "an",
        "that",
        "are",
        "is",
        "have",
        "has",
        "been",
        "and",
        "or",
        "not",
        "tìm",
        "hiển thị",
        "danh sách",
        "tất cả",
        "mục",
        "mật khẩu",
        "có",
        "không",
        "trong",
        "và",
        "hoặc",
    ];
    let words: Vec<&str> = input
        .split_whitespace()
        .filter(|w| !stop_words.contains(w))
        .collect();
    words.join(" ")
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn matches_any(input: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|&p| input.contains(p))
}

fn extract_number(input: &str) -> Option<u32> {
    // Extract first number found in string
    let mut num_str = String::new();
    for ch in input.chars() {
        if ch.is_ascii_digit() {
            num_str.push(ch);
        } else if !num_str.is_empty() {
            break;
        }
    }
    num_str.parse().ok()
}

fn extract_number_before(input: &str, units: &[&str]) -> Option<u32> {
    // Find "N unit" pattern (e.g., "6 months", "30 days")
    for unit in units {
        if let Some(pos) = input.find(unit) {
            // Look backwards for a number
            let before = &input[..pos].trim_end();
            let num_str: String = before
                .chars()
                .rev()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if let Ok(n) = num_str.parse::<u32>() {
                return Some(n);
            }
            // Word numbers
            let word_nums: HashMap<&str, u32> = [
                ("one", 1),
                ("two", 2),
                ("three", 3),
                ("four", 4),
                ("five", 5),
                ("six", 6),
                ("seven", 7),
                ("eight", 8),
                ("nine", 9),
                ("ten", 10),
                ("một", 1),
                ("hai", 2),
                ("ba", 3),
                ("bốn", 4),
                ("năm", 5),
                ("sáu", 6),
                ("bảy", 7),
                ("tám", 8),
                ("chín", 9),
                ("mười", 10),
            ]
            .iter()
            .cloned()
            .collect();
            for (word, val) in &word_nums {
                if before.ends_with(word) {
                    return Some(*val);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expired_en() {
        let q = parse_nl_query("show expired entries");
        assert_eq!(q.intent, NlIntent::Expired);
        assert_eq!(q.lang, Lang::En);
    }

    #[test]
    fn test_parse_expired_vi() {
        let q = parse_nl_query("tìm mục hết hạn");
        assert_eq!(q.intent, NlIntent::Expired);
        assert_eq!(q.lang, Lang::Vi);
    }

    #[test]
    fn test_parse_weak_passwords() {
        let q = parse_nl_query("find entries with weak password");
        assert_eq!(q.intent, NlIntent::Weak);
    }

    #[test]
    fn test_parse_otp_entries() {
        let q = parse_nl_query("list all entries with OTP");
        assert_eq!(q.intent, NlIntent::WithFeature(EntryFeature::Otp));
        assert!(q.features.contains(&EntryFeature::Otp));
    }

    #[test]
    fn test_parse_last_month() {
        let q = parse_nl_query("entries created last month");
        assert!(matches!(
            q.intent,
            NlIntent::CreatedIn(TimeFilter::LastMonth)
        ));
    }

    #[test]
    fn test_parse_last_n_months() {
        let q = parse_nl_query("passwords not used in 6 months");
        assert!(matches!(
            q.intent,
            NlIntent::NotUsedIn(TimeFilter::LastNMonths(6))
        ));
    }

    #[test]
    fn test_parse_group_filter() {
        let q = parse_nl_query("show entries in group Banking");
        assert_eq!(q.group, Some("banking".to_string()));
    }

    #[test]
    fn test_parse_favorites() {
        let q = parse_nl_query("show my favorites");
        assert_eq!(q.intent, NlIntent::Favorites);
    }

    #[test]
    fn test_parse_breached() {
        let q = parse_nl_query("find breached passwords");
        assert_eq!(q.intent, NlIntent::Breached);
    }

    #[test]
    fn test_parse_vi_otp() {
        let q = parse_nl_query("mục có OTP trong nhóm Công việc");
        assert!(q.features.contains(&EntryFeature::Otp));
        assert_eq!(q.lang, Lang::Vi);
    }

    #[test]
    fn test_parse_expiring_soon() {
        let q = parse_nl_query("entries expiring in 7 days");
        assert!(matches!(q.intent, NlIntent::ExpiringSoon { days: 7 }));
    }

    #[test]
    fn test_parse_reused() {
        let q = parse_nl_query("show reused passwords");
        assert_eq!(q.intent, NlIntent::Reused);
    }

    #[test]
    fn test_parse_no_password() {
        let q = parse_nl_query("entries without password");
        assert_eq!(q.intent, NlIntent::NoPassword);
    }

    #[test]
    fn test_parse_ssh_key() {
        let q = parse_nl_query("find entries with SSH key");
        assert!(q.features.contains(&EntryFeature::SshKey));
    }

    #[test]
    fn test_parse_this_week() {
        let q = parse_nl_query("entries modified this week");
        assert!(matches!(
            q.intent,
            NlIntent::ModifiedIn(TimeFilter::ThisWeek)
        ));
    }

    #[test]
    fn test_parse_free_text() {
        let q = parse_nl_query("github");
        assert!(matches!(q.intent, NlIntent::Search(_)));
    }

    #[test]
    fn test_parse_empty() {
        let q = parse_nl_query("");
        assert_eq!(q.intent, NlIntent::All);
    }
}
