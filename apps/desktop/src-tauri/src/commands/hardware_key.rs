//! Hardware Key Tauri commands
//!
//! Exposes YubiKey / FIDO2 hardware key operations to the frontend.
//! The actual HID/NFC communication is handled by the core library's
//! platform bridge (called from here).

use keepassex_core::hardware_key::{
    HardwareKeyConfig, HardwareKeyInfo, HardwareKeyType, YubikeySlot,
    list_hardware_keys, test_hardware_key,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use crate::state::AppState;

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct HardwareKeyInfoDto {
    pub key_type: String,
    pub device_id: String,
    pub label: String,
    pub firmware_version: Option<String>,
    pub serial_number: Option<String>,
    pub is_connected: bool,
}

impl From<HardwareKeyInfo> for HardwareKeyInfoDto {
    fn from(info: HardwareKeyInfo) -> Self {
        Self {
            key_type: info.key_type.to_string(),
            device_id: info.device_id,
            label: info.label,
            firmware_version: info.firmware_version,
            serial_number: info.serial_number,
            is_connected: info.is_connected,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfigureHardwareKeyArgs {
    /// "yubikey_hmac" | "yubikey_otp" | "fido2" | "smart_card" | "onlykey"
    pub key_type: String,
    /// YubiKey slot (1 or 2)
    pub slot: Option<u8>,
    /// Human-readable label
    pub label: String,
    /// Whether to require touch confirmation
    pub require_touch: bool,
}

// ─── Commands ─────────────────────────────────────────────────────────────────

/// List all connected hardware keys
#[tauri::command]
pub async fn list_hardware_keys_cmd() -> Result<Vec<HardwareKeyInfoDto>, String> {
    let keys = list_hardware_keys()
        .await
        .map_err(|e| e.to_string())?;
    Ok(keys.into_iter().map(HardwareKeyInfoDto::from).collect())
}

/// Test a hardware key challenge-response
#[tauri::command]
pub async fn test_hardware_key_cmd(args: ConfigureHardwareKeyArgs) -> Result<bool, String> {
    let config = build_config(&args)?;
    test_hardware_key(&config).await.map_err(|e| e.to_string())
}

/// Configure hardware key requirement for the current vault
#[tauri::command]
pub async fn configure_hardware_key(
    args: ConfigureHardwareKeyArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let config = build_config(&args)?;

    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    // Store hardware key config in vault metadata
    // In production this serializes to the vault's custom data section
    open_vault.vault.meta.hardware_key_config = Some(serde_json::to_string(&config)
        .map_err(|e| e.to_string())?);
    open_vault.vault.dirty = true;

    Ok(())
}

/// Remove hardware key requirement from the current vault
#[tauri::command]
pub fn remove_hardware_key(state: State<'_, AppState>) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;
    open_vault.vault.meta.hardware_key_config = None;
    open_vault.vault.dirty = true;
    Ok(())
}

/// Get the current hardware key configuration for the open vault
#[tauri::command]
pub fn get_hardware_key_config(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;
    Ok(open_vault.vault.meta.hardware_key_config.clone())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn build_config(args: &ConfigureHardwareKeyArgs) -> Result<HardwareKeyConfig, String> {
    let key_type = match args.key_type.as_str() {
        "yubikey_hmac" => HardwareKeyType::YubikeyHmac,
        "yubikey_otp" => HardwareKeyType::YubikeyOtp,
        "fido2" => HardwareKeyType::Fido2,
        "smart_card" => HardwareKeyType::SmartCard,
        "onlykey" => HardwareKeyType::OnlyKey,
        other => return Err(format!("Unknown hardware key type: {}", other)),
    };

    let slot = match args.slot {
        Some(1) => Some(YubikeySlot::Slot1),
        Some(2) | None => Some(YubikeySlot::Slot2),
        Some(n) => return Err(format!("Invalid YubiKey slot: {}. Use 1 or 2.", n)),
    };

    Ok(HardwareKeyConfig {
        key_type,
        device_id: None,
        slot,
        label: args.label.clone(),
        require_touch: args.require_touch,
    })
}
