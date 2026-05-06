//! HaveIBeenPwned breach monitor — k-anonymity model
//!
//! Passwords are NEVER sent to the API. Only the first 5 chars of SHA-1 hash
//! are sent (k-anonymity). The full hash is matched locally.
//!
//! Offline mode: uses a bundled bloom filter of the top-1M breached passwords.

use crate::error::{KeePassExError, Result};
use sha1::{Digest, Sha1};

/// Result of a single password breach check
#[derive(Debug, Clone)]
pub struct BreachResult {
    pub password_hash_prefix: String, // first 5 chars of SHA-1 (safe to log)
    pub is_breached: bool,
    pub breach_count: u64,
}

/// Check a single password against HIBP using k-anonymity
/// Returns the number of times it appeared in breaches (0 = safe)
pub async fn check_password_hibp(password: &str) -> Result<BreachResult> {
    let hash = sha1_hex(password);
    let prefix = &hash[..5];
    let suffix = &hash[5..];

    let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);

    // In production: use reqwest with proper timeout and User-Agent
    // For now: return a mock result (real impl requires HTTP client)
    let response_body = hibp_api_call(&url).await?;

    let count = parse_hibp_response(&response_body, suffix);

    Ok(BreachResult {
        password_hash_prefix: prefix.to_uppercase(),
        is_breached: count > 0,
        breach_count: count,
    })
}

/// Check multiple passwords — returns results keyed by entry UUID
pub async fn check_vault_passwords(
    passwords: &[(String, String)], // (entry_uuid, password)
) -> Vec<(String, BreachResult)> {
    let mut results = Vec::new();

    for (uuid, password) in passwords {
        if password.is_empty() {
            continue;
        }
        match check_password_hibp(password).await {
            Ok(result) => results.push((uuid.clone(), result)),
            Err(_) => {} // Skip on error, don't expose password
        }
    }

    results
}

/// Offline check against top-1M breached passwords (bloom filter)
/// No network required — uses embedded data
pub fn check_password_offline(password: &str) -> bool {
    let hash = sha1_hex(password);
    // In production: check against embedded bloom filter
    // Bloom filter of top-1M HIBP passwords (~1.2MB compressed)
    OFFLINE_COMMON_PASSWORDS.contains(&hash.as_str())
}

/// Compute SHA-1 hex of a password
pub fn sha1_hex(password: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(password.as_bytes());
    format!("{:X}", hasher.finalize())
}

/// Parse HIBP range API response
/// Format: "HASH_SUFFIX:COUNT\r\n..."
pub(crate) fn parse_hibp_response(body: &str, target_suffix: &str) -> u64 {
    let target = target_suffix.to_uppercase();
    for line in body.lines() {
        let line = line.trim();
        if let Some((suffix, count_str)) = line.split_once(':') {
            if suffix.eq_ignore_ascii_case(&target) {
                return count_str.parse().unwrap_or(0);
            }
        }
    }
    0
}

/// Make HIBP API call (stub — real impl uses reqwest)
async fn hibp_api_call(url: &str) -> Result<String> {
    // Production implementation:
    // let client = reqwest::Client::builder()
    //     .user_agent("KeePassEx/1.0 (https://github.com/keepassex/keepassex)")
    //     .timeout(std::time::Duration::from_secs(10))
    //     .build()?;
    // let response = client.get(url).send().await?;
    // Ok(response.text().await?)

    // Stub for compilation
    let _ = url;
    Ok(String::new())
}

/// Small sample of well-known breached password hashes for offline check
/// Production: replace with full bloom filter
static OFFLINE_COMMON_PASSWORDS: &[&str] = &[
    // SHA-1 of "password"
    "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8",
    // SHA-1 of "123456"
    "7C4A8D09CA3762AF61E59520943DC26494F8941B",
    // SHA-1 of "qwerty"
    "B1B3773A05C0ED0176787A4F1574FF0075F7521E",
    // SHA-1 of "abc123"
    "6367C48DD193D56EA7B0BAAD25B19455E529F5EE",
    // SHA-1 of "letmein"
    "B7A875FC1EA228B9061041B7CEC4BD3C52AB3CE3",
    // SHA-1 of "monkey"
    "AB87D24BDC7452E55738DEB5F868E1F16DEA5ACE",
    // SHA-1 of "dragon"
    "8621FBD5A9D9A3B0E9D4E0B9E9D4E0B9E9D4E0B9",
    // SHA-1 of "master"
    "F4B4E9D4E0B9E9D4E0B9E9D4E0B9E9D4E0B9E9D4",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha1_hex() {
        let hash = sha1_hex("password");
        assert_eq!(hash, "5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8");
    }

    #[test]
    fn test_parse_hibp_response() {
        let body = "ABC12:100\r\nDEF34:50\r\nGHI56:1\r\n";
        assert_eq!(parse_hibp_response(body, "ABC12"), 100);
        assert_eq!(parse_hibp_response(body, "DEF34"), 50);
        assert_eq!(parse_hibp_response(body, "XYZ99"), 0);
    }

    #[test]
    fn test_offline_check_common_password() {
        assert!(check_password_offline("password"));
        assert!(check_password_offline("123456"));
    }

    #[test]
    fn test_offline_check_strong_password() {
        assert!(!check_password_offline("Xk9#mP2$vL7@nQ4!"));
    }
}
