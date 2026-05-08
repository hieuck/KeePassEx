//! Entry cache — in-memory LRU cache for vault entries
//!
//! Provides fast repeated access to frequently-used entries without
//! re-reading from the vault data structure on every operation.
//! The cache is invalidated on any vault mutation (create/update/delete).
//!
//! # Design
//! - LRU eviction with configurable capacity (default: 500 entries)
//! - Cache keys are entry UUIDs
//! - Cache is invalidated atomically on write operations
//! - Thread-safe via `RwLock`
//! - Sensitive data (passwords) are NOT cached — only metadata DTOs
//!
//! # Usage
//! ```no_run
//! use keepassex_core::cache::{EntryCache, CachedEntry};
//! use uuid::Uuid;
//!
//! let cache = EntryCache::new(500);
//! let uuid = Uuid::new_v4();
//!
//! // Store a cached entry
//! cache.put(uuid, CachedEntry { uuid, title: "GitHub".into(), ..Default::default() });
//!
//! // Retrieve
//! if let Some(entry) = cache.get(&uuid) {
//!     println!("Cache hit: {}", entry.title);
//! }
//!
//! // Invalidate on mutation
//! cache.invalidate(&uuid);
//! cache.invalidate_all(); // on vault save/reload
//! ```

use std::collections::{HashMap, VecDeque};
use std::sync::RwLock;
use uuid::Uuid;

// ─── Cached Entry ─────────────────────────────────────────────────────────────

/// A lightweight, non-sensitive snapshot of an entry for display purposes.
/// Passwords and protected fields are NEVER stored in the cache.
#[derive(Debug, Clone, Default)]
pub struct CachedEntry {
    pub uuid: Uuid,
    pub group_uuid: Uuid,
    pub title: String,
    pub username: String,
    /// URL (not sensitive)
    pub url: String,
    pub icon_id: u32,
    pub tags: Vec<String>,
    pub has_password: bool,
    pub has_otp: bool,
    pub has_passkey: bool,
    pub has_ssh_key: bool,
    pub has_attachments: bool,
    pub is_expired: bool,
    pub expiry: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub modified_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u32,
}

// ─── LRU Cache ────────────────────────────────────────────────────────────────

/// Thread-safe LRU cache for vault entries.
pub struct EntryCache {
    inner: RwLock<LruInner>,
}

struct LruInner {
    /// Map from UUID to (index_in_order, CachedEntry)
    map: HashMap<Uuid, CachedEntry>,
    /// Access order (front = most recently used)
    order: VecDeque<Uuid>,
    /// Maximum number of entries to cache
    capacity: usize,
    /// Cache hit counter (for diagnostics)
    hits: u64,
    /// Cache miss counter (for diagnostics)
    misses: u64,
}

