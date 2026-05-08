//! Check for updates — Tauri updater commands

use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Clone)]
pub struct UpdateInfo {
    /// Whether an update is available
    pub available: bool,
    /// Current version
    pub current_version: String,
    /// Latest version (if available)
    pub latest_version: Option<String>,
    /// Release notes (if available)
    pub release_notes: Option<String>,
    /// Download URL (if available)
    pub download_url: Option<String>,
    /// Release date (if available)
    pub release_date: Option<String>,
}

/// Check for available updates.
/// Returns update info — does NOT install automatically.
/// User must explicitly confirm before any download.
#[tauri::command]
pub async fn check_for_updates(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();

    // Only check if user has enabled update checks
    let check_enabled = state.settings.read().unwrap().check_for_updates;
    if !check_enabled {
        return Ok(UpdateInfo {
            available: false,
            current_version: current,
            latest_version: None,
            release_notes: None,
            download_url: None,
            release_date: None,
        });
    }

    // Query GitHub releases API for latest version
    let client = reqwest::Client::builder()
        .user_agent("KeePassEx/0.1.0 (https://github.com/keepassex/keepassex)")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get("https://api.github.com/repos/keepassex/keepassex/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Ok(UpdateInfo {
            available: false,
            current_version: current,
            latest_version: None,
            release_notes: None,
            download_url: None,
            release_date: None,
        });
    }

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let latest_tag = release["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v')
        .to_string();

    let release_notes = release["body"].as_str().map(|s| {
        // Truncate long release notes
        if s.len() > 500 {
            format!("{}...", &s[..500])
        } else {
            s.to_string()
        }
    });

    let release_date = release["published_at"]
        .as_str()
        .map(|s| s[..10].to_string()); // YYYY-MM-DD

    // Compare versions (simple string comparison works for semver x.y.z)
    let available =
        !latest_tag.is_empty() && latest_tag != current && is_newer(&latest_tag, &current);

    let download_url = if available {
        // Find the appropriate asset for current platform
        let platform_suffix = get_platform_suffix();
        release["assets"]
            .as_array()
            .and_then(|assets| {
                assets.iter().find(|a| {
                    a["name"]
                        .as_str()
                        .map(|n| n.contains(platform_suffix))
                        .unwrap_or(false)
                })
            })
            .and_then(|a| a["browser_download_url"].as_str())
            .map(|s| s.to_string())
    } else {
        None
    };

    Ok(UpdateInfo {
        available,
        current_version: current,
        latest_version: if latest_tag.is_empty() {
            None
        } else {
            Some(latest_tag)
        },
        release_notes,
        download_url,
        release_date,
    })
}

/// Get current app version
#[tauri::command]
pub fn get_app_version(app: tauri::AppHandle) -> String {
    app.package_info().version.to_string()
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/// Compare two semver strings — returns true if `a` is newer than `b`
fn is_newer(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(a) > parse(b)
}

fn get_platform_suffix() -> &'static str {
    #[cfg(target_os = "windows")]
    return "windows";
    #[cfg(target_os = "macos")]
    return "macos";
    #[cfg(target_os = "linux")]
    return "linux";
    #[allow(unreachable_code)]
    "unknown"
}
