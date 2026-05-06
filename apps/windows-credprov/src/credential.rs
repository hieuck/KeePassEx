//! Credential model — stores Windows credentials retrieved from the vault

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Windows credentials retrieved from the KeePassEx vault
#[derive(ZeroizeOnDrop)]
pub struct WindowsCredential {
    /// Windows username (domain\user or just user)
    pub username: String,
    /// Windows password
    pub password: String,
    /// Optional domain
    pub domain: Option<String>,
}

impl WindowsCredential {
    pub fn new(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            password: password.into(),
            domain: None,
        }
    }

    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Format as DOMAIN\username for Windows authentication
    pub fn qualified_username(&self) -> String {
        match &self.domain {
            Some(d) => format!("{}\\{}", d, self.username),
            None => self.username.clone(),
        }
    }
}

impl Drop for WindowsCredential {
    fn drop(&mut self) {
        self.username.zeroize();
        self.password.zeroize();
        if let Some(ref mut d) = self.domain {
            d.zeroize();
        }
    }
}