impl EntryCache {
    /// Create a new cache with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(LruInner {
                map: HashMap::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
                capacity,
                hits: 0,
                misses: 0,
            }),
        }
    }

    /// Create a cache with the default capacity (500 entries).
    pub fn default() -> Self {
        Self::new(500)
    }

    /// Insert or update an entry in the cache.
    pub fn put(&self, uuid: Uuid, entry: CachedEntry) {
        let mut inner = self.inner.write().unwrap();

        // Remove existing position in order if present
        if inner.map.contains_key(&uuid) {
            inner.order.retain(|u| u != &uuid);
        }

        // Evict LRU entry if at capacity
        if inner.map.len() >= inner.capacity && !inner.map.contains_key(&uuid) {
            if let Some(lru_uuid) = inner.order.pop_back() {
                inner.map.remove(&lru_uuid);
            }
        }

        inner.map.insert(uuid, entry);
        inner.order.push_front(uuid);
    }

    /// Retrieve an entry from the cache. Returns `None` on miss.
    /// Promotes the entry to the front of the LRU order on hit.
    pub fn get(&self, uuid: &Uuid) -> Option<CachedEntry> {
        let mut inner = self.inner.write().unwrap();
        if let Some(entry) = inner.map.get(uuid).cloned() {
            // Hit — promote to front
            inner.order.retain(|u| u != uuid);
            inner.order.push_front(*uuid);
            inner.hits += 1;
            Some(entry)
        } else {
            // Miss
            inner.misses += 1;
            None
        }
    }

    /// Invalidate a single entry (e.g., after update or delete).
    pub fn invalidate(&self, uuid: &Uuid) {
        let mut inner = self.inner.write().unwrap();
        inner.map.remove(uuid);
        inner.order.retain(|u| u != uuid);
    }

    /// Invalidate all cached entries (e.g., after vault save/reload).
    pub fn invalidate_all(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.map.clear();
        inner.order.clear();
    }

    /// Returns the number of entries currently in the cache.
    pub fn len(&self) -> usize {
        self.inner.read().unwrap().map.len()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns cache statistics: (hits, misses, size, capacity).
    pub fn stats(&self) -> CacheStats {
        let inner = self.inner.read().unwrap();
        CacheStats {
            hits: inner.hits,
            misses: inner.misses,
            size: inner.map.len(),
            capacity: inner.capacity,
            hit_rate: if inner.hits + inner.misses > 0 {
                inner.hits as f64 / (inner.hits + inner.misses) as f64
            } else {
                0.0
            },
        }
    }
}

/// Cache performance statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
    pub capacity: usize,
    /// Hit rate as a fraction 0.0–1.0
    pub hit_rate: f64,
}

// ─── Group Cache ──────────────────────────────────────────────────────────────

/// Lightweight cached group for tree rendering.
#[derive(Debug, Clone, Default)]
pub struct CachedGroup {
    pub uuid: Uuid,
    pub parent_uuid: Option<Uuid>,
    pub name: String,
    pub icon_id: u32,
    pub is_expanded: bool,
    pub entry_count: usize,
    pub child_group_count: usize,
}

/// Thread-safe cache for vault groups.
pub struct GroupCache {
    inner: RwLock<HashMap<Uuid, CachedGroup>>,
}

impl GroupCache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn put(&self, uuid: Uuid, group: CachedGroup) {
        self.inner.write().unwrap().insert(uuid, group);
    }

    pub fn get(&self, uuid: &Uuid) -> Option<CachedGroup> {
        self.inner.read().unwrap().get(uuid).cloned()
    }

    pub fn invalidate(&self, uuid: &Uuid) {
        self.inner.write().unwrap().remove(uuid);
    }

    pub fn invalidate_all(&self) {
        self.inner.write().unwrap().clear();
    }

    pub fn all(&self) -> Vec<CachedGroup> {
        self.inner.read().unwrap().values().cloned().collect()
    }
}

// ─── Vault Cache ──────────────────────────────────────────────────────────────

/// Combined vault cache: entries + groups + search index.
pub struct VaultCache {
    pub entries: EntryCache,
    pub groups: GroupCache,
    /// Cached search results: query → list of matching UUIDs
    search_results: RwLock<HashMap<String, Vec<Uuid>>>,
}

impl VaultCache {
    pub fn new() -> Self {
        Self {
            entries: EntryCache::new(500),
            groups: GroupCache::new(),
            search_results: RwLock::new(HashMap::new()),
        }
    }

    /// Cache a search result for a query string.
    pub fn cache_search(&self, query: &str, results: Vec<Uuid>) {
        let mut cache = self.search_results.write().unwrap();
        // Limit search cache to 50 queries
        if cache.len() >= 50 {
            // Remove a random entry (simple eviction)
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }
        cache.insert(query.to_lowercase(), results);
    }

    /// Retrieve cached search results for a query.
    pub fn get_search(&self, query: &str) -> Option<Vec<Uuid>> {
        self.search_results
            .read()
            .unwrap()
            .get(&query.to_lowercase())
            .cloned()
    }

