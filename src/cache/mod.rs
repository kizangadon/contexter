//! Per-type LRU hot cache tier (L1) using DashMap for concurrent access.
//!
//! Each entity type (session, memory, agent, skill, setting, audit) gets its
//! own [`LruCache`] instance inside a [`DashMap`] so that one type cannot crowd
//! out another.  Cache keys follow the same scheme as RocksDB keys
//! (e.g. `"ses:{uuid}"`, `"mem:{uuid}"`).
//!
//! # Policies
//!
//! * **Write-through** — after a successful storage write, call
//!   [`DashMapCache::store`] immediately with the serialised bytes.
//! * **Write-around** — on update, [`DashMapCache::invalidate`] evicts the stale
//!   entry so the next read re-fetches from the backing store.
//! * **Invalidate on delete** — call [`DashMapCache::invalidate`] when an entity
//!   is removed.
//! * **Populate on miss** — the reader calls [`DashMapCache::store`] after a
//!   successful RocksDB fetch to warm the cache.

use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use lru::LruCache;
use serde::Serialize;

use crate::types::{Agent, Memory, Session, Skill};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a cache-key prefix to its entity-type name.
///
/// Every cache entry has the form `"{prefix}:{id}"`.  The prefix determines
/// which per-type [`LruCache`] bucket the entry lives in.
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

// ---------------------------------------------------------------------------
// CachedValue
// ---------------------------------------------------------------------------

/// A typed value stored inside the per-type [`LruCache`].
///
/// Domain objects are stored directly so cache hits avoid JSON
/// deserialization overhead.  The [`Raw`] variant exists for
/// non-domain data such as settings (key–value strings).
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

// ---------------------------------------------------------------------------
// CacheEntry
// ---------------------------------------------------------------------------

/// A single entry stored inside the per-type [`LruCache`].
struct CacheEntry {
    /// The typed domain object or raw payload.
    data: CachedValue,
    /// Wall-clock insertion time (used for TTL stale-tracking).
    inserted_at: Instant,
}

// ---------------------------------------------------------------------------
// CacheConfig
// ---------------------------------------------------------------------------

/// Tuning parameters for [`DashMapCache`].
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default per-type capacity when no override exists (default: `10_000`).
    pub default_capacity: usize,
    /// Per-entity-type capacity overrides.
    ///
    /// Keys are entity-type names (`"session"`, `"memory"`, …).
    pub per_type_capacity: HashMap<String, usize>,
    /// Optional maximum TTL for cache entries.
    ///
    /// When set, entries older than this duration are evicted on the next
    /// [`DashMapCache::get`] access (lazy eviction).  `None` means entries
    /// live until LRU eviction or explicit invalidation.
    pub max_ttl: Option<Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_capacity: 10_000,
            per_type_capacity: HashMap::new(),
            max_ttl: None,
        }
    }
}

// ---------------------------------------------------------------------------
// CacheTelemetry
// ---------------------------------------------------------------------------

/// Snapshot of cache performance counters.
#[derive(Debug, Clone, Serialize)]
pub struct CacheTelemetry {
    /// Total cache hits since creation.
    pub hits: u64,
    /// Total cache misses since creation.
    pub misses: u64,
    /// Total lookup operations (`hits + misses`).
    pub total_ops: u64,
    /// Ratio of hits to total lookups (`[0.0, 1.0]`).
    pub hit_ratio: f64,
    /// Number of entries per entity type.
    pub entries_by_type: HashMap<String, usize>,
}

// ---------------------------------------------------------------------------
// DashMapCache
// ---------------------------------------------------------------------------

/// A per-type LRU hot cache backed by [`DashMap`] for lock-free concurrent
/// access.
///
/// Each recognised entity type gets its own [`LruCache`] instance, so filling
/// one type to capacity does **not** evict entries of another type.
///
/// # Thread safety
///
/// `DashMapCache` is `Send + Sync`.  Use `Arc<DashMapCache>` for shared
/// ownership across threads.
pub struct DashMapCache {
    /// Per-entity-type LRU caches indexed by entity-type name.
    inner: DashMap<String, LruCache<String, CacheEntry>>,
    /// Configuration (capacities).
    config: CacheConfig,
    /// Running hit counter.
    hits: AtomicU64,
    /// Running miss counter.
    misses: AtomicU64,
}

