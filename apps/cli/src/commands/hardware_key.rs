//! `kpx hardware-key` — manage hardware key configuration
//!
//! Usage:
//!   kpx hardware-key list          # list connected hardware keys
//!   kpx hardware-key test           # test the configured hardware key
//!   kpx hardware-key setup          # interactive setup wizard

use keepassex_core::hardware_key::{
    HardwareKeyConfig, HardwareKeyType, YubikeySlot, list_hardware_keys, test_hardware_key,
};
use colored::Colorize;

pub async fn run_list() -> anyhow::Result<()> {
    println!("{}", "Scanning for hardware keys...".dimmed());

    let keys = list_hardware_keys().await.unwrap_or_default();

    if keys.is_empty() {
        println!("{}", "No hardware keys detected.".yellow());
        println!();
        println!("Supported key types:");
        println!("  {} — YubiKey HMAC-SHA1 challenge-response (slot 1 or 2)", "YubiKey 5".bold());
        println!("  {} — YubiKey OTP", "YubiKey OTP".bold());
        println!("  {} — Any FIDO2/WebAuthn security key", "FIDO2".bold());
        println!("  {} — PIV smart card / CAC", "Smart Card".bold());
        println!("  {} — OnlyKey hardware token", "OnlyKey".bold());
        println!();
        println!("Connect a hardware key and run {} again.", "kpx hardware-key list".cyan());
    } else {
        println!("{}", format!("{} hardware key(s) detected:", keys.len()).bold());
        println!("{}", "─".repeat(50).dimmed());
        for key in &keys {
            let status = if key.is_connected {
                "● connected".green()
            } else {
                "○ disconnected".red()
            };
            println!("  {} {}", key.label.bold(), status);
            println!("    Type: {}", key.key_type.to_string().cyan());
            println!("    ID:   {}", key.device_id.dimmed());
            if let Some(ref fw) = key.firmware_version {
                println!("    FW:   {}", fw.dimmed());
            }
            if let Some(ref sn) = key.serial_number {
                println!("    S/N:  {}", sn.dimmed());
            }
        }
    }

    Ok(())
}

pub async fn run_test(slot: u8) -> anyhow::Result<()> {
    let yubikey_slot = match slot {
        1 => YubikeySlot::Slot1,
        2 => YubikeySlot::Slot2,
        _ => {
            anyhow::bail!("Invalid slot: {}. Use 1 or 2.", slot);
        }
    };

    let config = HardwareKeyConfig::new_yubikey_hmac(yubikey_slot, "YubiKey");

    println!("{}", format!("Testing YubiKey HMAC-SHA1 (slot {})...", slot).dimmed());
    println!("{}", "Touch your YubiKey when it flashes.".yellow());

    match test_hardware_key(&config).await {
        Ok(true) => {
            println!("{}", "✓ Hardware key test successful!".green().bold());
            println!("  The key responded correctly to the challenge.");
        }
        Ok(false) => {
            println!("{}", "✗ Hardware key test failed.".red());
            println!("  The key did not respond as expected.");
        }
        Err(e) => {
            println!("{}", format!("✗ Error: {}", e).red());
            println!();
            println!("Troubleshooting:");
            println!("  1. Make sure the YubiKey is connected");
            println!("  2. Ensure slot {} is configured for HMAC-SHA1", slot);
            println!("  3. Use YubiKey Manager to configure the slot");
            println!("     Download: https://www.yubico.com/support/download/yubikey-manager/");
        }
    }

    Ok(())
}

pub fn run_setup() -> anyhow::Result<()> {
    println!("{}", "Hardware Key Setup Wizard".bold());
    println!("{}", "─".repeat(40).dimmed());
    println!();
    println!("This wizard helps you configure a hardware key as a second factor");
    println!("for your KeePassEx vault.");
    println!();
    println!("{}", "Supported key types:".underline());
    println!("  [1] YubiKey HMAC-SHA1 (recommended)");
    println!("      Works offline, most compatible");
    println!("  [2] FIDO2 / WebAuthn security key");
    println!("      Works with any FIDO2 key");
    println!("  [3] Smart Card / PIV");
    println!("      For enterprise/government use");
    println!();
    println!("{}", "To configure:".dimmed());
    println!("  1. Open your vault in the desktop app");
    println!("  2. Go to Settings → Security → Hardware Key");
    println!("  3. Follow the on-screen instructions");
    println!();
    println!("Or use the CLI after setup:");
    println!("  {}", "kpx hardware-key test --slot 2".cyan());

    Ok(())
}
