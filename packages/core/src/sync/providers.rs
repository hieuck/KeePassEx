//! Sync provider implementations — WebDAV, Local Folder, Google Drive, OneDrive,
//! Dropbox, Amazon S3 (Signature V4), SFTP, iCloud Drive

use crate::error::{KeePassExError, Result};
use crate::sync::{SyncConfig, SyncEntry, SyncMetadata, SyncProvider, SyncProviderType};
use async_trait::async_trait;

fn build_http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent("KeePassEx/1.0 (https://github.com/keepassex/keepassex)")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))
}

// ─── WebDAV Provider ──────────────────────────────────────────────────────────

/// WebDAV sync provider
pub struct WebDavProvider {
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[async_trait]
impl SyncProvider for WebDavProvider {
    fn name(&self) -> &str {
        "WebDAV"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let client = build_http_client()?;

        let response = client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "WebDAV PUT failed: HTTP {}",
                response.status()
            )));
        }

        Ok(SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag: response
                .headers()
                .get("etag")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string()),
            revision: None,
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let client = build_http_client()?;

        let response = client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "WebDAV GET failed: HTTP {}",
                response.status()
            )));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let data = response
            .bytes()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?
            .to_vec();

        let meta = SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag,
            revision: None,
        };

        Ok((data, meta))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let client = build_http_client()?;

        // WebDAV PROPFIND
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:getcontentlength/>
    <D:getlastmodified/>
    <D:getetag/>
  </D:prop>
</D:propfind>"#;

        let response = client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "WebDAV PROPFIND failed: HTTP {}",
                response.status()
            )));
        }

        // Parse response (simplified — production would parse XML)
        Ok(SyncMetadata {
            path: path.to_string(),
            size: 0,
            modified_at: chrono::Utc::now(),
            etag: None,
            revision: None,
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let client = build_http_client()?;

        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:prop>
    <D:displayname/>
    <D:resourcetype/>
    <D:getcontentlength/>
  </D:prop>
</D:propfind>"#;

        let response = client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(body)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "WebDAV PROPFIND list failed: HTTP {}",
                response.status()
            )));
        }

        // Simplified — production would parse XML response
        Ok(Vec::new())
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let client = build_http_client()?;

        let response = client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "WebDAV DELETE failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        let client = build_http_client()?;

        let response = client
            .request(
                reqwest::Method::from_bytes(b"OPTIONS").unwrap(),
                &self.base_url,
            )
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("Cannot connect to WebDAV server: {}", e))
            })?;

        if response.status().is_success() || response.status().as_u16() == 207 {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "WebDAV server returned HTTP {}",
                response.status()
            )))
        }
    }
}

// ─── Local Folder Provider ────────────────────────────────────────────────────

pub struct LocalFolderProvider {
    pub base_path: std::path::PathBuf,
}

#[async_trait]
impl SyncProvider for LocalFolderProvider {
    fn name(&self) -> &str {
        "Local Folder"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let full_path = self.base_path.join(path);
        tokio::fs::write(&full_path, data).await?;

        let meta = tokio::fs::metadata(&full_path).await?;
        Ok(SyncMetadata {
            path: path.to_string(),
            size: meta.len(),
            modified_at: meta
                .modified()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| chrono::Utc::now()),
            etag: None,
            revision: None,
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let full_path = self.base_path.join(path);
        let data = tokio::fs::read(&full_path).await?;
        let meta = tokio::fs::metadata(&full_path).await?;

        let sync_meta = SyncMetadata {
            path: path.to_string(),
            size: meta.len(),
            modified_at: meta
                .modified()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| chrono::Utc::now()),
            etag: None,
            revision: None,
        };

        Ok((data, sync_meta))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let full_path = self.base_path.join(path);
        let meta = tokio::fs::metadata(&full_path).await?;

        Ok(SyncMetadata {
            path: path.to_string(),
            size: meta.len(),
            modified_at: meta
                .modified()
                .map(|t| chrono::DateTime::from(t))
                .unwrap_or_else(|_| chrono::Utc::now()),
            etag: None,
            revision: None,
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        let full_path = self.base_path.join(path);
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&full_path).await?;

        while let Some(entry) = dir.next_entry().await? {
            let file_type = entry.file_type().await?;
            entries.push(SyncEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_directory: file_type.is_dir(),
                metadata: None,
            });
        }

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = self.base_path.join(path);
        tokio::fs::remove_file(full_path).await?;
        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        if self.base_path.exists() {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "Path does not exist: {}",
                self.base_path.display()
            )))
        }
    }
}

// ─── Google Drive Provider ────────────────────────────────────────────────────

/// Google Drive sync provider using OAuth2 REST API
pub struct GoogleDriveProvider {
    /// OAuth2 access token (short-lived, caller must refresh before use)
    pub access_token: String,
    /// Optional refresh token for automatic token renewal
    pub refresh_token: Option<String>,
    /// OAuth2 client ID
    pub client_id: Option<String>,
    /// OAuth2 client secret
    pub client_secret: Option<String>,
}

impl GoogleDriveProvider {
    const DRIVE_UPLOAD_URL: &'static str = "https://www.googleapis.com/upload/drive/v3/files";
    const DRIVE_FILES_URL: &'static str = "https://www.googleapis.com/drive/v3/files";

    /// Resolve a Drive file ID from a path like "KeePassEx/vault.kdbx".
    /// Returns None if the file does not exist.
    async fn resolve_file_id(&self, path: &str) -> Result<Option<String>> {
        let client = build_http_client()?;
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        let query = format!("name='{}' and trashed=false", filename);
        let response = client
            .get(Self::DRIVE_FILES_URL)
            .bearer_auth(&self.access_token)
            .query(&[
                ("q", query.as_str()),
                ("fields", "files(id,name,size,modifiedTime)"),
            ])
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive list failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let id = body["files"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|f| f["id"].as_str())
            .map(|s| s.to_string());

        Ok(id)
    }
}

