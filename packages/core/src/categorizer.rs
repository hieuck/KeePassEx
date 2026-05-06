//! Smart Entry Categorizer — auto-categorize entries by URL/title
//!
//! Analyzes entry URL and title to suggest the best group and tags.
//! Runs entirely on-device — no network calls.
//!
//! # Categories
//! Banking, Social, Email, Shopping, Development, Gaming, Work, Health,
//! Entertainment, Travel, Government, Education, Crypto, Cloud, Security
//!
//! # Usage
//! ```no_run
//! use keepassex_core::categorizer::{categorize_entry, EntryCategory};
//! let cat = categorize_entry("Chase Bank", "https://chase.com", "");
//! assert_eq!(cat.category, EntryCategory::Banking);
//! ```

use std::collections::HashMap;

/// Entry category
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntryCategory {
    Banking,
    Social,
    Email,
    Shopping,
    Development,
    Gaming,
    Work,
    Health,
    Entertainment,
    Travel,
    Government,
    Education,
    Crypto,
    Cloud,
    Security,
    Other,
}

impl EntryCategory {
    pub fn display_en(&self) -> &'static str {
        match self {
            Self::Banking => "Banking & Finance",
            Self::Social => "Social Media",
            Self::Email => "Email",
            Self::Shopping => "Shopping",
            Self::Development => "Development",
            Self::Gaming => "Gaming",
            Self::Work => "Work",
            Self::Health => "Health",
            Self::Entertainment => "Entertainment",
            Self::Travel => "Travel",
            Self::Government => "Government",
            Self::Education => "Education",
            Self::Crypto => "Cryptocurrency",
            Self::Cloud => "Cloud Storage",
            Self::Security => "Security",
            Self::Other => "Other",
        }
    }

    pub fn display_vi(&self) -> &'static str {
        match self {
            Self::Banking => "Ngân hàng & Tài chính",
            Self::Social => "Mạng xã hội",
            Self::Email => "Email",
            Self::Shopping => "Mua sắm",
            Self::Development => "Lập trình",
            Self::Gaming => "Game",
            Self::Work => "Công việc",
            Self::Health => "Sức khỏe",
            Self::Entertainment => "Giải trí",
            Self::Travel => "Du lịch",
            Self::Government => "Chính phủ",
            Self::Education => "Giáo dục",
            Self::Crypto => "Tiền điện tử",
            Self::Cloud => "Lưu trữ đám mây",
            Self::Security => "Bảo mật",
            Self::Other => "Khác",
        }
    }

    pub fn icon_id(&self) -> u32 {
        match self {
            Self::Banking => 10,
            Self::Social => 11,
            Self::Email => 8,
            Self::Shopping => 9,
            Self::Development => 5,
            Self::Gaming => 6,
            Self::Work => 1,
            Self::Health => 7,
            Self::Entertainment => 6,
            Self::Travel => 3,
            Self::Government => 2,
            Self::Education => 4,
            Self::Crypto => 9,
            Self::Cloud => 5,
            Self::Security => 0,
            Self::Other => 1,
        }
    }
}

/// Result of categorizing an entry
#[derive(Debug, Clone)]
pub struct CategorizationResult {
    pub category: EntryCategory,
    /// Suggested tags based on domain/title
    pub suggested_tags: Vec<String>,
    /// Confidence score 0.0-1.0
    pub confidence: f32,
    /// Suggested group name (EN)
    pub group_name_en: String,
    /// Suggested group name (VI)
    pub group_name_vi: String,
}

/// Categorize an entry based on title, URL, and notes
pub fn categorize_entry(title: &str, url: &str, notes: &str) -> CategorizationResult {
    let title_lower = title.to_lowercase();
    let url_lower = url.to_lowercase();
    let notes_lower = notes.to_lowercase();
    let combined = format!("{} {} {}", title_lower, url_lower, notes_lower);

    // Extract domain for matching
    let domain = extract_domain(url);

    // Try domain-based matching first (highest confidence)
    if let Some(result) = match_by_domain(&domain) {
        return result;
    }

    // Fall back to keyword matching
    match_by_keywords(&combined, &title_lower)
}

/// Batch categorize multiple entries
pub fn categorize_entries(entries: &[(&str, &str, &str)]) -> Vec<CategorizationResult> {
    entries
        .iter()
        .map(|(title, url, notes)| categorize_entry(title, url, notes))
        .collect()
}

// ─── Domain Database ──────────────────────────────────────────────────────────

