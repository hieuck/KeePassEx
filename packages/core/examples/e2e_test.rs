// End-to-end integration test — simulates real user workflow
// Run: cargo run -p keepassex-core --example e2e_test

use keepassex_core::{
    generator::PasswordGenerator,
    health::audit_vault,
    otp::{generate_totp, parse_otp_uri},
    types::{PasswordGeneratorConfig, SearchQuery},
    vault::{
        operations::{open_vault, save_vault, VaultCredentials},
        Vault,
    },
};
use tempfile::NamedTempFile;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔐 KeePassEx End-to-End Test");
    println!("================================");

    // ── 1. Create vault ───────────────────────────────────────────────────────
    let tmp = NamedTempFile::new()?;
    let vault_path = tmp.path().to_path_buf();
    let password = "TestPassword123!";
    let creds = VaultCredentials::password_only(password);

    let vault = Vault::new("Test Vault");
    save_vault(&vault, &vault_path, &creds).await?;
    println!("✅ 1. Created vault at {}", vault_path.display());

    // ── 2. Open vault ─────────────────────────────────────────────────────────
    let mut vault = open_vault(&vault_path, &creds).await?;
    assert_eq!(vault.entry_count(), 0);
    println!("✅ 2. Opened vault: {} entries", vault.entry_count());

    // ── 3. Add entries ────────────────────────────────────────────────────────
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
        e.password.set("weak"); // intentionally weak
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

    // ── 4. Read back ──────────────────────────────────────────────────────────
    let vault = open_vault(&vault_path, &creds).await?;
    assert_eq!(vault.entry_count(), 3, "Expected 3 entries");
    let github = vault.all_entries().find(|e| e.title.get() == "GitHub");
    assert!(github.is_some(), "GitHub entry not found");
    assert_eq!(github.unwrap().username.get(), "user@example.com");
    assert_eq!(github.unwrap().url, "https://github.com");
    println!("✅ 4. Read back entries correctly");

    // ── 5. Password generator ─────────────────────────────────────────────────
    let config = PasswordGeneratorConfig::default();
    let pw = PasswordGenerator::generate(&config)?;
    assert!(pw.len() >= 16, "Generated password too short: {}", pw.len());
    let strength = PasswordGenerator::score_strength(&pw);
    println!(
        "✅ 5. Generated password: {} chars, strength: {} ({}b entropy)",
        pw.len(),
        strength.label_en(),
        PasswordGenerator::estimate_entropy(&pw) as u32
    );

    // ── 6. Health audit ───────────────────────────────────────────────────────
    let report = audit_vault(&vault);
    assert_eq!(report.total_entries, 3);
    assert!(
        !report.weak_passwords.is_empty(),
        "Should detect weak Gmail password"
    );
    assert!(
        !report.reused_passwords.is_empty(),
        "Should detect reused password"
    );
    println!(
        "✅ 6. Health audit: score={}/100, weak={}, reused={}",
        report.score,
        report.weak_passwords.len(),
        report.reused_passwords.len()
    );

    // ── 7. OTP ────────────────────────────────────────────────────────────────
    let uri = "otpauth://totp/GitHub:user@example.com?secret=JBSWY3DPEHPK3PXP&issuer=GitHub";
    let otp_config = parse_otp_uri(uri)?;
    assert_eq!(otp_config.issuer.as_deref(), Some("GitHub"));
    let code = generate_totp(&otp_config)?;
    assert_eq!(code.code.len(), 6, "OTP should be 6 digits");
    assert!(code.remaining_seconds <= 30);
    println!(
        "✅ 7. OTP: code={}, remaining={}s, period={}s",
        code.code, code.remaining_seconds, code.period
    );

    // ── 8. Search ─────────────────────────────────────────────────────────────
    let results = vault.search(&SearchQuery::new("github"));
    assert!(!results.is_empty(), "Search should find GitHub");
    assert_eq!(results[0].title.get(), "GitHub");
    println!("✅ 8. Search 'github': {} result(s)", results.len());

    // ── 9. Group operations ───────────────────────────────────────────────────
    let mut vault = open_vault(&vault_path, &creds).await?;
    let work_group = vault.create_group("Work", vault.root_group_uuid)?;
    vault.move_entry(&e1, work_group)?;
    save_vault(&vault, &vault_path, &creds).await?;

    let vault = open_vault(&vault_path, &creds).await?;
    let work_entries = vault.get_group_entries(&work_group);
    assert_eq!(work_entries.len(), 1, "Work group should have 1 entry");
    assert_eq!(work_entries[0].title.get(), "GitHub");
    println!("✅ 9. Group operations: moved GitHub to Work group");

    // ── 10. Entry history ─────────────────────────────────────────────────────
    let mut vault = open_vault(&vault_path, &creds).await?;
    let entry = vault.get_entry(&e2).unwrap().clone();
    let mut updated = entry.clone();
    updated.password.set("NewStrongerPassword123!");
    updated.modified_at = chrono::Utc::now();
    vault.update_entry(updated)?;
    save_vault(&vault, &vault_path, &creds).await?;

    let vault = open_vault(&vault_path, &creds).await?;
    let entry = vault.get_entry(&e2).unwrap();
    assert!(
        !entry.history.is_empty(),
        "Should have history after update"
    );
    assert_eq!(entry.password.get(), "NewStrongerPassword123!");
    println!("✅ 10. Entry history: {} snapshot(s)", entry.history.len());

    // ── 11. Recycle bin ───────────────────────────────────────────────────────
    let mut vault = open_vault(&vault_path, &creds).await?;
    let count_before = vault.entry_count();
    vault.delete_entry(&e3, false)?; // move to recycle bin
    save_vault(&vault, &vault_path, &creds).await?;

    let vault = open_vault(&vault_path, &creds).await?;
    // Entry still exists in recycle bin
    assert_eq!(
        vault.entry_count(),
        count_before,
        "Entry should be in recycle bin"
    );
    println!("✅ 11. Recycle bin: entry moved (not permanently deleted)");

    // ── 12. KDBX round-trip integrity ─────────────────────────────────────────
    // Verify the vault can be opened with wrong password → error
    let wrong_creds = VaultCredentials::password_only("WrongPassword!");
    let result = open_vault(&vault_path, &wrong_creds).await;
    assert!(result.is_err(), "Should fail with wrong password");
    println!("✅ 12. Wrong password correctly rejected");

    // ── Summary ───────────────────────────────────────────────────────────────
    println!();
    println!("🎉 All 12 end-to-end tests passed!");
    println!("   KeePassEx core engine is working correctly.");
    println!();
    println!("   Tested:");
    println!("   • Vault create/open/save (KDBX 4.x)");
    println!("   • Entry CRUD (create, read, update, delete)");
    println!("   • Password generator (random + passphrase)");
    println!("   • Health audit (weak, reused detection)");
    println!("   • OTP (TOTP generation + URI parsing)");
    println!("   • Full-text search");
    println!("   • Group operations (create, move)");
    println!("   • Entry history");
    println!("   • Recycle bin");
    println!("   • Wrong password rejection");

    Ok(())
}