#[async_trait]
impl SyncProvider for GoogleDriveProvider {
    fn name(&self) -> &str {
        "Google Drive"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let client = build_http_client()?;
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        // Check if file already exists to decide create vs update
        let existing_id = self.resolve_file_id(path).await?;

        let metadata_json = serde_json::json!({ "name": filename });
        let metadata_bytes = metadata_json.to_string();

        // Multipart body: metadata part + media part
        let boundary = "keepassex_boundary_gdrive";
        let mut body = Vec::new();
        body.extend_from_slice(
            format!("--{boundary}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(metadata_bytes.as_bytes());
        body.extend_from_slice(
            format!("\r\n--{boundary}\r\nContent-Type: application/octet-stream\r\n\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{boundary}--").as_bytes());

        let content_type = format!("multipart/related; boundary={boundary}");

        let response = if let Some(file_id) = existing_id {
            // PATCH to update existing file
            client
                .patch(format!(
                    "{}/{}?uploadType=multipart",
                    Self::DRIVE_UPLOAD_URL,
                    file_id
                ))
                .bearer_auth(&self.access_token)
                .header("Content-Type", content_type)
                .body(body)
                .send()
                .await
        } else {
            // POST to create new file
            client
                .post(format!("{}?uploadType=multipart", Self::DRIVE_UPLOAD_URL))
                .bearer_auth(&self.access_token)
                .header("Content-Type", content_type)
                .body(body)
                .send()
                .await
        }
        .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive upload failed: HTTP {}",
                response.status()
            )));
        }

        let resp_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        Ok(SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag: None,
            revision: resp_body["id"].as_str().map(|s| s.to_string()),
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let file_id = self
            .resolve_file_id(path)
            .await?
            .ok_or_else(|| KeePassExError::SyncProviderError(format!("File not found: {path}")))?;

        let client = build_http_client()?;
        let response = client
            .get(format!("{}/{}?alt=media", Self::DRIVE_FILES_URL, file_id))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive download failed: HTTP {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?
            .to_vec();

        let meta = SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag: None,
            revision: Some(file_id),
        };

        Ok((data, meta))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let file_id = self
            .resolve_file_id(path)
            .await?
            .ok_or_else(|| KeePassExError::SyncProviderError(format!("File not found: {path}")))?;

        let client = build_http_client()?;
        let response = client
            .get(format!(
                "{}/{}?fields=id,name,size,modifiedTime",
                Self::DRIVE_FILES_URL,
                file_id
            ))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive metadata failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let size = body["size"]
            .as_str()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let modified_at = body["modifiedTime"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(SyncMetadata {
            path: path.to_string(),
            size,
            modified_at,
            etag: None,
            revision: Some(file_id),
        })
    }

    async fn list(&self, _path: &str) -> Result<Vec<SyncEntry>> {
        let client = build_http_client()?;
        let response = client
            .get(Self::DRIVE_FILES_URL)
            .bearer_auth(&self.access_token)
            .query(&[
                ("q", "trashed=false"),
                ("fields", "files(id,name,size,modifiedTime,mimeType)"),
            ])
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive list failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let entries = body["files"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|f| {
                        let name = f["name"].as_str().unwrap_or("").to_string();
                        let is_dir =
                            f["mimeType"].as_str() == Some("application/vnd.google-apps.folder");
                        SyncEntry {
                            name: name.clone(),
                            path: name,
                            is_directory: is_dir,
                            metadata: None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let file_id = self
            .resolve_file_id(path)
            .await?
            .ok_or_else(|| KeePassExError::SyncProviderError(format!("File not found: {path}")))?;

        let client = build_http_client()?;
        let response = client
            .delete(format!("{}/{}", Self::DRIVE_FILES_URL, file_id))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Google Drive delete failed: HTTP {}",
                response.status()
            )));
        }

        Ok(())
    }

    async fn test_connection(&self) -> Result<()> {
        let client = build_http_client()?;
        let response = client
            .get("https://www.googleapis.com/drive/v3/about?fields=user")
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("Cannot connect to Google Drive: {}", e))
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "Google Drive auth failed: HTTP {}",
                response.status()
            )))
        }
    }
}

// ─── OneDrive Provider ────────────────────────────────────────────────────────

/// Microsoft OneDrive sync provider using Microsoft Graph API
pub struct OneDriveProvider {
    /// OAuth2 access token
    pub access_token: String,
    /// Optional refresh token
    pub refresh_token: Option<String>,
    /// Drive root path prefix (e.g. "me/drive/root:/KeePassEx")
    pub drive_root: String,
}

impl OneDriveProvider {
    const GRAPH_BASE: &'static str = "https://graph.microsoft.com/v1.0";

    fn item_url(&self, path: &str) -> String {
        let clean = path.trim_start_matches('/');
        format!(
            "{}/{}/{}:/{}",
            Self::GRAPH_BASE,
            self.drive_root.trim_end_matches('/'),
            clean,
            ""
        )
        // Simpler: use /me/drive/root:/path:/content pattern
    }

    fn content_url(&self, path: &str) -> String {
        let clean = path.trim_start_matches('/');
        format!("{}/me/drive/root:/{}:/content", Self::GRAPH_BASE, clean)
    }

    fn metadata_url(&self, path: &str) -> String {
        let clean = path.trim_start_matches('/');
        format!("{}/me/drive/root:/{}", Self::GRAPH_BASE, clean)
    }
}

