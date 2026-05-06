//! Categorizer tests — uses the public categorize_entry() API

use crate::categorizer::{categorize_entries, categorize_entry, EntryCategory};

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
fn test_categorize_shopping_amazon() {
    let r = categorize_entry("Amazon", "https://amazon.com", "");
    assert_eq!(r.category, EntryCategory::Shopping);
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
fn test_categorize_no_url_falls_back_to_other() {
    let r = categorize_entry("Secure Note", "", "");
    assert_eq!(r.category, EntryCategory::Other);
}

#[test]
fn test_categorize_confidence_is_positive() {
    let r = categorize_entry("Chase", "https://chase.com", "");
    assert!(r.confidence > 0.0);
    assert!(r.confidence <= 1.0);
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
fn test_crypto_category() {
    let r = categorize_entry("Coinbase", "https://coinbase.com", "");
    assert_eq!(r.category, EntryCategory::Crypto);
}

#[test]
fn test_gaming_category() {
    let r = categorize_entry("Steam", "https://steam.com", "");
    assert_eq!(r.category, EntryCategory::Gaming);
}

#[test]
fn test_group_name_en_and_vi_populated() {
    let r = categorize_entry("Chase", "https://chase.com", "");
    assert!(!r.group_name_en.is_empty());
    assert!(!r.group_name_vi.is_empty());
}