    /// Invalidate all caches (call after any vault mutation).
    pub fn invalidate_all(&self) {
        self.entries.invalidate_all();
        self.groups.invalidate_all();
        self.search_results.write().unwrap().clear();
    }

    /// Invalidate caches for a specific entry (call after entry update/delete).
    pub fn invalidate_entry(&self, uuid: &Uuid) {
        self.entries.invalidate(uuid);
        // Also clear search cache since results may have changed
        self.search_results.write().unwrap().clear();
    }
}

impl Default for VaultCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_cache_put_get() {
        let cache = EntryCache::new(10);
        let uuid = Uuid::new_v4();
        let entry = CachedEntry {
            uuid,
            title: "GitHub".to_string(),
            ..Default::default()
        };

        cache.put(uuid, entry.clone());
        let retrieved = cache.get(&uuid).unwrap();
        assert_eq!(retrieved.title, "GitHub");
    }

    #[test]
    fn test_entry_cache_miss() {
        let cache = EntryCache::new(10);
        let uuid = Uuid::new_v4();
        assert!(cache.get(&uuid).is_none());
    }

    #[test]
    fn test_entry_cache_invalidate() {
        let cache = EntryCache::new(10);
        let uuid = Uuid::new_v4();
        cache.put(
            uuid,
            CachedEntry {
                uuid,
                title: "Test".into(),
                ..Default::default()
            },
        );
        cache.invalidate(&uuid);
        assert!(cache.get(&uuid).is_none());
    }

    #[test]
    fn test_entry_cache_lru_eviction() {
        let cache = EntryCache::new(3);
        let uuids: Vec<Uuid> = (0..4).map(|_| Uuid::new_v4()).collect();

        for &uuid in &uuids[..3] {
            cache.put(
                uuid,
                CachedEntry {
                    uuid,
                    ..Default::default()
                },
            );
        }
        assert_eq!(cache.len(), 3);

        // Access first entry to make it MRU
        cache.get(&uuids[0]);

        // Insert 4th entry — should evict LRU (uuids[1])
        cache.put(
            uuids[3],
            CachedEntry {
                uuid: uuids[3],
                ..Default::default()
            },
        );
        assert_eq!(cache.len(), 3);

        // uuids[0] should still be present (was accessed recently)
        assert!(cache.get(&uuids[0]).is_some());
        // uuids[3] should be present (just inserted)
        assert!(cache.get(&uuids[3]).is_some());
    }

    #[test]
    fn test_entry_cache_invalidate_all() {
        let cache = EntryCache::new(10);
        for _ in 0..5 {
            let uuid = Uuid::new_v4();
            cache.put(
                uuid,
                CachedEntry {
                    uuid,
                    ..Default::default()
                },
            );
        }
        assert_eq!(cache.len(), 5);
        cache.invalidate_all();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = EntryCache::new(10);
        let uuid = Uuid::new_v4();
        cache.put(
            uuid,
            CachedEntry {
                uuid,
                ..Default::default()
            },
        );

        cache.get(&uuid); // hit
        cache.get(&Uuid::new_v4()); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_vault_cache_search() {
        let cache = VaultCache::new();
        let uuids = vec![Uuid::new_v4(), Uuid::new_v4()];

        cache.cache_search("github", uuids.clone());
        let results = cache.get_search("github").unwrap();
        assert_eq!(results.len(), 2);

        // Case-insensitive
        let results2 = cache.get_search("GITHUB").unwrap();
        assert_eq!(results2.len(), 2);
    }

    #[test]
    fn test_vault_cache_invalidate_clears_search() {
        let cache = VaultCache::new();
        let uuid = Uuid::new_v4();
        cache.cache_search("test", vec![uuid]);
        cache.invalidate_entry(&uuid);
        assert!(cache.get_search("test").is_none());
    }
}