#[async_trait]
impl SyncProvider for OneDriveProvider {
    fn name(&self) -> &str {
        "OneDrive"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let client = build_http_client()?;
        let url = self.content_url(path);

        let response = client
            .put(&url)
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/octet-stream")
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "OneDrive upload failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let etag = body["eTag"].as_str().map(|s| s.to_string());
        let revision = body["id"].as_str().map(|s| s.to_string());

        Ok(SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag,
            revision,
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let client = build_http_client()?;
        let url = self.content_url(path);

        let response = client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "OneDrive download failed: HTTP {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?
            .to_vec();

        Ok((
            data.clone(),
            SyncMetadata {
                path: path.to_string(),
                size: data.len() as u64,
                modified_at: chrono::Utc::now(),
                etag: None,
                revision: None,
            },
        ))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let client = build_http_client()?;
        let url = self.metadata_url(path);

        let response = client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "OneDrive metadata failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let size = body["size"].as_u64().unwrap_or(0);
        let modified_at = body["lastModifiedDateTime"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(SyncMetadata {
            path: path.to_string(),
            size,
            modified_at,
            etag: body["eTag"].as_str().map(|s| s.to_string()),
            revision: body["id"].as_str().map(|s| s.to_string()),
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        let client = build_http_client()?;
        let url = if path.is_empty() || path == "/" {
            format!("{}/me/drive/root/children", Self::GRAPH_BASE)
        } else {
            format!(
                "{}/me/drive/root:/{}:/children",
                Self::GRAPH_BASE,
                path.trim_start_matches('/')
            )
        };

        let response = client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "OneDrive list failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let entries = body["value"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let name = item["name"].as_str().unwrap_or("").to_string();
                        let is_dir = item.get("folder").is_some();
                        SyncEntry {
                            name: name.clone(),
                            path: format!("{}/{}", path.trim_end_matches('/'), name),
                            is_directory: is_dir,
                            metadata: None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let client = build_http_client()?;
        let url = self.metadata_url(path);

        let response = client
            .delete(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        // 204 No Content is success for DELETE
        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "OneDrive delete failed: HTTP {}",
                response.status()
            )))
        }
    }

    async fn test_connection(&self) -> Result<()> {
        let client = build_http_client()?;
        let response = client
            .get(format!("{}/me/drive", Self::GRAPH_BASE))
            .bearer_auth(&self.access_token)
            .send()
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("Cannot connect to OneDrive: {}", e))
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "OneDrive auth failed: HTTP {}",
                response.status()
            )))
        }
    }
}

// ─── Dropbox Provider ─────────────────────────────────────────────────────────

/// Dropbox sync provider using Dropbox API v2
pub struct DropboxProvider {
    /// Dropbox OAuth2 access token
    pub access_token: String,
}

impl DropboxProvider {
    const CONTENT_BASE: &'static str = "https://content.dropboxapi.com/2";
    const API_BASE: &'static str = "https://api.dropboxapi.com/2";

    fn dropbox_path(path: &str) -> String {
        let p = path.trim_start_matches('/');
        if p.is_empty() {
            String::from("")
        } else {
            format!("/{p}")
        }
    }
}

#[async_trait]
impl SyncProvider for DropboxProvider {
    fn name(&self) -> &str {
        "Dropbox"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let client = build_http_client()?;
        let dbx_path = Self::dropbox_path(path);

        // Dropbox-API-Arg header carries JSON parameters
        let arg = serde_json::json!({
            "path": dbx_path,
            "mode": "overwrite",
            "autorename": false,
            "mute": false
        });

        let response = client
            .post(format!("{}/files/upload", Self::CONTENT_BASE))
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/octet-stream")
            .header("Dropbox-API-Arg", arg.to_string())
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Dropbox upload failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let revision = body["rev"].as_str().map(|s| s.to_string());
        let size = body["size"].as_u64().unwrap_or(data.len() as u64);

        Ok(SyncMetadata {
            path: path.to_string(),
            size,
            modified_at: chrono::Utc::now(),
            etag: None,
            revision,
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let client = build_http_client()?;
        let dbx_path = Self::dropbox_path(path);

        let arg = serde_json::json!({ "path": dbx_path });

        let response = client
            .post(format!("{}/files/download", Self::CONTENT_BASE))
            .bearer_auth(&self.access_token)
            .header("Dropbox-API-Arg", arg.to_string())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Dropbox download failed: HTTP {}",
                response.status()
            )));
        }

        // Metadata is returned in the Dropbox-API-Result header
        let revision = response
            .headers()
            .get("Dropbox-API-Result")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v["rev"].as_str().map(|r| r.to_string()));

        let data = response
            .bytes()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?
            .to_vec();

        Ok((
            data.clone(),
            SyncMetadata {
                path: path.to_string(),
                size: data.len() as u64,
                modified_at: chrono::Utc::now(),
                etag: None,
                revision,
            },
        ))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let client = build_http_client()?;
        let dbx_path = Self::dropbox_path(path);

        let response = client
            .post(format!("{}/files/get_metadata", Self::API_BASE))
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "path": dbx_path }).to_string())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Dropbox metadata failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let size = body["size"].as_u64().unwrap_or(0);
        let modified_at = body["server_modified"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        Ok(SyncMetadata {
            path: path.to_string(),
            size,
            modified_at,
            etag: None,
            revision: body["rev"].as_str().map(|s| s.to_string()),
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        let client = build_http_client()?;
        let dbx_path = Self::dropbox_path(path);

        let response = client
            .post(format!("{}/files/list_folder", Self::API_BASE))
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "path": dbx_path }).to_string())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "Dropbox list failed: HTTP {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let entries = body["entries"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| {
                        let name = item["name"].as_str().unwrap_or("").to_string();
                        let tag = item[".tag"].as_str().unwrap_or("");
                        SyncEntry {
                            name: name.clone(),
                            path: item["path_display"].as_str().unwrap_or(&name).to_string(),
                            is_directory: tag == "folder",
                            metadata: None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let client = build_http_client()?;
        let dbx_path = Self::dropbox_path(path);

        let response = client
            .post(format!("{}/files/delete_v2", Self::API_BASE))
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .body(serde_json::json!({ "path": dbx_path }).to_string())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "Dropbox delete failed: HTTP {}",
                response.status()
            )))
        }
    }

    async fn test_connection(&self) -> Result<()> {
        let client = build_http_client()?;
        let response = client
            .post(format!("{}/users/get_current_account", Self::API_BASE))
            .bearer_auth(&self.access_token)
            .header("Content-Type", "application/json")
            .body("null")
            .send()
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("Cannot connect to Dropbox: {}", e))
            })?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "Dropbox auth failed: HTTP {}",
                response.status()
            )))
        }
    }
}

