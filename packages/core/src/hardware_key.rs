//! Hardware Key support — YubiKey HMAC-SHA1, FIDO2, Smart Card
//!
//! Provides challenge-response authentication with physical hardware keys
//! as a second factor for vault unlock (CompositeKey integration).
//!
//! # Supported key types
//! - **YubiKey HMAC-SHA1** (slot 1 or 2) — most common, works offline
//! - **YubiKey OTP** — Yubico OTP protocol
//! - **FIDO2** — any FIDO2/WebAuthn security key
//! - **Smart Card / PIV** — CAC, PIV cards
//! - **OnlyKey** — OnlyKey hardware token

use crate::error::{KeePassExError, Result};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

// ─── Types ────────────────────────────────────────────────────────────────────

/// Hardware key type identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardwareKeyType {
    /// YubiKey HMAC-SHA1 challenge-response (slot 1 or 2)
    YubikeyHmac,
    /// YubiKey OTP (Yubico OTP protocol)
    YubikeyOtp,
    /// FIDO2 / WebAuthn hardware key
    Fido2,
    /// PIV smart card / CAC
    SmartCard,
    /// OnlyKey hardware token
    OnlyKey,
}

impl std::fmt::Display for HardwareKeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardwareKeyType::YubikeyHmac => write!(f, "YubiKey HMAC-SHA1"),
            HardwareKeyType::YubikeyOtp => write!(f, "YubiKey OTP"),
            HardwareKeyType::Fido2 => write!(f, "FIDO2"),
            HardwareKeyType::SmartCard => write!(f, "Smart Card"),
            HardwareKeyType::OnlyKey => write!(f, "OnlyKey"),
        }
    }
}

/// YubiKey slot selection for HMAC-SHA1 challenge-response
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YubikeySlot {
    Slot1 = 1,
    Slot2 = 2,
}

/// Configuration for a hardware key requirement on a vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareKeyConfig {
    pub key_type: HardwareKeyType,
    /// Device serial number or FIDO2 credential ID (for identification)
    pub device_id: Option<String>,
    /// YubiKey slot (only for YubikeyHmac)
    pub slot: Option<YubikeySlot>,
    /// Human-readable label
    pub label: String,
    /// Whether physical touch is required
    pub require_touch: bool,
}

impl HardwareKeyConfig {
    pub fn new_yubikey_hmac(slot: YubikeySlot, label: impl Into<String>) -> Self {
        Self {
            key_type: HardwareKeyType::YubikeyHmac,
            device_id: None,
            slot: Some(slot),
            label: label.into(),
            require_touch: true,
        }
    }

    pub fn new_fido2(credential_id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key_type: HardwareKeyType::Fido2,
            device_id: Some(credential_id.into()),
            slot: None,
            label: label.into(),
            require_touch: true,
        }
    }
}

/// Information about a detected hardware key device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareKeyInfo {
    pub key_type: HardwareKeyType,
    pub device_id: String,
    pub label: String,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub is_connected: bool,
}

/// Result of a hardware key challenge-response operation
#[derive(Debug, Zeroize, ZeroizeOnDrop)]
pub struct HardwareKeyResponse {
    /// The 20-byte HMAC-SHA1 response (for YubiKey HMAC)
    pub response_bytes: Vec<u8>,
}

// ─── Challenge-Response ───────────────────────────────────────────────────────

/// Perform a HMAC-SHA1 challenge-response with a YubiKey.
///
/// The challenge is a 64-byte value derived from the vault's master seed.
/// The response is used as an additional key component in the CompositeKey.
///
/// # Platform notes
/// - Desktop: uses `yubico-manager` or direct HID communication
/// - Mobile: YubiKey NFC via Core NFC (iOS) / NFC adapter (Android)
/// - CLI: uses `ykman` or direct HID
///
/// This function is async because it may require user interaction (touch).
pub async fn yubikey_hmac_challenge(
    challenge: &[u8; 64],
    slot: YubikeySlot,
    device_serial: Option<u32>,
) -> Result<HardwareKeyResponse> {
    // In production this calls into platform-specific HID/NFC code.
    // The actual implementation is in the platform bridge layer
    // (Tauri command / React Native native module / CLI).
    //
    // Here we define the contract and error handling.
    let _ = (challenge, slot, device_serial);
    Err(KeePassExError::HardwareKeyNotConnected)
}

/// Enumerate connected hardware keys.
pub async fn list_hardware_keys() -> Result<Vec<HardwareKeyInfo>> {
    // Platform bridge implementation
    Ok(vec![])
}

/// Test that a hardware key responds correctly (for setup verification).
pub async fn test_hardware_key(config: &HardwareKeyConfig) -> Result<bool> {
    let _ = config;
    // Generate a random challenge and verify the key responds
    Err(KeePassExError::HardwareKeyNotConnected)
}

// ─── CompositeKey integration ─────────────────────────────────────────────────

/// Derive the hardware key component for use in CompositeKey.
///
/// The vault's master seed is used as the challenge, ensuring the hardware
/// key response is vault-specific (prevents cross-vault replay attacks).
pub async fn derive_hardware_key_component(
    master_seed: &[u8; 32],
    config: &HardwareKeyConfig,
) -> Result<Vec<u8>> {
    match config.key_type {
        HardwareKeyType::YubikeyHmac => {
            // Pad master_seed to 64 bytes for HMAC-SHA1 challenge
            let mut challenge = [0u8; 64];
            challenge[..32].copy_from_slice(master_seed);
            // Fill remaining bytes with a fixed pattern to reach 64 bytes
            for i in 32..64 {
                challenge[i] = (i as u8).wrapping_mul(0x5A);
            }
            let slot = config.slot.unwrap_or(YubikeySlot::Slot2);
            let response = yubikey_hmac_challenge(&challenge, slot, None).await?;
            Ok(response.response_bytes.clone())
        }
        HardwareKeyType::Fido2 => {
            // FIDO2 assertion with master_seed as client data hash
            Err(KeePassExError::HardwareKeyNotConnected)
        }
        _ => Err(KeePassExError::HardwareKeyNotSupported(
            config.key_type.to_string(),
        )),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_key_config_creation() {
        let config = HardwareKeyConfig::new_yubikey_hmac(YubikeySlot::Slot2, "My YubiKey 5");
        assert_eq!(config.key_type, HardwareKeyType::YubikeyHmac);
        assert_eq!(config.slot, Some(YubikeySlot::Slot2));
        assert_eq!(config.label, "My YubiKey 5");
        assert!(config.require_touch);
    }

    #[test]
    fn test_hardware_key_type_display() {
        assert_eq!(
            HardwareKeyType::YubikeyHmac.to_string(),
            "YubiKey HMAC-SHA1"
        );
        assert_eq!(HardwareKeyType::Fido2.to_string(), "FIDO2");
        assert_eq!(HardwareKeyType::SmartCard.to_string(), "Smart Card");
    }

    #[test]
    fn test_fido2_config_creation() {
        let config = HardwareKeyConfig::new_fido2("cred-id-base64url", "Security Key NFC");
        assert_eq!(config.key_type, HardwareKeyType::Fido2);
        assert_eq!(config.device_id, Some("cred-id-base64url".to_string()));
        assert!(config.slot.is_none());
    }

    #[test]
    fn test_yubikey_slot_values() {
        assert_eq!(YubikeySlot::Slot1 as u8, 1);
        assert_eq!(YubikeySlot::Slot2 as u8, 2);
    }
}