impl DashMapCache {
    /// Create a cache with the default configuration (10 000 entries per type).
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a cache with a custom [`CacheConfig`].
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            inner: DashMap::new(),
            config,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    // -----------------------------------------------------------------------
    // Core operations
    // -----------------------------------------------------------------------

    /// Look up `key` in the cache.
    ///
    /// Returns the cached value on a hit, or `None` on a miss.  A hit promotes
    /// the entry inside the LRU (marks it as recently used).
    ///
    /// When [`CacheConfig::max_ttl`] is set, entries older than the TTL are
    /// lazily evicted on access (expired entries are removed and treated as a
    /// miss).
    ///
    /// The returned [`CachedValue`] is a **clone** of the stored value, so
    /// mutating it does **not** affect the cache.
    pub fn get(&self, key: &str) -> Option<CachedValue> {
        let entity_type = extract_entity_type(key);
        let result = entity_type
            .and_then(|et| self.inner.get_mut(et))
            .and_then(|mut cache| {
                // Lazy TTL eviction: check without promoting the entry.
                if let Some(ref max_ttl) = self.config.max_ttl {
                    let expired = cache
                        .peek(key)
                        .is_some_and(|e| e.inserted_at.elapsed() > *max_ttl);
                    if expired {
                        cache.pop(key);
                        return None;
                    }
                }
                cache.get(key).map(|e| e.data.clone())
            });

        if result.is_some() {
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    /// Insert a [`CachedValue`] at `key`.
    ///
    /// If the per-type [`LruCache`] does not exist yet it is created on demand
    /// with the appropriate capacity.  If the cache is at capacity the least
    /// recently used entry is evicted.
    pub fn store(&self, key: &str, value: CachedValue) {
        let Some(entity_type) = extract_entity_type(key) else {
            return;
        };

        let entry = CacheEntry {
            data: value,
            inserted_at: Instant::now(),
        };

        let capacity = self.capacity_for(entity_type);
        self.inner
            .entry(entity_type.to_string())
            .or_insert_with(|| LruCache::new(NonZeroUsize::new(capacity.max(1)).unwrap()))
            .put(key.to_string(), entry);
    }

    /// Remove `key` from the cache.
    ///
    /// Used for write-around invalidation on update and for delete propagation.
    pub fn invalidate(&self, key: &str) {
        let Some(entity_type) = extract_entity_type(key) else {
            return;
        };
        if let Some(mut cache) = self.inner.get_mut(entity_type) {
            cache.pop(key);
        }
    }

    /// Check whether `key` exists in the cache **without** promoting the
    /// entry inside the LRU.
    ///
    /// This is a peek operation — it does not change eviction order.
    pub fn contains(&self, key: &str) -> bool {
        extract_entity_type(key)
            .and_then(|et| self.inner.get(et))
            .is_some_and(|cache| cache.contains(key))
    }

    // -----------------------------------------------------------------------
    // Bulk operations
    // -----------------------------------------------------------------------

    /// Remove all entries for a specific entity type.
    pub fn clear_type(&self, entity_type: &str) {
        if let Some(mut cache) = self.inner.get_mut(entity_type) {
            cache.clear();
        }
    }

    /// Remove **all** entries from every entity type.
    pub fn clear_all(&self) {
        self.inner.clear();
    }

    // -----------------------------------------------------------------------
    // Size queries
    // -----------------------------------------------------------------------

    /// Number of entries currently cached for a given entity type.
    pub fn type_size(&self, entity_type: &str) -> usize {
        self.inner
            .get(entity_type)
            .map(|cache| cache.len())
            .unwrap_or(0)
    }

    /// Total number of entries across all entity types.
    pub fn total_size(&self) -> usize {
        self.inner.iter().map(|e| e.len()).sum()
    }

    // -----------------------------------------------------------------------
    // Telemetry
    // -----------------------------------------------------------------------

    /// Cache hit ratio since creation (`[0.0, 1.0]`).
    pub fn hit_ratio(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }

    /// Cache miss ratio since creation (`[0.0, 1.0]`).
    pub fn miss_ratio(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            misses as f64 / total as f64
        }
    }

    /// Snapshot of all cache performance counters and per-type entry counts.
    pub fn telemetry(&self) -> CacheTelemetry {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_ratio = if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        };

        let mut entries_by_type = HashMap::new();
        for entry in self.inner.iter() {
            entries_by_type.insert(entry.key().clone(), entry.len());
        }

        CacheTelemetry {
            hits,
            misses,
            total_ops: total,
            hit_ratio,
            entries_by_type,
        }
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Resolve the effective capacity for a given entity type.
    fn capacity_for(&self, entity_type: &str) -> usize {
        self.config
            .per_type_capacity
            .get(entity_type)
            .copied()
            .unwrap_or(self.config.default_capacity)
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
    use std::sync::Arc;

    // ------------------------------------------------------------------
    // Basic store / get / invalidate
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_store_and_get() {
        let cache = DashMapCache::new();
        cache.store(
            "ses:550e8400-e29b-41d4-a716-446655440000",
            CachedValue::Raw(b"hello world".to_vec()),
        );
        let got = cache.get("ses:550e8400-e29b-41d4-a716-446655440000");
        assert!(matches!(got, Some(CachedValue::Raw(ref v)) if v == b"hello world"));
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let cache = DashMapCache::new();
        assert!(cache.get("ses:00000000-0000-0000-0000-000000000000").is_none());
    }

    #[test]
    fn test_cache_invalidate_removes_entry() {
        let cache = DashMapCache::new();
        cache.store("mem:1", CachedValue::Raw(b"some data".to_vec()));
        assert!(cache.get("mem:1").is_some());

        cache.invalidate("mem:1");
        assert!(
            cache.get("mem:1").is_none(),
            "entry should be gone after invalidate"
        );
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

    // ------------------------------------------------------------------
    // contains (peek, no promote)
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_contains_does_not_promote() {
        let mut per_type = HashMap::new();
        per_type.insert("session".to_string(), 2usize);
        let config = CacheConfig {
            default_capacity: 2,
            per_type_capacity: per_type,
            max_ttl: None,
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

    // ------------------------------------------------------------------
    // Bulk operations
    // ------------------------------------------------------------------

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
        assert_eq!(cache.total_size(), 0);
    }

    // ------------------------------------------------------------------
    // Telemetry & hit / miss tracking
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_empty_telemetry() {
        let cache = DashMapCache::new();
        let telem = cache.telemetry();
        assert_eq!(telem.hits, 0);
        assert_eq!(telem.misses, 0);
        assert_eq!(telem.total_ops, 0);
        assert!((telem.hit_ratio - 0.0).abs() < f64::EPSILON);
        assert!(telem.entries_by_type.is_empty());
    }

    #[test]
    fn test_cache_telemetry_tracks_hits_and_misses() {
        let cache = DashMapCache::new();
        // 1 miss
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
        assert_eq!(telem.total_ops, 4);
        assert!((telem.hit_ratio - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cache_hit_ratio() {
        let cache = DashMapCache::new();
        // 3 misses
        assert!(cache.get("ses:x").is_none());
        assert!(cache.get("mem:y").is_none());
        assert!(cache.get("agt:z").is_none());
        assert!((cache.hit_ratio() - 0.0).abs() < f64::EPSILON);
        assert!((cache.miss_ratio() - 1.0).abs() < f64::EPSILON);

        // 2 hits
        cache.store("ses:x", CachedValue::Raw(b"x".to_vec()));
        cache.store("mem:y", CachedValue::Raw(b"y".to_vec()));
        assert!(cache.get("ses:x").is_some());
        assert!(cache.get("mem:y").is_some());

        // 5 total lookups (3 misses + 2 hits)
        assert!((cache.hit_ratio() - 0.4).abs() < f64::EPSILON);
        assert!((cache.miss_ratio() - 0.6).abs() < f64::EPSILON);
    }

    // ------------------------------------------------------------------
    // LRU eviction & type isolation
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_lru_eviction() {
        let mut per_type = HashMap::new();
        per_type.insert("session".to_string(), 3usize);
        let config = CacheConfig {
            default_capacity: 2,
            per_type_capacity: per_type,
            max_ttl: None,
        };
        let cache = DashMapCache::with_config(config);

        cache.store("ses:1", CachedValue::Raw(b"data1".to_vec()));
        cache.store("ses:2", CachedValue::Raw(b"data2".to_vec()));
        cache.store("ses:3", CachedValue::Raw(b"data3".to_vec()));
        // At capacity (3).  Storing a 4th entry evicts the LRU (ses:1).
        cache.store("ses:4", CachedValue::Raw(b"data4".to_vec()));

        assert!(
            cache.get("ses:1").is_none(),
            "LRU eviction: oldest entry should be evicted"
        );
        assert!(
            matches!(cache.get("ses:2"), Some(CachedValue::Raw(ref v)) if v == b"data2")
        );
        assert!(
            matches!(cache.get("ses:3"), Some(CachedValue::Raw(ref v)) if v == b"data3")
        );
        assert!(
            matches!(cache.get("ses:4"), Some(CachedValue::Raw(ref v)) if v == b"data4")
        );
    }

    #[test]
    fn test_cache_type_isolation() {
        let mut per_type = HashMap::new();
        per_type.insert("session".to_string(), 2usize);
        let config = CacheConfig {
            default_capacity: 100,
            per_type_capacity: per_type,
            max_ttl: None,
        };
        let cache = DashMapCache::with_config(config);

        // Fill session to its capacity of 2.
        cache.store("ses:1", CachedValue::Raw(b"s1".to_vec()));
        cache.store("ses:2", CachedValue::Raw(b"s2".to_vec()));

        // The memory cache is independent and has capacity 100.
        cache.store("mem:1", CachedValue::Raw(b"m1".to_vec()));
        cache.store("mem:2", CachedValue::Raw(b"m2".to_vec()));
        cache.store("mem:3", CachedValue::Raw(b"m3".to_vec()));

        // All memory entries should survive.
        assert!(
            matches!(cache.get("mem:1"), Some(CachedValue::Raw(ref v)) if v == b"m1")
        );
        assert!(
            matches!(cache.get("mem:2"), Some(CachedValue::Raw(ref v)) if v == b"m2")
        );
        assert!(
            matches!(cache.get("mem:3"), Some(CachedValue::Raw(ref v)) if v == b"m3")
        );

        // Session still holds exactly its capacity.
        assert_eq!(cache.type_size("session"), 2);
        assert_eq!(cache.type_size("memory"), 3);
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

    // ------------------------------------------------------------------
    // Concurrent access
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_concurrent_access() {
        let cache = Arc::new(DashMapCache::new());
        let mut handles = Vec::new();

        for i in 0..4 {
            let cache = Arc::clone(&cache);
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
        assert_eq!(cache.total_size(), 400);
    }

    // ------------------------------------------------------------------
    // Edge cases
    // ------------------------------------------------------------------

    #[test]
    fn test_cache_unknown_prefix_does_not_panic() {
        let cache = DashMapCache::new();
        // Keys with unrecognised prefixes should be silently ignored
        cache.store("unknown:1", CachedValue::Raw(b"nope".to_vec()));
        assert!(cache.get("unknown:1").is_none());
        assert!(!cache.contains("unknown:1"));
        cache.invalidate("unknown:1"); // should not panic
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
        cache.invalidate("ses:totally-not-there"); // should not panic
    }

    #[test]
    fn test_cache_clear_nonexistent_type() {
        let cache = DashMapCache::new();
        cache.clear_type("nonexistent"); // should not panic
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
        assert!(
            !cache.contains("agt:42"),
            "contains should return false after invalidate"
        );
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
        // Verify that the cache returns independent copies (CachedValue is cloned).
        let cache = DashMapCache::new();
        let original = CachedValue::Raw(b"original".to_vec());
        cache.store("ses:1", original.clone());

        // Mutate the returned Raw payload copy.
        let mut retrieved = match cache.get("ses:1").unwrap() {
            CachedValue::Raw(v) => v,
            _ => panic!("expected Raw"),
        };
        retrieved[0] = b'm';

        // Fetch again — original should be untouched.
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