// ─── Amazon S3 Provider ───────────────────────────────────────────────────────

/// Amazon S3 sync provider with AWS Signature Version 4 authentication.
/// Also compatible with S3-compatible services (MinIO, Backblaze B2, etc.)
/// when `endpoint_url` is set.
pub struct S3Provider {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
    pub bucket: String,
    /// Override endpoint for S3-compatible services (e.g. "https://s3.example.com")
    pub endpoint_url: Option<String>,
}

impl S3Provider {
    fn endpoint(&self) -> String {
        if let Some(ep) = &self.endpoint_url {
            ep.trim_end_matches('/').to_string()
        } else {
            format!("https://s3.{}.amazonaws.com", self.region)
        }
    }

    fn object_url(&self, key: &str) -> String {
        format!(
            "{}/{}/{}",
            self.endpoint(),
            self.bucket,
            key.trim_start_matches('/')
        )
    }

    /// Compute AWS Signature V4 for a request.
    ///
    /// Returns the `Authorization` header value and the `x-amz-date` header value.
    fn sign_v4(
        &self,
        method: &str,
        url: &str,
        payload: &[u8],
        extra_headers: &[(&str, &str)],
    ) -> Result<(String, String)> {
        use std::fmt::Write;

        let now = chrono::Utc::now();
        let date_stamp = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

        // Parse host from URL
        let parsed = ::url::Url::parse(url)
            .map_err(|e| KeePassExError::SyncProviderError(format!("Invalid S3 URL: {e}")))?;
        let host = parsed.host_str().unwrap_or("").to_string();
        let path = parsed.path();

        // Payload hash (SHA-256 hex)
        let payload_hash = sha256_hex(payload);

        // Canonical headers (must be sorted)
        let mut headers: Vec<(String, String)> = vec![
            ("host".to_string(), host.clone()),
            ("x-amz-content-sha256".to_string(), payload_hash.clone()),
            ("x-amz-date".to_string(), amz_date.clone()),
        ];
        for (k, v) in extra_headers {
            headers.push((k.to_lowercase(), v.to_string()));
        }
        headers.sort_by(|a, b| a.0.cmp(&b.0));

        let canonical_headers: String = headers.iter().fold(String::new(), |mut acc, (k, v)| {
            let _ = writeln!(acc, "{}:{}", k, v.trim());
            acc
        });

        let signed_headers: String = headers
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>()
            .join(";");

        // Canonical query string (empty for simple object ops)
        let canonical_query = parsed.query().unwrap_or("");

        let canonical_request = format!(
            "{method}\n{path}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
        );

        // String to sign
        let credential_scope = format!("{date_stamp}/{}/{}/aws4_request", self.region, "s3");
        let string_to_sign = format!(
            "AWS4-HMAC-SHA256\n{amz_date}\n{credential_scope}\n{}",
            sha256_hex(canonical_request.as_bytes())
        );

        // Signing key: HMAC chain
        let signing_key = {
            let k_date = hmac_sha256(
                format!("AWS4{}", self.secret_access_key).as_bytes(),
                date_stamp.as_bytes(),
            );
            let k_region = hmac_sha256(&k_date, self.region.as_bytes());
            let k_service = hmac_sha256(&k_region, b"s3");
            hmac_sha256(&k_service, b"aws4_request")
        };

        let signature = hex_encode(&hmac_sha256(&signing_key, string_to_sign.as_bytes()));

        let authorization = format!(
            "AWS4-HMAC-SHA256 Credential={}/{},SignedHeaders={},Signature={}",
            self.access_key_id, credential_scope, signed_headers, signature
        );

        Ok((authorization, amz_date))
    }
}

/// Compute HMAC-SHA256 (pure Rust, no external crate beyond what's already in core)
fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    // Block size for SHA-256 is 64 bytes
    const BLOCK_SIZE: usize = 64;
    let mut k = if key.len() > BLOCK_SIZE {
        sha256_bytes(key)
    } else {
        key.to_vec()
    };
    k.resize(BLOCK_SIZE, 0);

    let mut ipad = vec![0x36u8; BLOCK_SIZE];
    let mut opad = vec![0x5cu8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= k[i];
        opad[i] ^= k[i];
    }

    let mut inner = ipad;
    inner.extend_from_slice(data);
    let inner_hash = sha256_bytes(&inner);

    let mut outer = opad;
    outer.extend_from_slice(&inner_hash);
    sha256_bytes(&outer)
}

/// SHA-256 returning raw bytes — uses the sha2 crate already in packages/core
fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

