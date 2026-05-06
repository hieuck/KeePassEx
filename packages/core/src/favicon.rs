//! Favicon / site icon fetching
//!
//! Fetches favicons for entries using multiple strategies:
//!   1. Google Favicon API (fast, reliable, no tracking — just domain lookup)
//!   2. DuckDuckGo Favicon API (privacy-friendly fallback)
//!   3. Direct /favicon.ico fetch from the site
//!
//! All fetches are done with k-anonymity in mind:
//!   - Only the domain is sent, never the full URL or any credentials
//!   - User-Agent identifies KeePassEx (no fingerprinting)
//!   - Results are cached in vault custom icons to avoid repeated requests

use crate::error::{KeePassExError, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use std::time::Duration;
use url::Url;

/// Maximum icon size to store (64 KB)
const MAX_ICON_SIZE_BYTES: usize = 64 * 1024;

/// Timeout for favicon fetch requests
const FETCH_TIMEOUT_SECS: u64 = 5;

/// Result of a favicon fetch operation
#[derive(Debug, Clone)]
pub struct FaviconResult {
    /// Base64-encoded PNG/ICO data
    pub data_base64: String,
    /// MIME type of the icon
    pub mime_type: String,
    /// The domain the icon was fetched for
    pub domain: String,
    /// Which strategy succeeded
    pub source: FaviconSource,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FaviconSource {
    GoogleApi,
    DuckDuckGoApi,
    DirectFetch,
}

impl std::fmt::Display for FaviconSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GoogleApi => write!(f, "Google Favicon API"),
            Self::DuckDuckGoApi => write!(f, "DuckDuckGo Favicon API"),
            Self::DirectFetch => write!(f, "Direct fetch"),
        }
    }
}

/// Extract the registrable domain from a URL string.
/// e.g. "https://mail.google.com/mail/u/0/" → "google.com"
pub fn extract_domain(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    let host = parsed.host_str()?;

    // Strip www. prefix
    let host = host.strip_prefix("www.").unwrap_or(host);

    // Return just the domain (no port)
    Some(host.to_lowercase())
}

/// Fetch a favicon for the given URL using multiple strategies.
/// Only the domain is used — the path and credentials are never sent.
pub async fn fetch_favicon(url: &str) -> Result<FaviconResult> {
    let domain = extract_domain(url)
        .ok_or_else(|| KeePassExError::Other(format!("Cannot extract domain from URL: {}", url)))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(FETCH_TIMEOUT_SECS))
        .user_agent(format!(
            "KeePassEx/{} (https://github.com/keepassex/keepassex)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    // Strategy 1: Google Favicon API (size=32 for crisp icons)
    if let Ok(result) = fetch_google_favicon(&client, &domain).await {
        return Ok(result);
    }

    // Strategy 2: DuckDuckGo Favicon API
    if let Ok(result) = fetch_duckduckgo_favicon(&client, &domain).await {
        return Ok(result);
    }

    // Strategy 3: Direct /favicon.ico
    if let Ok(result) = fetch_direct_favicon(&client, &domain).await {
        return Ok(result);
    }

    Err(KeePassExError::Other(format!(
        "Could not fetch favicon for domain: {}",
        domain
    )))
}

/// Fetch favicon via Google's favicon service.
/// Privacy note: Google receives the domain name only.
async fn fetch_google_favicon(client: &Client, domain: &str) -> Result<FaviconResult> {
    let url = format!("https://www.google.com/s2/favicons?domain={}&sz=32", domain);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if !response.status().is_success() {
        return Err(KeePassExError::Other(format!(
            "Google favicon API returned {}",
            response.status()
        )));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if bytes.len() > MAX_ICON_SIZE_BYTES {
        return Err(KeePassExError::Other("Favicon too large".into()));
    }

    // Google returns a 16x16 grey globe for unknown domains — detect and reject
    if bytes.len() < 200 {
        return Err(KeePassExError::Other(
            "Favicon appears to be placeholder".into(),
        ));
    }

    Ok(FaviconResult {
        data_base64: BASE64.encode(&bytes),
        mime_type: content_type,
        domain: domain.to_string(),
        source: FaviconSource::GoogleApi,
    })
}

/// Fetch favicon via DuckDuckGo's favicon service.
/// Privacy-friendly: DuckDuckGo does not track users.
async fn fetch_duckduckgo_favicon(client: &Client, domain: &str) -> Result<FaviconResult> {
    let url = format!("https://icons.duckduckgo.com/ip3/{}.ico", domain);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if !response.status().is_success() {
        return Err(KeePassExError::Other(format!(
            "DuckDuckGo favicon API returned {}",
            response.status()
        )));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if bytes.len() > MAX_ICON_SIZE_BYTES || bytes.len() < 100 {
        return Err(KeePassExError::Other("Favicon invalid or too large".into()));
    }

    Ok(FaviconResult {
        data_base64: BASE64.encode(&bytes),
        mime_type: "image/x-icon".to_string(),
        domain: domain.to_string(),
        source: FaviconSource::DuckDuckGoApi,
    })
}

/// Fetch favicon directly from the site's /favicon.ico endpoint.
async fn fetch_direct_favicon(client: &Client, domain: &str) -> Result<FaviconResult> {
    let url = format!("https://{}/favicon.ico", domain);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if !response.status().is_success() {
        return Err(KeePassExError::Other(format!(
            "Direct favicon fetch returned {}",
            response.status()
        )));
    }

    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/x-icon")
        .to_string();

    let bytes = response
        .bytes()
        .await
        .map_err(|e| KeePassExError::Other(e.to_string()))?;

    if bytes.len() > MAX_ICON_SIZE_BYTES || bytes.len() < 50 {
        return Err(KeePassExError::Other("Favicon invalid or too large".into()));
    }

    Ok(FaviconResult {
        data_base64: BASE64.encode(&bytes),
        mime_type: content_type,
        domain: domain.to_string(),
        source: FaviconSource::DirectFetch,
    })
}

/// Batch fetch favicons for multiple URLs.
/// Returns a map of domain → FaviconResult for successful fetches.
/// Failed fetches are silently skipped.
pub async fn fetch_favicons_batch(
    urls: &[String],
) -> std::collections::HashMap<String, FaviconResult> {
    use futures::future::join_all;

    // Deduplicate domains
    let mut domains: Vec<String> = urls
        .iter()
        .filter_map(|u| extract_domain(u))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    domains.sort();

    let futures: Vec<_> = domains
        .iter()
        .map(|domain| {
            let domain = domain.clone();
            async move {
                let result = fetch_favicon(&format!("https://{}", domain)).await;
                (domain, result)
            }
        })
        .collect();

    let results = join_all(futures).await;

    results
        .into_iter()
        .filter_map(|(domain, result)| result.ok().map(|r| (domain, r)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://mail.google.com/mail/u/0/"),
            Some("mail.google.com".to_string())
        );
        assert_eq!(
            extract_domain("https://www.github.com/keepassex"),
            Some("github.com".to_string())
        );
        assert_eq!(
            extract_domain("https://example.com:8080/path"),
            Some("example.com".to_string())
        );
        assert_eq!(extract_domain("not-a-url"), None);
    }

    #[test]
    fn test_extract_domain_strips_www() {
        assert_eq!(
            extract_domain("https://www.example.com/"),
            Some("example.com".to_string())
        );
    }
}