fn match_by_domain(domain: &str) -> Option<CategorizationResult> {
    let domain_map: &[(&str, EntryCategory, &[&str])] = &[
        // Banking
        ("chase.com", EntryCategory::Banking, &["bank", "finance"]),
        (
            "bankofamerica.com",
            EntryCategory::Banking,
            &["bank", "finance"],
        ),
        (
            "wellsfargo.com",
            EntryCategory::Banking,
            &["bank", "finance"],
        ),
        ("citibank.com", EntryCategory::Banking, &["bank", "finance"]),
        (
            "paypal.com",
            EntryCategory::Banking,
            &["payment", "finance"],
        ),
        (
            "stripe.com",
            EntryCategory::Banking,
            &["payment", "finance"],
        ),
        ("venmo.com", EntryCategory::Banking, &["payment"]),
        (
            "vietcombank.com.vn",
            EntryCategory::Banking,
            &["bank", "vietnam"],
        ),
        (
            "techcombank.com.vn",
            EntryCategory::Banking,
            &["bank", "vietnam"],
        ),
        (
            "vpbank.com.vn",
            EntryCategory::Banking,
            &["bank", "vietnam"],
        ),
        ("momo.vn", EntryCategory::Banking, &["payment", "vietnam"]),
        // Social
        ("facebook.com", EntryCategory::Social, &["social", "meta"]),
        (
            "instagram.com",
            EntryCategory::Social,
            &["social", "meta", "photo"],
        ),
        ("twitter.com", EntryCategory::Social, &["social"]),
        ("x.com", EntryCategory::Social, &["social"]),
        (
            "linkedin.com",
            EntryCategory::Social,
            &["social", "professional"],
        ),
        ("tiktok.com", EntryCategory::Social, &["social", "video"]),
        ("reddit.com", EntryCategory::Social, &["social", "forum"]),
        (
            "discord.com",
            EntryCategory::Social,
            &["social", "gaming", "chat"],
        ),
        (
            "zalo.me",
            EntryCategory::Social,
            &["social", "vietnam", "chat"],
        ),
        // Email
        ("gmail.com", EntryCategory::Email, &["email", "google"]),
        ("outlook.com", EntryCategory::Email, &["email", "microsoft"]),
        ("yahoo.com", EntryCategory::Email, &["email"]),
        (
            "protonmail.com",
            EntryCategory::Email,
            &["email", "privacy"],
        ),
        ("fastmail.com", EntryCategory::Email, &["email"]),
        // Shopping
        ("amazon.com", EntryCategory::Shopping, &["shopping", "aws"]),
        ("ebay.com", EntryCategory::Shopping, &["shopping"]),
        (
            "shopify.com",
            EntryCategory::Shopping,
            &["shopping", "ecommerce"],
        ),
        (
            "lazada.vn",
            EntryCategory::Shopping,
            &["shopping", "vietnam"],
        ),
        (
            "shopee.vn",
            EntryCategory::Shopping,
            &["shopping", "vietnam"],
        ),
        ("tiki.vn", EntryCategory::Shopping, &["shopping", "vietnam"]),
        // Development
        (
            "github.com",
            EntryCategory::Development,
            &["dev", "git", "code"],
        ),
        (
            "gitlab.com",
            EntryCategory::Development,
            &["dev", "git", "code"],
        ),
        ("bitbucket.org", EntryCategory::Development, &["dev", "git"]),
        (
            "stackoverflow.com",
            EntryCategory::Development,
            &["dev", "forum"],
        ),
        ("npmjs.com", EntryCategory::Development, &["dev", "nodejs"]),
        (
            "docker.com",
            EntryCategory::Development,
            &["dev", "containers"],
        ),
        ("aws.amazon.com", EntryCategory::Cloud, &["cloud", "aws"]),
        ("cloud.google.com", EntryCategory::Cloud, &["cloud", "gcp"]),
        (
            "azure.microsoft.com",
            EntryCategory::Cloud,
            &["cloud", "azure"],
        ),
        // Gaming
        ("steam.com", EntryCategory::Gaming, &["gaming", "steam"]),
        ("epicgames.com", EntryCategory::Gaming, &["gaming"]),
        ("battle.net", EntryCategory::Gaming, &["gaming", "blizzard"]),
        (
            "playstation.com",
            EntryCategory::Gaming,
            &["gaming", "sony"],
        ),
        ("xbox.com", EntryCategory::Gaming, &["gaming", "microsoft"]),
        // Entertainment
        (
            "netflix.com",
            EntryCategory::Entertainment,
            &["streaming", "video"],
        ),
        (
            "spotify.com",
            EntryCategory::Entertainment,
            &["music", "streaming"],
        ),
        (
            "youtube.com",
            EntryCategory::Entertainment,
            &["video", "google"],
        ),
        (
            "disney.com",
            EntryCategory::Entertainment,
            &["streaming", "video"],
        ),
        // Crypto
        (
            "coinbase.com",
            EntryCategory::Crypto,
            &["crypto", "bitcoin"],
        ),
        (
            "binance.com",
            EntryCategory::Crypto,
            &["crypto", "exchange"],
        ),
        ("kraken.com", EntryCategory::Crypto, &["crypto", "exchange"]),
        // Cloud Storage
        ("dropbox.com", EntryCategory::Cloud, &["cloud", "storage"]),
        (
            "drive.google.com",
            EntryCategory::Cloud,
            &["cloud", "google"],
        ),
        (
            "onedrive.live.com",
            EntryCategory::Cloud,
            &["cloud", "microsoft"],
        ),
        ("icloud.com", EntryCategory::Cloud, &["cloud", "apple"]),
        // Security
        (
            "1password.com",
            EntryCategory::Security,
            &["security", "password"],
        ),
        (
            "lastpass.com",
            EntryCategory::Security,
            &["security", "password"],
        ),
        ("authy.com", EntryCategory::Security, &["security", "2fa"]),
    ];

    for (pattern, category, tags) in domain_map {
        if domain.contains(pattern) || domain.ends_with(pattern) {
            let group_en = category.display_en().to_string();
            let group_vi = category.display_vi().to_string();
            return Some(CategorizationResult {
                category: category.clone(),
                suggested_tags: tags.iter().map(|s| s.to_string()).collect(),
                confidence: 0.95,
                group_name_en: group_en,
                group_name_vi: group_vi,
            });
        }
    }
    None
}