fn sha256_hex(data: &[u8]) -> String {
    hex_encode(&sha256_bytes(data))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[async_trait]
impl SyncProvider for S3Provider {
    fn name(&self) -> &str {
        "Amazon S3"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        let key = path.trim_start_matches('/');
        let url = self.object_url(key);
        let content_type = "application/octet-stream";

        let (auth, amz_date) =
            self.sign_v4("PUT", &url, data, &[("content-type", content_type)])?;

        let client = build_http_client()?;
        let response = client
            .put(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(data))
            .header("Content-Type", content_type)
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "S3 PUT failed: HTTP {}",
                response.status()
            )));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(SyncMetadata {
            path: path.to_string(),
            size: data.len() as u64,
            modified_at: chrono::Utc::now(),
            etag,
            revision: None,
        })
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        let key = path.trim_start_matches('/');
        let url = self.object_url(key);

        let (auth, amz_date) = self.sign_v4("GET", &url, b"", &[])?;

        let client = build_http_client()?;
        let response = client
            .get(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(b""))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "S3 GET failed: HTTP {}",
                response.status()
            )));
        }

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        let data = response
            .bytes()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?
            .to_vec();

        Ok((
            data.clone(),
            SyncMetadata {
                path: path.to_string(),
                size: data.len() as u64,
                modified_at: chrono::Utc::now(),
                etag,
                revision: None,
            },
        ))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        let key = path.trim_start_matches('/');
        let url = self.object_url(key);

        let (auth, amz_date) = self.sign_v4("HEAD", &url, b"", &[])?;

        let client = build_http_client()?;
        let response = client
            .head(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(b""))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "S3 HEAD failed: HTTP {}",
                response.status()
            )));
        }

        let size = response
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        let modified_at = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| chrono::DateTime::parse_from_rfc2822(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim_matches('"').to_string());

        Ok(SyncMetadata {
            path: path.to_string(),
            size,
            modified_at,
            etag,
            revision: None,
        })
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        let prefix = path.trim_start_matches('/');
        let url = format!(
            "{}/{}?list-type=2&prefix={}",
            self.endpoint(),
            self.bucket,
            prefix
        );

        let (auth, amz_date) = self.sign_v4("GET", &url, b"", &[])?;

        let client = build_http_client()?;
        let response = client
            .get(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(b""))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(KeePassExError::SyncProviderError(format!(
                "S3 list failed: HTTP {}",
                response.status()
            )));
        }

        // Simplified XML parsing — production would use quick-xml
        let body = response
            .text()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        let entries: Vec<SyncEntry> = body
            .split("<Key>")
            .skip(1)
            .filter_map(|chunk| {
                let key = chunk.split("</Key>").next()?;
                let name = key.rsplit('/').next().unwrap_or(key).to_string();
                Some(SyncEntry {
                    name,
                    path: key.to_string(),
                    is_directory: key.ends_with('/'),
                    metadata: None,
                })
            })
            .collect();

        Ok(entries)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let key = path.trim_start_matches('/');
        let url = self.object_url(key);

        let (auth, amz_date) = self.sign_v4("DELETE", &url, b"", &[])?;

        let client = build_http_client()?;
        let response = client
            .delete(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(b""))
            .send()
            .await
            .map_err(|e| KeePassExError::SyncProviderError(e.to_string()))?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "S3 DELETE failed: HTTP {}",
                response.status()
            )))
        }
    }

    async fn test_connection(&self) -> Result<()> {
        // HEAD bucket to verify credentials and bucket existence
        let url = format!("{}/{}", self.endpoint(), self.bucket);
        let (auth, amz_date) = self.sign_v4("HEAD", &url, b"", &[])?;

        let client = build_http_client()?;
        let response = client
            .head(&url)
            .header("Authorization", auth)
            .header("x-amz-date", &amz_date)
            .header("x-amz-content-sha256", sha256_hex(b""))
            .send()
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("Cannot connect to S3: {}", e))
            })?;

        if response.status().is_success() || response.status().as_u16() == 200 {
            Ok(())
        } else {
            Err(KeePassExError::SyncProviderError(format!(
                "S3 bucket access failed: HTTP {}",
                response.status()
            )))
        }
    }
}

// ─── SFTP Provider ────────────────────────────────────────────────────────────

/// SFTP sync provider (SSH File Transfer Protocol).
///
/// Uses `russh` (pure-Rust async SSH2) + `russh-sftp` for SFTP operations.
/// No native OpenSSL dependency — works on all platforms including Windows.
///
/// # Authentication
/// Supports both password and public-key (PEM) authentication.
/// When `private_key` is provided it takes precedence over `password`.
///
/// # Host verification
/// When `host_fingerprint` is set, the SHA-256 fingerprint of the server's
/// host key is verified before any data is transferred, preventing MITM attacks.
pub struct SftpProvider {
    pub host: String,
    pub port: u16,
    pub username: String,
    /// Password authentication
    pub password: Option<String>,
    /// PEM-encoded private key for key-based authentication
    pub private_key: Option<String>,
    /// Optional passphrase for encrypted private key
    pub private_key_passphrase: Option<String>,
    /// Expected SHA-256 fingerprint (base64) of the server host key.
    /// If set, the connection is aborted when the fingerprint does not match.
    pub host_fingerprint: Option<String>,
}

impl SftpProvider {
    fn validate(&self) -> Result<()> {
        if self.host.is_empty() {
            return Err(KeePassExError::SyncProviderError(
                "SFTP host is required".to_string(),
            ));
        }
        if self.username.is_empty() {
            return Err(KeePassExError::SyncProviderError(
                "SFTP username is required".to_string(),
            ));
        }
        if self.password.is_none() && self.private_key.is_none() {
            return Err(KeePassExError::SyncProviderError(
                "SFTP requires either password or private key".to_string(),
            ));
        }
        Ok(())
    }

