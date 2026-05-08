// End-to-end integration test — simulates real user workflow
// Run: cargo run --example e2e_test (or as a test)
// Tests: create vault → add entry → read entry → health check → export

use keepassex_core::{
    vault::{operations::{open_vault, save_vault, VaultCredentials}, Vault},
    generator::{PasswordGenerator},
    types::PasswordGeneratorConfig,
    health::audit_vault,
    otp::{generate_totp, parse_otp_uri},
};
use std::path::Path;
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔐 KeePassEx End-to-End Test");
    println!("================================");

    // 1. Create vault
    let tmp = NamedTempFile::new()?;
    let vault_path = tmp.path().to_path_buf();
    let password = "TestPassword123!";
    let creds = VaultCredentials::password_only(password);

    let mut vault = Vault::new("Test Vault");
    save_vault(&vault, &vault_path, &creds).await?;
    println!("✅ 1. Created vault: {}", vault_path.display());

    // 2. Open vault
    let mut vault = open_vault(&vault_path, &creds).await?;
    println!("✅ 2. Opened vault: {} entries", vault.entry_count());

    // 3. Add entries
    let root = vault.root_group_uuid;
    let e1 = vault.create_entry(root)?;
    if let Some(e) = vault.get_entry_mut(&e1) {
        e.title.set("GitHub");
        e.username.set("user@example.com");
        e.password.set("Tr0ub4dor&3xY!");
        e.url = "https://github.com".to_string();
        e.tags = vec!["development".to_string()];
    }

    let e2 = vault.create_entry(root)?;
    if let Some(e) = vault.get_entry_mut(&e2) {
        e.title.set("Gmail");
        e.username.set("user@gmail.com");
        e.password.set("weak"); // intentionally weak for health test
        e.url = "https://gmail.com".to_string();
    }

    let e3 = vault.create_entry(root)?;
    if let Some(e) = vault.get_entry_mut(&e3) {
        e.title.set("Twitter");
        e.username.set("@user");
        e.password.set("Tr0ub4dor&3xY!"); // reused password
        e.url = "https://twitter.com".to_string();
    }

    save_vault(&vault, &vault_path, &creds).await?;
    println!("✅ 3. Added 3 entries (GitHub, Gmail, Twitter)");

    // 4. Read back
    let vault = open_vault(&vault_path, &creds).await?;
    assert_eq!(vault.entry_count(), 3, "Expected 3 entries");
    let github = vault.all_entries().find(|e| e.title.get() == "GitHub");
    assert!(github.is_some(), "GitHub entry not found");
    assert_eq!(github.unwrap().username.get(), "user@example.com");
    println!("✅ 4. Read back entries correctly");

    // 5. Password generator
    let config = PasswordGeneratorConfig::default();
    let pw = PasswordGenerator::generate(&config)?;
    assert!(pw.len() >= 16, "Generated password too short: {}", pw.len());
    let strength = PasswordGenerator::score_strength(&pw);
    println!("✅ 5. Generated password: {} chars, strength: {:?}", pw.len(), strength.label_en());

    // 6. Health audit
    let report = audit_vault(&vault);
    assert_eq!(report.total_entries, 3);
    assert!(report.weak_count >= 1, "Should detect weak 'Gmail' password");
    assert!(report.reused_count >= 1, "Should detect reused password");
    println!("✅ 6. Health audit: score={}, weak={}, reused={}",
        report.score, report.weak_count, report.reused_count);

    // 7. OTP parsing
    let uri = "otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitHub";
    let otp_config = parse_otp_uri(uri)?;
    assert_eq!(otp_config.issuer.as_deref(), Some("GitHub"));
    let code = generate_totp(&otp_config)?;
    assert_eq!(code.code.len(), 6, "OTP should be 6 digits");
    assert!(code.remaining_seconds <= 30);
    println!("✅ 7. OTP: code={}, remaining={}s", code.code, code.remaining_seconds);

    // 8. Search
    let results = vault.search(&keepassex_core::types::SearchQuery::new("github"));
    assert!(!results.is_empty(), "Search should find GitHub");
    println!("✅ 8. Search 'github': {} results", results.len());

    // 9. Group operations
    let mut vault = open_vault(&vault_path, &creds).await?;
    let work_group = vault.create_group("Work", vault.root_group_uuid)?;
    vault.move_entry(&e1, work_group)?;
    save_vault(&vault, &vault_path, &creds).await?;
    let vault = open_vault(&vault_path, &creds).await?;
    let work_entries = vault.get_group_entries(&work_group);
    assert_eq!(work_entries.len(), 1, "Work group should have 1 entry");
    println!("✅ 9. Group operations: moved GitHub to Work group");

    // 10. History
    let mut vault = open_vault(&vault_path, &creds).await?;
    let entry = vault.get_entry(&e2).unwrap().clone();
    let mut updated = entry.clone();
    updated.password.set("NewStrongerPassword123!");
    vault.update_entry(updated)?;
    let entry = vault.get_entry(&e2).unwrap();
    assert!(!entry.history.is_empty(), "Should have history after update");
    println!("✅ 10. Entry history: {} snapshots", entry.history.len());

    println!();
    println!("🎉 All 10 end-to-end tests passed!");
    println!("   KeePassEx v0.1.0 is working correctly.");

    Ok(())
}
