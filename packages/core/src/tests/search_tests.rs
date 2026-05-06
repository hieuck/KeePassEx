//! Search module tests — uses the vault's search() method and SearchQuery type

use crate::types::SearchQuery;
use crate::vault::Vault;

fn vault_with_entries() -> Vault {
    let mut vault = Vault::new("Search Test");
    let root = vault.root_group_uuid;

    let banking = vault.create_group("Banking", root).unwrap();
    let social = vault.create_group("Social", root).unwrap();

    // Banking entries
    let e1 = vault.create_entry(banking).unwrap();
    {
        let e = vault.get_entry_mut(&e1).unwrap();
        e.title.set("Chase Bank");
        e.username.set("john@example.com");
        e.url = "https://chase.com".to_string();
        e.password.set("BankP@ss1!");
        e.tags = vec!["finance".to_string(), "important".to_string()];
    }

    let e2 = vault.create_entry(banking).unwrap();
    {
        let e = vault.get_entry_mut(&e2).unwrap();
        e.title.set("PayPal");
        e.username.set("john@example.com");
        e.url = "https://paypal.com".to_string();
        e.password.set("PayP@ss2!");
    }

    // Social entries
    let e3 = vault.create_entry(social).unwrap();
    {
        let e = vault.get_entry_mut(&e3).unwrap();
        e.title.set("GitHub");
        e.username.set("johndoe");
        e.url = "https://github.com".to_string();
        e.password.set("GitP@ss3!");
        e.tags = vec!["dev".to_string()];
    }

    let e4 = vault.create_entry(social).unwrap();
    {
        let e = vault.get_entry_mut(&e4).unwrap();
        e.title.set("Twitter");
        e.username.set("@johndoe");
        e.url = "https://twitter.com".to_string();
        e.password.set("TwitP@ss4!");
    }

    vault
}

#[test]
fn test_search_by_title() {
    let vault = vault_with_entries();
    let q = SearchQuery::new("bank");
    let results = vault.search(&q);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.get(), "Chase Bank");
}

#[test]
fn test_search_case_insensitive() {
    let vault = vault_with_entries();
    let q = SearchQuery::new("GITHUB");
    let results = vault.search(&q);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.get(), "GitHub");
}

#[test]
fn test_search_by_url() {
    let vault = vault_with_entries();
    let mut q = SearchQuery::new("paypal.com");
    q.search_title = false;
    q.search_username = false;
    q.search_url = true;
    let results = vault.search(&q);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.get(), "PayPal");
}

#[test]
fn test_search_by_username() {
    let vault = vault_with_entries();
    let mut q = SearchQuery::new("john@example.com");
    q.search_title = false;
    q.search_username = true;
    q.search_url = false;
    let results = vault.search(&q);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_search_no_results() {
    let vault = vault_with_entries();
    let q = SearchQuery::new("zzz_nonexistent_zzz");
    let results = vault.search(&q);
    assert!(results.is_empty());
}

#[test]
fn test_search_partial_match() {
    let vault = vault_with_entries();
    let q = SearchQuery::new("git");
    let results = vault.search(&q);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.get(), "GitHub");
}

#[test]
fn test_search_multiple_matches() {
    let vault = vault_with_entries();
    // "john" appears in usernames of Chase Bank and PayPal
    let mut q = SearchQuery::new("john");
    q.search_title = false;
    q.search_username = true;
    q.search_url = false;
    let results = vault.search(&q);
    assert!(results.len() >= 2);
}

#[test]
fn test_search_empty_vault() {
    let vault = Vault::new("Empty");
    let q = SearchQuery::new("anything");
    let results = vault.search(&q);
    assert!(results.is_empty());
}

#[test]
fn test_search_query_defaults() {
    let q = SearchQuery::new("test");
    assert!(q.search_title);
    assert!(q.search_username);
    assert!(q.search_url);
    assert!(q.search_tags);
    assert!(q.recursive);
    assert!(!q.case_sensitive);
}

#[test]
fn test_search_twitter_by_title() {
    let vault = vault_with_entries();
    let q = SearchQuery::new("twitter");
    let results = vault.search(&q);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title.get(), "Twitter");
}
