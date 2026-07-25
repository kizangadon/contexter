//! DashMap-backed per-type LRU cache (L1 hot cache).

use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;
use std::time::Instant;

use dashmap::DashMap;
use lru::LruCache;

use crate::models::{Agent, Memory, Session, Skill};

use super::metrics::{CacheCounters, CacheTelemetry};
use super::CacheConfig;

/// Map a cache-key prefix to its entity-type name.
fn extract_entity_type(key: &str) -> Option<&str> {
    let prefix = key.split(':').next()?;
    match prefix {
        "ses" => Some("session"),
        "mem" => Some("memory"),
        "agt" => Some("agent"),
        "skl" => Some("skill"),
        "cfg" => Some("setting"),
        "aud" => Some("audit"),
        _ => None,
    }
}

/// A typed value stored inside the per-type [`LruCache`].
#[derive(Debug, Clone)]
pub enum CachedValue {
    /// A cached [`Session`] entity.
    Session(Session),
    /// A cached [`Memory`] entity.
    Memory(Memory),
    /// A cached [`Agent`] entity.
    Agent(Agent),
    /// A cached [`Skill`] entity.
    Skill(Skill),
    /// Raw byte payload for non-domain data (e.g. settings).
    Raw(Vec<u8>),
}

/// A single entry stored inside the per-type [`LruCache`].
struct CacheEntry {
    data: CachedValue,
    inserted_at: Instant,
}

/// Per-type LRU hot cache (L1) backed by DashMap for concurrent access.
pub struct DashMapCache {
    inner: DashMap<String, LruCache<String, CacheEntry>>,
    config: CacheConfig,
    counters: CacheCounters,
}

impl DashMapCache {
    /// Create a new cache with default configuration.
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
            config: CacheConfig::default(),
            counters: CacheCounters::default(),
        }
    }

    /// Create a new cache with the given configuration.
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            inner: DashMap::new(),
            config,
            counters: CacheCounters::default(),
        }
    }

    /// Retrieve a value by key.
    pub fn get(&self, key: &str) -> Option<CachedValue> {
        self.counters.gets.fetch_add(1, Ordering::Relaxed);
        let entity_type = extract_entity_type(key)?;

        if let Some(mut bucket) = self.inner.get_mut(entity_type) {
            if let Some(entry) = bucket.get(key) {
                if self.config.max_ttl.is_some()
                    && entry.inserted_at.elapsed()
                        > self.config.max_ttl.unwrap()
                {
                    // TTL expired — remove and treat as miss
                    bucket.pop(key);
                    self.counters.misses.fetch_add(1, Ordering::Relaxed);
                    return None;
                }
                self.counters.hits.fetch_add(1, Ordering::Relaxed);
                return Some(entry.data.clone());
            }
        }
        self.counters.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Store a value by key.
    pub fn store(&self, key: &str, value: CachedValue) {
        self.counters.stores.fetch_add(1, Ordering::Relaxed);
        let entity_type = match extract_entity_type(key) {
            Some(t) => t,
            None => return,
        };

        let capacity = self
            .config
            .per_type_capacity
            .get(entity_type)
            .copied()
            .unwrap_or(self.config.default_capacity);

        let cap = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1).unwrap());

        let mut bucket = self.inner.entry(entity_type.to_string()).or_insert_with(|| {
            LruCache::new(cap)
        });

        // Update capacity if config changed since creation.
        bucket.resize(cap);

        let entry = CacheEntry {
            data: value,
            inserted_at: Instant::now(),
        };
        bucket.put(key.to_string(), entry);
    }

    /// Remove a value by key.
    pub fn invalidate(&self, key: &str) {
        self.counters.invalidations.fetch_add(1, Ordering::Relaxed);
        if let Some(entity_type) = extract_entity_type(key) {
            if let Some(mut bucket) = self.inner.get_mut(entity_type) {
                bucket.pop(key);
            }
        }
    }

    /// Check if a key exists in the cache.
    pub fn contains(&self, key: &str) -> bool {
        let entity_type = match extract_entity_type(key) {
            Some(t) => t,
            None => return false,
        };
        if let Some(bucket) = self.inner.get(entity_type) {
            bucket.contains(key)
        } else {
            false
        }
    }

    /// Clear all entries for a given entity type.
    pub fn clear_type(&self, entity_type: &str) {
        if let Some(mut bucket) = self.inner.get_mut(entity_type) {
            bucket.clear();
        }
    }

    /// Return the number of entries for a given entity type.
    pub fn type_size(&self, entity_type: &str) -> usize {
        if let Some(bucket) = self.inner.get(entity_type) {
            bucket.len()
        } else {
            0
        }
    }

    /// Return a snapshot of cache telemetry.
    pub fn telemetry(&self) -> CacheTelemetry {
        let (gets, hits, misses, stores, invalidations) = self.counters.snapshot();
        let mut entries_by_type = std::collections::HashMap::new();
        for entry in self.inner.iter() {
            entries_by_type.insert(entry.key().clone(), entry.value().len() as u64);
        }
        CacheTelemetry {
            gets,
            hits,
            misses,
            stores,
            invalidations,
            total_ops: gets,
            entries_by_type,
        }
    }

    /// Clear all entries from the cache.
    pub fn clear_all(&self) {
        self.inner.clear();
    }
}

