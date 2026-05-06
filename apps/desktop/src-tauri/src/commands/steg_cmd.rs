//! Tauri commands — Steganography
use serde::{Deserialize, Serialize};
use tauri::command;

#[derive(Debug, Serialize)]
pub struct StegCarrierInfo {
    pub format: String,
    pub has_vault: bool,
    pub capacity_bytes: Option<usize>,
}

/// Detect carrier format and check for embedded vault
#[command]
pub async fn detect_steg_carrier(path: String) -> Result<StegCarrierInfo, String> {
    let data = std::fs::read(&path).map_err(|e| e.to_string())?;

    let format = match keepassex_core::steg::StegFormat::detect(&data) {
        Some(keepassex_core::steg::StegFormat::Png) => "png",
        Some(keepassex_core::steg::StegFormat::Jpeg) => "jpeg",
        Some(keepassex_core::steg::StegFormat::Mp4) => "mp4",
        Some(keepassex_core::steg::StegFormat::Avi) => "avi",
        None => return Err("Unsupported file format".into()),
    };

    let has_vault = keepassex_core::steg::has_embedded_vault(&data);
    let capacity_bytes = keepassex_core::steg::max_capacity(&data);

    Ok(StegCarrierInfo {
        format: format.to_string(),
        has_vault,
        capacity_bytes,
    })
}

/// Embed vault into carrier file
#[command]
pub async fn steg_embed_vault(
    carrier_path: String,
    vault_path: String,
    output_path: String,
    steg_password: String,
) -> Result<(), String> {
    let carrier = std::fs::read(&carrier_path).map_err(|e| e.to_string())?;
    let vault_data = std::fs::read(&vault_path).map_err(|e| e.to_string())?;

    let modified = keepassex_core::steg::embed(&carrier, &vault_data, &steg_password)
        .map_err(|e| e.to_string())?;

    std::fs::write(&output_path, modified).map_err(|e| e.to_string())?;
    Ok(())
}

/// Extract vault from carrier file
#[command]
pub async fn steg_extract_vault(
    carrier_path: String,
    output_path: String,
    steg_password: String,
) -> Result<(), String> {
    let carrier = std::fs::read(&carrier_path).map_err(|e| e.to_string())?;

    let vault_data =
        keepassex_core::steg::extract(&carrier, &steg_password).map_err(|e| e.to_string())?;

    std::fs::write(&output_path, vault_data).map_err(|e| e.to_string())?;
    Ok(())
}