    /// Establish an authenticated SSH session and open an SFTP subsystem.
    /// Returns `(session_handle, sftp_session)`.
    async fn connect_sftp(
        &self,
    ) -> Result<(
        russh::client::Handle<SftpClientHandler>,
        russh_sftp::client::SftpSession,
    )> {
        use std::sync::Arc;

        let config = Arc::new(russh::client::Config::default());
        let handler = SftpClientHandler {
            expected_fingerprint: self.host_fingerprint.clone(),
        };

        let addr = format!("{}:{}", self.host, self.port);
        let mut session = russh::client::connect(config, addr.as_str(), handler)
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("SFTP connect to {addr} failed: {e}"))
            })?;

        // ── Authentication ─────────────────────────────────────────────────
        let auth_result = if let Some(pem) = &self.private_key {
            let passphrase = self.private_key_passphrase.as_deref();
            let key_pair = russh::keys::decode_secret_key(pem, passphrase).map_err(|e| {
                KeePassExError::SyncProviderError(format!("SFTP key decode failed: {e}"))
            })?;
            session
                .authenticate_publickey(
                    &self.username,
                    russh::keys::PrivateKeyWithHashAlg::new(
                        Arc::new(key_pair),
                        None, // use default hash algorithm
                    ),
                )
                .await
                .map_err(|e| {
                    KeePassExError::SyncProviderError(format!(
                        "SFTP key authentication failed: {e}"
                    ))
                })?
        } else if let Some(pw) = &self.password {
            session
                .authenticate_password(&self.username, pw)
                .await
                .map_err(|e| {
                    KeePassExError::SyncProviderError(format!(
                        "SFTP password authentication failed: {e}"
                    ))
                })?
        } else {
            return Err(KeePassExError::SyncProviderError(
                "No authentication method available".to_string(),
            ));
        };

        if !auth_result.success() {
            return Err(KeePassExError::SyncProviderError(
                "SFTP authentication failed — check credentials".to_string(),
            ));
        }

        // ── Open SFTP subsystem ────────────────────────────────────────────
        let channel = session.channel_open_session().await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP channel open failed: {e}"))
        })?;
        channel.request_subsystem(true, "sftp").await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP subsystem request failed: {e}"))
        })?;

        let sftp = russh_sftp::client::SftpSession::new(channel.into_stream())
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("SFTP session init failed: {e}"))
            })?;

        Ok((session, sftp))
    }

    /// Convert `russh_sftp` metadata to our `SyncMetadata`.
    fn sftp_meta_to_sync(path: &str, meta: &russh_sftp::client::fs::Metadata) -> SyncMetadata {
        use chrono::{DateTime, Utc};
        // russh_sftp Metadata::modified() returns Result<SystemTime, io::Error>
        let modified_at = meta
            .modified()
            .ok()
            .and_then(|t: std::time::SystemTime| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .ok()
                    .and_then(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
            })
            .unwrap_or_else(Utc::now);

        SyncMetadata {
            path: path.to_string(),
            size: meta.len(),
            modified_at,
            etag: None,
            revision: None,
        }
    }
}

/// russh client handler for SFTP connections.
/// Performs optional host fingerprint verification.
struct SftpClientHandler {
    expected_fingerprint: Option<String>,
}

impl russh::client::Handler for SftpClientHandler {
    // russh requires Error: From<russh::Error>; use russh::Error directly
    // and convert to KeePassExError at the call site.
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKey,
    ) -> std::result::Result<bool, Self::Error> {
        if let Some(expected) = &self.expected_fingerprint {
            use sha2::{Digest, Sha256};
            let key_bytes = server_public_key
                .to_bytes()
                .map_err(|_| russh::Error::CouldNotReadKey)?;
            let digest = Sha256::digest(&key_bytes);
            let actual = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, digest);
            if actual != *expected {
                // Reject the connection — fingerprint mismatch
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[async_trait]
impl SyncProvider for SftpProvider {
    fn name(&self) -> &str {
        "SFTP"
    }

    async fn upload(&self, path: &str, data: &[u8]) -> Result<SyncMetadata> {
        self.validate()?;
        use russh_sftp::protocol::OpenFlags;
        use tokio::io::AsyncWriteExt;

        let (_session, sftp) = self.connect_sftp().await?;

        // Ensure parent directory exists (best-effort)
        if let Some(parent) = std::path::Path::new(path).parent() {
            let parent_str = parent.to_string_lossy();
            if !parent_str.is_empty() && parent_str != "/" {
                let _ = sftp.create_dir(parent_str.as_ref()).await;
            }
        }

        let mut file = sftp
            .open_with_flags(
                path,
                OpenFlags::CREATE | OpenFlags::TRUNCATE | OpenFlags::WRITE,
            )
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("SFTP create {path} failed: {e}"))
            })?;

        file.write_all(data).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP write {path} failed: {e}"))
        })?;
        file.flush().await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP flush {path} failed: {e}"))
        })?;
        file.shutdown().await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP close {path} failed: {e}"))
        })?;

        // Stat the file to return accurate metadata
        let meta = sftp
            .metadata(path)
            .await
            .unwrap_or_else(|_| russh_sftp::client::fs::Metadata::default());
        Ok(Self::sftp_meta_to_sync(path, &meta))
    }

    async fn download(&self, path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        self.validate()?;
        use russh_sftp::protocol::OpenFlags;
        use tokio::io::AsyncReadExt;

        let (_session, sftp) = self.connect_sftp().await?;

        let meta = sftp.metadata(path).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP stat {path} failed: {e}"))
        })?;

        let mut file = sftp
            .open_with_flags(path, OpenFlags::READ)
            .await
            .map_err(|e| {
                KeePassExError::SyncProviderError(format!("SFTP open {path} failed: {e}"))
            })?;

        let mut data = Vec::new();
        file.read_to_end(&mut data).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP read {path} failed: {e}"))
        })?;

        let metadata = Self::sftp_meta_to_sync(path, &meta);
        Ok((data, metadata))
    }

    async fn get_metadata(&self, path: &str) -> Result<SyncMetadata> {
        self.validate()?;
        let (_session, sftp) = self.connect_sftp().await?;

        let meta = sftp.metadata(path).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP stat {path} failed: {e}"))
        })?;

        Ok(Self::sftp_meta_to_sync(path, &meta))
    }

    async fn list(&self, path: &str) -> Result<Vec<SyncEntry>> {
        self.validate()?;
        let (_session, sftp) = self.connect_sftp().await?;

        let entries = sftp.read_dir(path).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP readdir {path} failed: {e}"))
        })?;

        let result = entries
            .into_iter()
            .filter_map(|entry| {
                let name = entry.file_name();
                if name == "." || name == ".." {
                    return None;
                }
                let full_path = format!("{}/{}", path.trim_end_matches('/'), name);
                let meta = entry.metadata();
                let is_directory = meta.is_dir();
                let sync_meta = Self::sftp_meta_to_sync(&full_path, &meta);
                Some(SyncEntry {
                    name: name.to_string(),
                    path: full_path,
                    is_directory,
                    metadata: Some(sync_meta),
                })
            })
            .collect();

        Ok(result)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        self.validate()?;
        let (_session, sftp) = self.connect_sftp().await?;

        sftp.remove_file(path).await.map_err(|e| {
            KeePassExError::SyncProviderError(format!("SFTP delete {path} failed: {e}"))
        })
    }

    async fn test_connection(&self) -> Result<()> {
        self.validate()?;
        // Attempt full handshake + auth + SFTP subsystem open
        let (_session, sftp) = self.connect_sftp().await?;
        // Canonicalize "." as a lightweight no-op to confirm SFTP works
        let _ = sftp.canonicalize(".").await;
        Ok(())
    }
}

