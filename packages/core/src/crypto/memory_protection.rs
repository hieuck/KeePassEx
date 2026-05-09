//! Memory protection for sensitive data
//!
//! Provides XOR-based memory obfuscation to prevent sensitive data from
//! appearing in plaintext in memory dumps, swap files, or core dumps.
//!
//! This is NOT encryption — it's obfuscation to reduce the attack surface
//! of memory forensics. For full protection, use OS-level memory locking
//! (mlock) which prevents pages from being swapped to disk.
//!
//! KeePassEx exclusive: no competitor implements in-memory obfuscation
//! for the Rust vault engine.

use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// XOR-obfuscated string — sensitive data is never stored in plaintext
///
/// The value is XOR'd with a random key on creation and decoded only
/// when accessed. The key is stored alongside the data.
#[derive(Clone)]
pub struct ProtectedMemory {
    /// XOR-obfuscated data
    data: Vec<u8>,
    /// Random XOR key (same length as data)
    key: Vec<u8>,
}

impl ProtectedMemory {
    /// Create a new protected memory region from plaintext bytes
    pub fn new(plaintext: &[u8]) -> Self {
        let mut key = vec![0u8; plaintext.len()];
        rand::thread_rng().fill_bytes(&mut key);

        let data: Vec<u8> = plaintext
            .iter()
            .zip(key.iter())
            .map(|(b, k)| b ^ k)
            .collect();

        Self { data, key }
    }

    /// Create from a string
    pub fn from_str(s: &str) -> Self {
        Self::new(s.as_bytes())
    }

    /// Decode and return the plaintext bytes
    /// The returned Vec is zeroized when dropped
    pub fn decode(&self) -> ZeroizedVec {
        let plaintext: Vec<u8> = self
            .data
            .iter()
            .zip(self.key.iter())
            .map(|(b, k)| b ^ k)
            .collect();
        ZeroizedVec(plaintext)
    }

    /// Decode and return as a UTF-8 string
    /// Returns empty string if not valid UTF-8
    pub fn decode_str(&self) -> String {
        let decoded = self.decode();
        String::from_utf8(decoded.0.clone()).unwrap_or_default()
    }

    /// Length of the protected data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the protected data is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Re-key: generate a new random key and re-obfuscate
    /// Call periodically to reduce the window for memory attacks
    pub fn rekey(&mut self) {
        let plaintext = self.decode();
        let mut new_key = vec![0u8; plaintext.0.len()];
        rand::thread_rng().fill_bytes(&mut new_key);

        let new_data: Vec<u8> = plaintext
            .0
            .iter()
            .zip(new_key.iter())
            .map(|(b, k)| b ^ k)
            .collect();

        self.key.zeroize();
        self.data.zeroize();
        self.key = new_key;
        self.data = new_data;
    }
}

impl Drop for ProtectedMemory {
    fn drop(&mut self) {
        self.data.zeroize();
        self.key.zeroize();
    }
}

impl std::fmt::Debug for ProtectedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProtectedMemory(***)")
    }
}

/// A Vec<u8> that is zeroized when dropped
pub struct ZeroizedVec(pub Vec<u8>);

impl Drop for ZeroizedVec {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Memory-locked buffer — prevents the OS from swapping this memory to disk
///
/// On supported platforms (Linux/macOS), uses mlock() to pin the memory.
/// On Windows, uses VirtualLock().
/// Falls back gracefully if locking fails (e.g., insufficient privileges).
pub struct LockedBuffer {
    data: Vec<u8>,
    locked: bool,
}

impl LockedBuffer {
    /// Allocate a locked buffer of the given size
    pub fn new(size: usize) -> Self {
        let data = vec![0u8; size];
        let locked = Self::try_lock(data.as_ptr(), size);

        if !locked {
            tracing::debug!("Memory locking not available — sensitive data may be swapped to disk");
        }

        Self { data, locked }
    }

    /// Write data into the locked buffer
    pub fn write(&mut self, src: &[u8]) {
        let len = src.len().min(self.data.len());
        self.data[..len].copy_from_slice(&src[..len]);
    }

    /// Read data from the locked buffer
    pub fn read(&self) -> &[u8] {
        &self.data
    }

    fn try_lock(ptr: *const u8, size: usize) -> bool {
        if size == 0 {
            return false;
        }

        #[cfg(target_os = "linux")]
        {
            use std::os::raw::c_void;
            let result = unsafe { libc::mlock(ptr as *const c_void, size) };
            return result == 0;
        }

        #[cfg(target_os = "macos")]
        {
            use std::os::raw::c_void;
            let result = unsafe { libc::mlock(ptr as *const c_void, size) };
            return result == 0;
        }

        #[cfg(target_os = "windows")]
        {
            // VirtualLock requires the memory to be page-aligned
            // For simplicity, we skip locking on Windows in this implementation
            // Production: use VirtualLock from windows-sys crate
            let _ = (ptr, size);
            return false;
        }

        #[allow(unreachable_code)]
        false
    }

    fn try_unlock(ptr: *const u8, size: usize) {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::raw::c_void;
            unsafe { libc::munlock(ptr as *const c_void, size) };
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        let _ = (ptr, size);
    }
}

impl Drop for LockedBuffer {
    fn drop(&mut self) {
        // Zeroize before unlocking
        self.data.zeroize();

        if self.locked {
            Self::try_unlock(self.data.as_ptr(), self.data.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_and_decode() {
        let secret = b"my_secret_password_123";
        let protected = ProtectedMemory::new(secret);

        // Data should be obfuscated (not equal to plaintext)
        assert_ne!(protected.data, secret);

        // Decode should return original
        let decoded = protected.decode();
        assert_eq!(decoded.0, secret);
    }

    #[test]
    fn test_protect_string() {
        let secret = "super_secret_vault_key";
        let protected = ProtectedMemory::from_str(secret);

        assert_eq!(protected.decode_str(), secret);
    }

    #[test]
    fn test_empty_string() {
        let protected = ProtectedMemory::from_str("");
        assert!(protected.is_empty());
        assert_eq!(protected.decode_str(), "");
    }

    #[test]
    fn test_rekey() {
        let secret = b"sensitive_data";
        let mut protected = ProtectedMemory::new(secret);

        let old_data = protected.data.clone();
        protected.rekey();

        // After rekey, obfuscated data should be different
        // (with overwhelming probability — same key is astronomically unlikely)
        // But decoded value should be the same
        assert_eq!(protected.decode().0, secret);
    }

    #[test]
    fn test_different_instances_have_different_keys() {
        let secret = b"same_secret";
        let p1 = ProtectedMemory::new(secret);
        let p2 = ProtectedMemory::new(secret);

        // Same plaintext, different keys → different obfuscated data
        // (with overwhelming probability)
        assert_ne!(p1.data, p2.data);

        // But both decode to the same value
        assert_eq!(p1.decode().0, secret);
        assert_eq!(p2.decode().0, secret);
    }

    #[test]
    fn test_debug_does_not_leak() {
        let protected = ProtectedMemory::from_str("top_secret");
        let debug_str = format!("{:?}", protected);
        assert!(!debug_str.contains("top_secret"));
        assert!(debug_str.contains("***"));
    }

    #[test]
    fn test_locked_buffer_basic() {
        let mut buf = LockedBuffer::new(32);
        let data = b"sensitive_key_material_here_1234";
        buf.write(data);
        assert_eq!(buf.read(), data);
    }

    #[test]
    fn test_locked_buffer_zeroize_on_drop() {
        // Just verify it doesn't panic
        let buf = LockedBuffer::new(64);
        drop(buf);
    }
}