fn match_by_keywords(combined: &str, title: &str) -> CategorizationResult {
    let keyword_map: &[(&[&str], EntryCategory, f32)] = &[
        (
            &[
                "bank",
                "credit",
                "debit",
                "loan",
                "mortgage",
                "invest",
                "finance",
                "ngân hàng",
                "tài chính",
            ],
            EntryCategory::Banking,
            0.8,
        ),
        (
            &[
                "facebook",
                "twitter",
                "instagram",
                "social",
                "linkedin",
                "mạng xã hội",
            ],
            EntryCategory::Social,
            0.8,
        ),
        (
            &["email", "mail", "inbox", "smtp", "imap"],
            EntryCategory::Email,
            0.85,
        ),
        (
            &[
                "shop",
                "store",
                "buy",
                "order",
                "cart",
                "amazon",
                "ebay",
                "mua sắm",
            ],
            EntryCategory::Shopping,
            0.75,
        ),
        (
            &[
                "github",
                "gitlab",
                "code",
                "dev",
                "api",
                "server",
                "deploy",
                "lập trình",
            ],
            EntryCategory::Development,
            0.8,
        ),
        (
            &[
                "game",
                "steam",
                "play",
                "xbox",
                "playstation",
                "gaming",
                "game",
            ],
            EntryCategory::Gaming,
            0.8,
        ),
        (
            &["work", "office", "slack", "jira", "confluence", "công việc"],
            EntryCategory::Work,
            0.7,
        ),
        (
            &[
                "health",
                "doctor",
                "hospital",
                "medical",
                "pharmacy",
                "sức khỏe",
            ],
            EntryCategory::Health,
            0.8,
        ),
        (
            &[
                "netflix",
                "spotify",
                "youtube",
                "music",
                "video",
                "stream",
                "giải trí",
            ],
            EntryCategory::Entertainment,
            0.75,
        ),
        (
            &["flight", "hotel", "travel", "booking", "airbnb", "du lịch"],
            EntryCategory::Travel,
            0.8,
        ),
        (
            &["gov", "government", "tax", "irs", "dmv", "chính phủ"],
            EntryCategory::Government,
            0.85,
        ),
        (
            &["university", "school", "edu", "course", "learn", "giáo dục"],
            EntryCategory::Education,
            0.8,
        ),
        (
            &[
                "bitcoin",
                "crypto",
                "ethereum",
                "wallet",
                "blockchain",
                "tiền điện tử",
            ],
            EntryCategory::Crypto,
            0.85,
        ),
        (
            &["cloud", "storage", "backup", "drive", "dropbox", "đám mây"],
            EntryCategory::Cloud,
            0.75,
        ),
        (
            &["vpn", "security", "password", "2fa", "auth", "bảo mật"],
            EntryCategory::Security,
            0.75,
        ),
    ];

    let mut best_category = EntryCategory::Other;
    let mut best_confidence = 0.3f32;
    let mut best_tags: Vec<String> = vec![];

    for (keywords, category, base_confidence) in keyword_map {
        let matches = keywords.iter().filter(|&&kw| combined.contains(kw)).count();
        if matches > 0 {
            let confidence = base_confidence * (1.0 + (matches as f32 - 1.0) * 0.1).min(1.0);
            if confidence > best_confidence {
                best_confidence = confidence;
                best_category = category.clone();
                best_tags = keywords
                    .iter()
                    .filter(|&&kw| combined.contains(kw))
                    .map(|s| s.to_string())
                    .take(3)
                    .collect();
            }
        }
    }

    let group_en = best_category.display_en().to_string();
    let group_vi = best_category.display_vi().to_string();

    CategorizationResult {
        category: best_category,
        suggested_tags: best_tags,
        confidence: best_confidence,
        group_name_en: group_en,
        group_name_vi: group_vi,
    }
}

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
        .map(|h| h.trim_start_matches("www.").to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_banking_by_domain() {
        let r = categorize_entry("Chase Bank", "https://chase.com", "");
        assert_eq!(r.category, EntryCategory::Banking);
        assert!(r.confidence > 0.9);
        assert!(r.suggested_tags.contains(&"bank".to_string()));
    }

    #[test]
    fn test_categorize_social_by_domain() {
        let r = categorize_entry("Facebook", "https://facebook.com", "");
        assert_eq!(r.category, EntryCategory::Social);
        assert!(r.confidence > 0.9);
    }

    #[test]
    fn test_categorize_development_github() {
        let r = categorize_entry("GitHub", "https://github.com", "");
        assert_eq!(r.category, EntryCategory::Development);
        assert!(r.suggested_tags.contains(&"dev".to_string()));
    }

    #[test]
    fn test_categorize_email_gmail() {
        let r = categorize_entry("Gmail", "https://gmail.com", "");
        assert_eq!(r.category, EntryCategory::Email);
    }

    #[test]
    fn test_categorize_by_keyword_fallback() {
        let r = categorize_entry("My Bank Account", "", "");
        assert_eq!(r.category, EntryCategory::Banking);
    }

    #[test]
    fn test_categorize_vietnam_bank() {
        let r = categorize_entry("Vietcombank", "https://vietcombank.com.vn", "");
        assert_eq!(r.category, EntryCategory::Banking);
        assert!(r.suggested_tags.contains(&"vietnam".to_string()));
    }

    #[test]
    fn test_categorize_vietnam_shopping() {
        let r = categorize_entry("Shopee", "https://shopee.vn", "");
        assert_eq!(r.category, EntryCategory::Shopping);
        assert!(r.suggested_tags.contains(&"vietnam".to_string()));
    }

    #[test]
    fn test_categorize_unknown_returns_other() {
        let r = categorize_entry("My Random Site", "https://randomsite12345.xyz", "");
        assert_eq!(r.category, EntryCategory::Other);
    }

    #[test]
    fn test_category_display_en() {
        assert_eq!(EntryCategory::Banking.display_en(), "Banking & Finance");
        assert_eq!(EntryCategory::Development.display_en(), "Development");
    }

    #[test]
    fn test_category_display_vi() {
        assert_eq!(EntryCategory::Banking.display_vi(), "Ngân hàng & Tài chính");
        assert_eq!(EntryCategory::Social.display_vi(), "Mạng xã hội");
    }

    #[test]
    fn test_batch_categorize() {
        let entries = vec![
            ("GitHub", "https://github.com", ""),
            ("Gmail", "https://gmail.com", ""),
            ("Chase", "https://chase.com", ""),
        ];
        let results = categorize_entries(&entries);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].category, EntryCategory::Development);
        assert_eq!(results[1].category, EntryCategory::Email);
        assert_eq!(results[2].category, EntryCategory::Banking);
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.github.com/user"), "github.com");
        assert_eq!(extract_domain("https://mail.google.com"), "mail.google.com");
        assert_eq!(extract_domain(""), "");
    }

    #[test]
    fn test_crypto_category() {
        let r = categorize_entry("Coinbase", "https://coinbase.com", "");
        assert_eq!(r.category, EntryCategory::Crypto);
    }

    #[test]
    fn test_gaming_category() {
        let r = categorize_entry("Steam", "https://steam.com", "");
        assert_eq!(r.category, EntryCategory::Gaming);
    }
}