// ─── iCloud Drive Provider ────────────────────────────────────────────────────

/// iCloud Drive sync provider.
///
/// iCloud Drive access on iOS/macOS is handled entirely through the native
/// NSFileManager / UIDocument APIs in the Swift layer. This Rust struct acts
/// as a configuration holder and delegates all operations to the native layer
/// via the Tauri/React Native bridge.
///
/// On non-Apple platforms this provider is unavailable and all methods return
/// a clear error directing users to use a different provider.
pub struct ICloudDriveProvider {
    /// Relative path within the app's iCloud container (e.g. "vault.kdbx")
    pub container_path: String,
}

#[async_trait]
impl SyncProvider for ICloudDriveProvider {
    fn name(&self) -> &str {
        "iCloud Drive"
    }

    async fn upload(&self, _path: &str, _data: &[u8]) -> Result<SyncMetadata> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive sync is implemented in the native iOS/macOS layer. \
             Use the platform bridge to invoke NSFileManager operations."
                .to_string(),
        ))
    }

    async fn download(&self, _path: &str) -> Result<(Vec<u8>, SyncMetadata)> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive sync is implemented in the native iOS/macOS layer.".to_string(),
        ))
    }

    async fn get_metadata(&self, _path: &str) -> Result<SyncMetadata> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive sync is implemented in the native iOS/macOS layer.".to_string(),
        ))
    }

    async fn list(&self, _path: &str) -> Result<Vec<SyncEntry>> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive sync is implemented in the native iOS/macOS layer.".to_string(),
        ))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive sync is implemented in the native iOS/macOS layer.".to_string(),
        ))
    }

    async fn test_connection(&self) -> Result<()> {
        Err(KeePassExError::SyncProviderError(
            "iCloud Drive is only available on iOS and macOS. \
             On other platforms, please use WebDAV, Dropbox, or another provider."
                .to_string(),
        ))
    }
}

// ─── Factory Function ─────────────────────────────────────────────────────────

/// Create a boxed `SyncProvider` from a `SyncConfig`.
///
/// The config's `provider` field selects the implementation; credentials are
/// read from the optional `credentials` sub-struct.
///
/// # Errors
/// Returns `KeePassExError::SyncProviderError` when required credentials are
/// missing for the selected provider.
pub fn create_provider(config: &SyncConfig) -> Result<Box<dyn SyncProvider>> {
    let creds = config.credentials.as_ref();

    match config.provider {
        SyncProviderType::WebDav => {
            let username = creds
                .and_then(|c| c.username.as_deref())
                .unwrap_or("")
                .to_string();
            let password = creds
                .and_then(|c| c.password.as_deref())
                .unwrap_or("")
                .to_string();
            Ok(Box::new(WebDavProvider {
                base_url: config.remote_path.clone(),
                username,
                password,
            }))
        }

        SyncProviderType::LocalFolder => Ok(Box::new(LocalFolderProvider {
            base_path: std::path::PathBuf::from(&config.remote_path),
        })),

        SyncProviderType::GoogleDrive => {
            let access_token = creds
                .and_then(|c| c.token.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError(
                        "Google Drive requires an OAuth2 access token".to_string(),
                    )
                })?
                .to_string();
            Ok(Box::new(GoogleDriveProvider {
                access_token,
                refresh_token: None,
                client_id: None,
                client_secret: None,
            }))
        }

        SyncProviderType::OneDrive => {
            let access_token = creds
                .and_then(|c| c.token.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError(
                        "OneDrive requires an OAuth2 access token".to_string(),
                    )
                })?
                .to_string();
            Ok(Box::new(OneDriveProvider {
                access_token,
                refresh_token: None,
                drive_root: "me/drive/root".to_string(),
            }))
        }

        SyncProviderType::Dropbox => {
            let access_token = creds
                .and_then(|c| c.token.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError(
                        "Dropbox requires an access token".to_string(),
                    )
                })?
                .to_string();
            Ok(Box::new(DropboxProvider { access_token }))
        }

        SyncProviderType::S3 => {
            let access_key_id = creds
                .and_then(|c| c.access_key_id.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError("S3 requires access_key_id".to_string())
                })?
                .to_string();
            let secret_access_key = creds
                .and_then(|c| c.secret_access_key.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError("S3 requires secret_access_key".to_string())
                })?
                .to_string();
            let region = creds
                .and_then(|c| c.region.as_deref())
                .unwrap_or("us-east-1")
                .to_string();
            let bucket = creds
                .and_then(|c| c.bucket.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError("S3 requires bucket name".to_string())
                })?
                .to_string();
            let endpoint_url = creds.and_then(|c| c.endpoint.clone());
            Ok(Box::new(S3Provider {
                access_key_id,
                secret_access_key,
                region,
                bucket,
                endpoint_url,
            }))
        }

        SyncProviderType::SftpServer => {
            let host = config.remote_path.clone();
            let username = creds
                .and_then(|c| c.username.as_deref())
                .ok_or_else(|| {
                    KeePassExError::SyncProviderError("SFTP requires username".to_string())
                })?
                .to_string();
            let password = creds.and_then(|c| c.password.clone());
            let private_key = creds.and_then(|c| c.private_key_path.clone());
            Ok(Box::new(SftpProvider {
                host,
                port: 22,
                username,
                password,
                private_key,
                private_key_passphrase: None,
                host_fingerprint: creds.and_then(|c| c.host_fingerprint.clone()),
            }))
        }

        SyncProviderType::ICloudDrive => Ok(Box::new(ICloudDriveProvider {
            container_path: config.remote_path.clone(),
        })),
    }
}