impl Default for DashMapCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Session;

    fn make_session() -> Session {
        Session {
            id: uuid::Uuid::now_v7(),
            project: "test".into(),
            agent_id: uuid::Uuid::now_v7(),
            status: crate::models::SessionStatus::Active,
            turn_count: 0,
            duration_ms: 0,
            metadata: serde_json::Value::Object(Default::default()),
            efficiency_score: None,
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        }
    }

    // --------------------------------------------------------------
    // Basic store / get / invalidate
    // --------------------------------------------------------------

    #[test]
    fn test_cache_store_get_roundtrip() {
        let cache = DashMapCache::new();
        let session = make_session();
        cache.store("ses:1", CachedValue::Session(session.clone()));
        let retrieved = cache.get("ses:1").expect("should find key");
        match retrieved {
            CachedValue::Session(s) => assert_eq!(s.id, session.id),
            _ => panic!("expected Session variant"),
        }
    }

    #[test]
    fn test_cache_get_missing() {
        let cache = DashMapCache::new();
        assert!(cache.get("ses:nonexistent").is_none());
    }

    #[test]
    fn test_cache_invalidate_removes_entry() {
        let cache = DashMapCache::new();
        cache.store("mem:42", CachedValue::Raw(b"data".to_vec()));
        assert!(cache.contains("mem:42"));
        cache.invalidate("mem:42");
        assert!(!cache.contains("mem:42"));
    }

    #[test]
    fn test_cache_write_through_then_get() {
        let cache = DashMapCache::new();
        // Simulate write-through: wrap in CachedValue::Raw
        let raw = serde_json::to_vec(&"write-through-value").unwrap();
        cache.store("ses:w1", CachedValue::Raw(raw.clone()));
        let got = cache.get("ses:w1");
        assert!(matches!(got, Some(CachedValue::Raw(ref v)) if v == &raw));
    }

    // --------------------------------------------------------------
    // contains (peek, no promote)
    // --------------------------------------------------------------

    #[test]
    fn test_cache_contains_does_not_promote() {
        let config = CacheConfig {
            default_capacity: 2,
            per_type_capacity: std::collections::HashMap::new(),
            ..CacheConfig::default()
        };
        let cache = DashMapCache::with_config(config);

        cache.store("ses:a", CachedValue::Raw(b"first".to_vec()));
        cache.store("ses:b", CachedValue::Raw(b"second".to_vec()));

        // contains peeks but does NOT promote
        assert!(cache.contains("ses:a"));

        // This should evict "ses:a" (oldest) because capacity is 2
        // and contains() did not promote it.
        cache.store("ses:c", CachedValue::Raw(b"third".to_vec()));

        assert!(
            cache.get("ses:a").is_none(),
            "ses:a should have been evicted (contains did not promote)"
        );
        assert!(
            matches!(cache.get("ses:b"), Some(CachedValue::Raw(ref v)) if v == b"second")
        );
        assert!(
            matches!(cache.get("ses:c"), Some(CachedValue::Raw(ref v)) if v == b"third")
        );
    }

    // --------------------------------------------------------------
    // Bulk operations
    // --------------------------------------------------------------

    #[test]
    fn test_cache_clear_type() {
        let cache = DashMapCache::new();
        cache.store("ses:1", CachedValue::Raw(b"s1".to_vec()));
        cache.store("mem:1", CachedValue::Raw(b"m1".to_vec()));
        cache.store("agt:1", CachedValue::Raw(b"a1".to_vec()));

        cache.clear_type("session");
        assert!(cache.get("ses:1").is_none());
        assert!(matches!(cache.get("mem:1"), Some(CachedValue::Raw(ref v)) if v == b"m1"));
        assert!(matches!(cache.get("agt:1"), Some(CachedValue::Raw(ref v)) if v == b"a1"));
    }

    #[test]
    fn test_cache_clear_all() {
        let cache = DashMapCache::new();
        cache.store("ses:1", CachedValue::Raw(b"s1".to_vec()));
        cache.store("mem:1", CachedValue::Raw(b"m1".to_vec()));
        cache.store("agt:1", CachedValue::Raw(b"a1".to_vec()));

        cache.clear_all();
        assert!(cache.get("ses:1").is_none());
        assert!(cache.get("mem:1").is_none());
        assert!(cache.get("agt:1").is_none());
        assert_eq!(cache.type_size("session"), 0);
        assert_eq!(cache.type_size("memory"), 0);
        assert_eq!(cache.type_size("agent"), 0);
    }

    // --------------------------------------------------------------
    // Telemetry & hit / miss tracking
    // --------------------------------------------------------------

    #[test]
    fn test_cache_empty_telemetry() {
        let cache = DashMapCache::new();
        let telem = cache.telemetry();
        assert_eq!(telem.hits, 0);
        assert_eq!(telem.misses, 0);
        assert_eq!(telem.total_ops, 0);
        assert!(telem.entries_by_type.is_empty());
    }

    #[test]
    fn test_cache_telemetry_tracks_hits_and_misses() {
        let cache = DashMapCache::new();
        // 1 miss (get increments gets + misses)
        assert!(cache.get("ses:does-not-exist").is_none());
        // 1 hit
        cache.store("ses:exists", CachedValue::Raw(b"present".to_vec()));
        assert!(cache.get("ses:exists").is_some());
        // 1 more hit
        assert!(cache.get("ses:exists").is_some());
        // 1 more miss
        assert!(cache.get("mem:ghost").is_none());

        let telem = cache.telemetry();
        assert_eq!(telem.hits, 2, "expected 2 hits");
        assert_eq!(telem.misses, 2, "expected 2 misses");
        assert_eq!(telem.gets, 4);
    }

    #[test]
    fn test_cache_hit_ratio() {
        let cache = DashMapCache::new();
        // 3 misses
        assert!(cache.get("ses:x").is_none());
        assert!(cache.get("mem:y").is_none());
        assert!(cache.get("agt:z").is_none());

        // 2 hits
        cache.store("ses:x", CachedValue::Raw(b"x".to_vec()));
        cache.store("mem:y", CachedValue::Raw(b"y".to_vec()));
        assert!(cache.get("ses:x").is_some());
        assert!(cache.get("mem:y").is_some());

        // 5 total lookups (3 misses + 2 hits)
        let telem = cache.telemetry();
        assert_eq!(telem.hits, 2);
        assert_eq!(telem.misses, 3);
        assert_eq!(telem.gets, 5);
    }

    // --------------------------------------------------------------
    // LRU eviction & type isolation
    // --------------------------------------------------------------

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            default_capacity: 2,
            per_type_capacity: std::collections::HashMap::new(), // avoid defaults overriding
            ..CacheConfig::default()
        };
        let cache = DashMapCache::with_config(config);
        cache.store("mem:1", CachedValue::Raw(b"a".to_vec()));
        cache.store("mem:2", CachedValue::Raw(b"b".to_vec()));
        cache.store("mem:3", CachedValue::Raw(b"c".to_vec()));
        // mem:1 should be evicted (LRU)
        assert!(cache.get("mem:1").is_none(), "mem:1 should be evicted");
        assert!(cache.get("mem:2").is_some(), "mem:2 should still exist");
        assert!(cache.get("mem:3").is_some(), "mem:3 should still exist");
    }

    #[test]
    fn test_cache_per_type_isolation() {
        let cache = DashMapCache::new();
        cache.store("ses:1", CachedValue::Raw(b"session".to_vec()));
        cache.store("mem:1", CachedValue::Raw(b"memory".to_vec()));
        // Each type should have 1 entry.
        assert_eq!(cache.type_size("session"), 1);
        assert_eq!(cache.type_size("memory"), 1);
    }

    #[test]
    fn test_cache_multiple_types_independent() {
        let cache = DashMapCache::new();

        cache.store("ses:1", CachedValue::Raw(b"session_data".to_vec()));
        cache.store("mem:1", CachedValue::Raw(b"memory_data".to_vec()));
        cache.store("agt:1", CachedValue::Raw(b"agent_data".to_vec()));
        cache.store("skl:1", CachedValue::Raw(b"skill_data".to_vec()));
        cache.store("cfg:1", CachedValue::Raw(b"setting_data".to_vec()));
        cache.store("aud:1", CachedValue::Raw(b"audit_data".to_vec()));

        assert!(
            matches!(cache.get("ses:1"), Some(CachedValue::Raw(ref v)) if v == b"session_data")
        );
        assert!(
            matches!(cache.get("mem:1"), Some(CachedValue::Raw(ref v)) if v == b"memory_data")
        );
        assert!(
            matches!(cache.get("agt:1"), Some(CachedValue::Raw(ref v)) if v == b"agent_data")
        );
        assert!(
            matches!(cache.get("skl:1"), Some(CachedValue::Raw(ref v)) if v == b"skill_data")
        );
        assert!(
            matches!(cache.get("cfg:1"), Some(CachedValue::Raw(ref v)) if v == b"setting_data")
        );
        assert!(
            matches!(cache.get("aud:1"), Some(CachedValue::Raw(ref v)) if v == b"audit_data")
        );

        assert_eq!(cache.type_size("session"), 1);
        assert_eq!(cache.type_size("memory"), 1);
        assert_eq!(cache.type_size("agent"), 1);
        assert_eq!(cache.type_size("skill"), 1);
        assert_eq!(cache.type_size("setting"), 1);
        assert_eq!(cache.type_size("audit"), 1);

        // Clear one type — others are untouched.
        cache.clear_type("memory");
        assert!(cache.get("mem:1").is_none());
        assert!(
            matches!(cache.get("ses:1"), Some(CachedValue::Raw(ref v)) if v == b"session_data")
        );
        assert!(
            matches!(cache.get("agt:1"), Some(CachedValue::Raw(ref v)) if v == b"agent_data")
        );
    }

    // --------------------------------------------------------------
    // Concurrent access
    // --------------------------------------------------------------

    #[test]
    fn test_cache_concurrent_access() {
        let cache = std::sync::Arc::new(DashMapCache::new());
        let mut handles = Vec::new();

        for i in 0..4 {
            let cache = std::sync::Arc::clone(&cache);
            handles.push(std::thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("ses:thread-{i}-{j}");
                    let value = format!("value-{i}-{j}");
                    cache.store(&key, CachedValue::Raw(value.into_bytes()));
                    // Read back to exercise the get path
                    let _ = cache.get(&key);
                }
            }));
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // 4 threads × 100 unique keys all under "session" type
        assert_eq!(cache.type_size("session"), 400);
    }

    // --------------------------------------------------------------
    // Edge cases
    // --------------------------------------------------------------

    #[test]
    fn test_cache_unknown_prefix_does_not_panic() {
        let cache = DashMapCache::new();
        cache.store("unknown:1", CachedValue::Raw(b"nope".to_vec()));
        assert!(cache.get("unknown:1").is_none());
        assert!(!cache.contains("unknown:1"));
        cache.invalidate("unknown:1");
    }

    #[test]
    fn test_cache_empty_key_prefix() {
        let cache = DashMapCache::new();
        cache.store(":", CachedValue::Raw(b"empty prefix".to_vec()));
        assert!(cache.get(":").is_none());
    }

    #[test]
    fn test_cache_invalidate_nonexistent_key() {
        let cache = DashMapCache::new();
        cache.invalidate("ses:totally-not-there");
    }

    #[test]
    fn test_cache_clear_nonexistent_type() {
        let cache = DashMapCache::new();
        cache.clear_type("nonexistent");
    }

    #[test]
    fn test_cache_type_size_nonexistent() {
        let cache = DashMapCache::new();
        assert_eq!(cache.type_size("nonexistent"), 0);
    }

    #[test]
    fn test_cache_contains_after_invalidate() {
        let cache = DashMapCache::new();
        cache.store("agt:42", CachedValue::Raw(b"agent-42".to_vec()));
        assert!(cache.contains("agt:42"));
        cache.invalidate("agt:42");
        assert!(!cache.contains("agt:42"));
    }

    #[test]
    fn test_cache_telemetry_after_clear() {
        let cache = DashMapCache::new();
        cache.store("ses:1", CachedValue::Raw(b"a".to_vec()));
        cache.store("mem:1", CachedValue::Raw(b"b".to_vec()));
        assert_eq!(cache.telemetry().entries_by_type.len(), 2);

        cache.clear_type("session");
        let telem = cache.telemetry();
        assert_eq!(telem.entries_by_type.get("session").copied(), Some(0));
        assert_eq!(telem.entries_by_type.get("memory").copied(), Some(1));
    }

    #[test]
    fn test_cache_clone_value_independence() {
        let cache = DashMapCache::new();
        let original = CachedValue::Raw(b"original".to_vec());
        cache.store("ses:1", original.clone());

        let mut retrieved = match cache.get("ses:1").unwrap() {
            CachedValue::Raw(v) => v,
            _ => panic!("expected Raw"),
        };
        retrieved[0] = b'm';

        let retrieved2 = match cache.get("ses:1").unwrap() {
            CachedValue::Raw(v) => v,
            _ => panic!("expected Raw"),
        };
        assert_eq!(
            retrieved2, b"original",
            "mutating the returned Vec should not affect the cache"
        );
    }
}
