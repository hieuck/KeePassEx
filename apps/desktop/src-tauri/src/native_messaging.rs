//! Native Messaging host — browser extension ↔ desktop communication
//!
//! Protocol: Chrome/Firefox native messaging
//! Each message is prefixed with a 4-byte little-endian length header.

use crate::state::AppState;
use keepassex_core::types::SearchQuery;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize)]
struct NativeRequest {
    id: String,
    action: String,
    payload: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct NativeResponse {
    id: String,
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

/// Start the native messaging host loop
/// Reads from stdin, writes to stdout (Chrome native messaging protocol)
pub async fn run_native_messaging_host(state: Arc<AppState>) {
    loop {
        // Read 4-byte length prefix
        let mut len_buf = [0u8; 4];
        if std::io::stdin().read_exact(&mut len_buf).is_err() {
            break; // EOF — browser closed
        }

        let msg_len = u32::from_le_bytes(len_buf) as usize;
        if msg_len == 0 || msg_len > 1024 * 1024 {
            break; // Invalid length
        }

        // Read message body
        let mut msg_buf = vec![0u8; msg_len];
        if std::io::stdin().read_exact(&mut msg_buf).is_err() {
            break;
        }

        // Parse request
        let request: NativeRequest = match serde_json::from_slice(&msg_buf) {
            Ok(r) => r,
            Err(e) => {
                send_error("unknown", &format!("Parse error: {}", e));
                continue;
            }
        };

        // Handle request
        let response = handle_request(&request, &state).await;
        send_response(&response);
    }
}

async fn handle_request(request: &NativeRequest, state: &AppState) -> NativeResponse {
    let id = request.id.clone();

    match request.action.as_str() {
        "ping" => NativeResponse {
            id,
            success: true,
            data: Some(serde_json::json!({ "version": "1.0.0" })),
            error: None,
        },

        "getVaultStatus" => {
            let vault = state.vault.read().unwrap();
            let (connected, locked, name) = match vault.as_ref() {
                Some(v) => (true, v.locked, Some(v.vault.meta.name.clone())),
                None => (false, true, None),
            };
            NativeResponse {
                id,
                success: true,
                data: Some(serde_json::json!({
                    "connected": connected,
                    "locked": locked,
                    "vaultName": name,
                })),
                error: None,
            }
        }

        "getCredentialsForUrl" => {
            let url = request.payload
                .as_ref()
                .and_then(|p| p.get("url"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let vault = state.vault.read().unwrap();
            let open_vault = match vault.as_ref() {
                Some(v) if !v.locked => v,
                _ => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Vault is locked or not open".into()),
                },
            };

            // Search by URL domain
            let domain = extract_domain(url);
            let mut query = SearchQuery::new(&domain);
            query.search_url = true;
            query.search_title = true;
            query.search_username = false;
            query.search_password = false;

            let entries: Vec<serde_json::Value> = open_vault.vault
                .search(&query)
                .iter()
                .map(|e| serde_json::json!({
                    "uuid": e.uuid.to_string(),
                    "title": e.title.get(),
                    "username": e.username.get(),
                    "url": e.url,
                    "hasOtp": e.otp.is_some(),
                }))
                .collect();

            NativeResponse {
                id,
                success: true,
                data: Some(serde_json::Value::Array(entries)),
                error: None,
            }
        }

        "getEntryForAutofill" => {
            let uuid_str = request.payload
                .as_ref()
                .and_then(|p| p.get("uuid"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let uuid = match uuid::Uuid::parse_str(uuid_str) {
                Ok(u) => u,
                Err(_) => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Invalid UUID".into()),
                },
            };

            let vault = state.vault.read().unwrap();
            let open_vault = match vault.as_ref() {
                Some(v) if !v.locked => v,
                _ => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Vault is locked".into()),
                },
            };

            match open_vault.vault.get_entry(&uuid) {
                Some(entry) => NativeResponse {
                    id,
                    success: true,
                    data: Some(serde_json::json!({
                        "uuid": entry.uuid.to_string(),
                        "title": entry.title.get(),
                        "username": entry.username.get(),
                        "password": entry.password.get(),
                        "url": entry.url,
                    })),
                    error: None,
                },
                None => NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Entry not found".into()),
                },
            }
        }

        "getEntryPassword" => {
            let uuid_str = request.payload
                .as_ref()
                .and_then(|p| p.get("uuid"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let uuid = match uuid::Uuid::parse_str(uuid_str) {
                Ok(u) => u,
                Err(_) => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Invalid UUID".into()),
                },
            };

            let vault = state.vault.read().unwrap();
            let open_vault = match vault.as_ref() {
                Some(v) if !v.locked => v,
                _ => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Vault is locked".into()),
                },
            };

            match open_vault.vault.get_entry(&uuid) {
                Some(entry) => NativeResponse {
                    id,
                    success: true,
                    data: Some(serde_json::Value::String(entry.password.get().to_string())),
                    error: None,
                },
                None => NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Entry not found".into()),
                },
            }
        }

        "generateTotp" => {
            let uuid_str = request.payload
                .as_ref()
                .and_then(|p| p.get("uuid"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let uuid = match uuid::Uuid::parse_str(uuid_str) {
                Ok(u) => u,
                Err(_) => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Invalid UUID".into()),
                },
            };

            let vault = state.vault.read().unwrap();
            let open_vault = match vault.as_ref() {
                Some(v) if !v.locked => v,
                _ => return NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Vault is locked".into()),
                },
            };

            match open_vault.vault.get_entry(&uuid) {
                Some(entry) => {
                    match entry.otp.as_ref() {
                        Some(otp_config) => {
                            match keepassex_core::otp::generate_totp(otp_config) {
                                Ok(code) => NativeResponse {
                                    id,
                                    success: true,
                                    data: Some(serde_json::json!({
                                        "code": code.code,
                                        "remainingSeconds": code.remaining_seconds,
                                        "period": code.period,
                                    })),
                                    error: None,
                                },
                                Err(e) => NativeResponse {
                                    id,
                                    success: false,
                                    data: None,
                                    error: Some(e.to_string()),
                                },
                            }
                        }
                        None => NativeResponse {
                            id,
                            success: false,
                            data: None,
                            error: Some("Entry has no OTP configured".into()),
                        },
                    }
                }
                None => NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some("Entry not found".into()),
                },
            }
        }

        "generatePassword" => {
            use keepassex_core::generator::PasswordGenerator;
            use keepassex_core::types::PasswordGeneratorConfig;

            let config = PasswordGeneratorConfig::default();
            match PasswordGenerator::generate(&config) {
                Ok(password) => {
                    let entropy = PasswordGenerator::estimate_entropy(&password);
                    NativeResponse {
                        id,
                        success: true,
                        data: Some(serde_json::json!({
                            "password": password,
                            "entropy": entropy,
                        })),
                        error: None,
                    }
                }
                Err(e) => NativeResponse {
                    id,
                    success: false,
                    data: None,
                    error: Some(e.to_string()),
                },
            }
        }

        unknown => NativeResponse {
            id,
            success: false,
            data: None,
            error: Some(format!("Unknown action: {}", unknown)),
        },
    }
}

fn send_response(response: &NativeResponse) {
    let json = serde_json::to_vec(response).unwrap_or_default();
    let len = json.len() as u32;
    let _ = std::io::stdout().write_all(&len.to_le_bytes());
    let _ = std::io::stdout().write_all(&json);
    let _ = std::io::stdout().flush();
}

fn send_error(id: &str, error: &str) {
    let response = NativeResponse {
        id: id.to_string(),
        success: false,
        data: None,
        error: Some(error.to_string()),
    };
    send_response(&response);
}

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or(url)
        .trim_start_matches("www.")
        .to_string()
}