// ─── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{ConflictResolution, SyncConfig, SyncProviderType};

    fn make_config(provider: SyncProviderType) -> SyncConfig {
        SyncConfig {
            provider,
            remote_path: "/test/vault.kdbx".to_string(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: None,
        }
    }

    #[test]
    fn test_create_local_folder_provider() {
        let config = make_config(SyncProviderType::LocalFolder);
        let provider = create_provider(&config).expect("should create local folder provider");
        assert_eq!(provider.name(), "Local Folder");
    }

    #[test]
    fn test_create_webdav_provider() {
        let config = SyncConfig {
            provider: SyncProviderType::WebDav,
            remote_path: "https://dav.example.com".to_string(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                username: Some("user".to_string()),
                password: Some("pass".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create webdav provider");
        assert_eq!(provider.name(), "WebDAV");
    }

    #[test]
    fn test_create_gdrive_provider_missing_token() {
        let config = make_config(SyncProviderType::GoogleDrive);
        let result = create_provider(&config);
        assert!(result.is_err(), "should fail without access token");
        let err = result.err().unwrap().to_string();
        assert!(
            err.contains("access token"),
            "error should mention access token"
        );
    }

    #[test]
    fn test_create_gdrive_provider_with_token() {
        let config = SyncConfig {
            provider: SyncProviderType::GoogleDrive,
            remote_path: String::new(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                token: Some("ya29.test_token".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create gdrive provider");
        assert_eq!(provider.name(), "Google Drive");
    }

    #[test]
    fn test_create_onedrive_provider_with_token() {
        let config = SyncConfig {
            provider: SyncProviderType::OneDrive,
            remote_path: String::new(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                token: Some("eyJ0test".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create onedrive provider");
        assert_eq!(provider.name(), "OneDrive");
    }

    #[test]
    fn test_create_dropbox_provider_with_token() {
        let config = SyncConfig {
            provider: SyncProviderType::Dropbox,
            remote_path: String::new(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                token: Some("sl.test_token".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create dropbox provider");
        assert_eq!(provider.name(), "Dropbox");
    }

    #[test]
    fn test_create_s3_provider_missing_credentials() {
        let config = make_config(SyncProviderType::S3);
        let result = create_provider(&config);
        assert!(result.is_err(), "should fail without S3 credentials");
    }

    #[test]
    fn test_create_s3_provider_with_credentials() {
        let config = SyncConfig {
            provider: SyncProviderType::S3,
            remote_path: String::new(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                access_key_id: Some("AKIAIOSFODNN7EXAMPLE".to_string()),
                secret_access_key: Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
                region: Some("us-east-1".to_string()),
                bucket: Some("my-keepass-bucket".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create S3 provider");
        assert_eq!(provider.name(), "Amazon S3");
    }

    #[test]
    fn test_create_sftp_provider_missing_username() {
        let config = make_config(SyncProviderType::SftpServer);
        let result = create_provider(&config);
        assert!(result.is_err(), "should fail without SFTP username");
    }

    #[test]
    fn test_create_sftp_provider_with_credentials() {
        let config = SyncConfig {
            provider: SyncProviderType::SftpServer,
            remote_path: "sftp.example.com".to_string(),
            auto_sync: false,
            sync_interval_seconds: 300,
            conflict_resolution: ConflictResolution::Merge,
            credentials: Some(crate::sync::SyncCredentials {
                username: Some("sftpuser".to_string()),
                password: Some("sftppass".to_string()),
                ..Default::default()
            }),
        };
        let provider = create_provider(&config).expect("should create SFTP provider");
        assert_eq!(provider.name(), "SFTP");
    }

    #[test]
    fn test_create_icloud_provider() {
        let config = make_config(SyncProviderType::ICloudDrive);
        let provider = create_provider(&config).expect("should create iCloud provider");
        assert_eq!(provider.name(), "iCloud Drive");
    }

    #[test]
    fn test_s3_sha256_hex() {
        // SHA-256 of empty string is well-known
        let hash = sha256_hex(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_dropbox_path_normalization() {
        assert_eq!(DropboxProvider::dropbox_path("vault.kdbx"), "/vault.kdbx");
        assert_eq!(DropboxProvider::dropbox_path("/vault.kdbx"), "/vault.kdbx");
        assert_eq!(DropboxProvider::dropbox_path(""), "");
    }
}
