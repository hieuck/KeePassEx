//! OTP Tauri commands

use keepassex_core::otp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OtpCodeDto {
    pub code: String,
    pub remaining_seconds: u64,
    pub period: u64,
    pub progress: f32,
    pub issuer: Option<String>,
    pub account: Option<String>,
}

/// Generate current TOTP for an entry
#[tauri::command]
pub fn generate_totp(
    entry_uuid: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<OtpCodeDto, String> {
    let vault_lock = state.vault.read().unwrap();
    let open_vault = vault_lock.as_ref().ok_or("No vault open")?;

    let uuid = uuid::Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault.vault.get_entry(&uuid).ok_or("Entry not found")?;

    let otp_config = entry.otp.as_ref().ok_or("Entry has no OTP configured")?;

    let code = otp::generate_totp(otp_config).map_err(|e| e.to_string())?;
    let progress = code.progress();

    Ok(OtpCodeDto {
        code: code.code,
        remaining_seconds: code.remaining_seconds,
        period: code.period,
        progress,
        issuer: code.issuer,
        account: code.account,
    })
}

/// Save OTP configuration to an entry from a URI (otpauth://...)
#[tauri::command]
pub fn set_entry_otp(
    entry_uuid: String,
    uri: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<(), String> {
    let config = otp::parse_otp_uri(&uri).map_err(|e| e.to_string())?;

    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = uuid::Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    entry.otp = Some(config);
    open_vault.vault.dirty = true;
    Ok(())
}

/// Remove OTP from an entry
#[tauri::command]
pub fn remove_entry_otp(
    entry_uuid: String,
    state: tauri::State<'_, crate::state::AppState>,
) -> Result<(), String> {
    let mut vault_lock = state.vault.write().unwrap();
    let open_vault = vault_lock.as_mut().ok_or("No vault open")?;

    if open_vault.locked {
        return Err("Vault is locked".into());
    }

    let uuid = uuid::Uuid::parse_str(&entry_uuid).map_err(|e| e.to_string())?;
    let entry = open_vault
        .vault
        .get_entry_mut(&uuid)
        .ok_or("Entry not found")?;

    entry.otp = None;
    open_vault.vault.dirty = true;
    Ok(())
}

/// Parse an OTP URI (otpauth://...)
#[tauri::command]
pub fn parse_otp_uri(uri: String) -> Result<OtpConfigDto, String> {
    let config = otp::parse_otp_uri(&uri).map_err(|e| e.to_string())?;

    Ok(OtpConfigDto {
        secret: config.secret.get().to_string(),
        algorithm: format!("{:?}", config.algorithm),
        digits: config.digits,
        period: config.period,
        otp_type: format!("{:?}", config.otp_type),
        issuer: config.issuer,
        account: config.account,
    })
}

#[derive(Debug, Serialize)]
pub struct OtpConfigDto {
    pub secret: String,
    pub algorithm: String,
    pub digits: u8,
    pub period: u64,
    pub otp_type: String,
    pub issuer: Option<String>,
    pub account: Option<String>,
}
