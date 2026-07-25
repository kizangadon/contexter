//! Unified `Engine` API — composes `RocksDbBackend` (L2 durable storage)
//! with `DashMapCache` (L1 hot cache) behind cache policies.
//!
//! # Cache policy
//!
//! | Operation | Policy         | Behaviour                                                |
//! |-----------|----------------|----------------------------------------------------------|
//! | Create    | Write-through  | Persist → cache the serialised result                    |
//! | Read      | Cache-aside    | Check cache → miss → persist → cache result              |
//! | Update    | Write-around   | Persist → invalidate cache entry                         |
//! | Delete    | Invalidate     | Delete → invalidate cache entry                          |
//! | List      | Bypass         | Direct to storage, no caching                            |
//! | Count     | Bypass         | Direct to storage, no caching                            |
//!
//! Every cache key matches the key pattern used by `RocksDbBackend` so that
//! the cache prefix parser (`extract_entity_type`) routes entries into the
//! correct per-type LRU bucket.

pub mod agent;
pub mod analytics;
pub mod export;
pub mod maintenance;
pub mod memory;
pub mod search;
pub mod session;
pub mod settings;
pub mod skill;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use uuid::Uuid;

use crate::cache::{CacheConfig, DashMapCache};
use crate::error::EngineResult;
use crate::storage::rocksdb::RocksDbBackend;
use crate::storage::SharedBackend;
use crate::telemetry::TelemetryCollector;

// ---------------------------------------------------------------------------
// Cache key helpers
// ---------------------------------------------------------------------------
// These MUST match the key prefixes defined in `rocksdb_backend` so that
// `extract_entity_type` inside `DashMapCache` can route entries correctly.

pub(crate) fn session_cache_key(id: &Uuid) -> String {
    format!("ses:{id}")
}

pub(crate) fn memory_cache_key(id: &Uuid) -> String {
    format!("mem:{id}")
}

pub(crate) fn agent_cache_key(id: &Uuid) -> String {
    format!("agt:{id}")
}

pub(crate) fn skill_cache_key(id: &Uuid) -> String {
    format!("skl:{id}")
}

pub(crate) fn setting_cache_key(key: &str) -> String {
    format!("cfg:{key}")
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of entries fetched per batch in chunked iteration.
///
/// Between each batch the `SharedBackend` read lock is released so that
/// concurrent write operations can make progress.
const BATCH_SIZE: usize = 100;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Telemetry counters for engine-wide operation tracking.
///
/// All counters use relaxed atomic ordering — they are observability
/// hints, not synchronisation primitives.
#[derive(Debug, Default)]
pub struct EngineStats {
    /// Total sessions created.
    pub sessions_created: AtomicU64,
    /// Total sessions deleted.
    pub sessions_deleted: AtomicU64,
    /// Total memories created.
    pub memories_created: AtomicU64,
    /// Total memories deleted.
    pub memories_deleted: AtomicU64,
    /// Total memory searches completed.
    pub searches_completed: AtomicU64,
    /// Total raw storage `store` operations.
    pub store_ops: AtomicU64,
    /// Total raw storage `get` operations.
    pub get_ops: AtomicU64,
}

impl EngineStats {
    /// Snapshot the current counter values into a [`HashMap`].
    pub fn snapshot(&self) -> HashMap<String, u64> {
        let mut m = HashMap::new();
        m.insert(
            "sessions_created".into(),
            self.sessions_created.load(Ordering::Relaxed),
        );
        m.insert(
            "sessions_deleted".into(),
            self.sessions_deleted.load(Ordering::Relaxed),
        );
        m.insert(
            "memories_created".into(),
            self.memories_created.load(Ordering::Relaxed),
        );
        m.insert(
            "memories_deleted".into(),
            self.memories_deleted.load(Ordering::Relaxed),
        );
        m.insert(
            "searches_completed".into(),
            self.searches_completed.load(Ordering::Relaxed),
        );
        m.insert("store_ops".into(), self.store_ops.load(Ordering::Relaxed));
        m.insert("get_ops".into(), self.get_ops.load(Ordering::Relaxed));
        m
    }
}

/// Configuration for opening a storage engine instance.
///
/// Controls the storage path and optional cache settings.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Filesystem path to the storage directory.
    pub path: PathBuf,
    /// Optional cache configuration. When `None`, the default cache
    /// configuration (10 000 entries per entity type) is used.
    pub cache_config: Option<CacheConfig>,
}

/// The unified Contexter storage engine.
///
/// Wraps a [`SharedBackend`] (L2 durable storage behind a RwLock) and a
/// [`DashMapCache`] (L1 hot cache) behind a consistent API that applies cache
/// policies transparently to callers.
///
/// # Thread safety
///
/// `Engine` is `Send + Sync`. The inner `SharedBackend` uses an `RwLock` so
/// concurrent reads are allowed while writes are serialised.
pub struct Engine {
    pub(crate) storage: SharedBackend,
    pub(crate) cache: DashMapCache,
    pub(crate) telemetry: Arc<TelemetryCollector>,
}

impl Engine {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Open or create a database at `path` with the default cache configuration
    /// (10 000 entries per entity type).
    pub fn open(path: impl AsRef<std::path::Path>) -> EngineResult<Self> {
        let backend = RocksDbBackend::open(path)?;
        let storage: SharedBackend = Arc::new(RwLock::new(Box::new(backend)));
        let cache = DashMapCache::new();
        Ok(Self {
            storage,
            cache,
            telemetry: Arc::new(TelemetryCollector::new()),
        })
    }

    /// Open or create a database with the given [`StorageConfig`].
    ///
    /// Use this to control the storage path and optionally tune the L1 cache.
    pub fn with_config(config: StorageConfig) -> EngineResult<Self> {
        let backend = RocksDbBackend::open(&config.path)?;
        let storage: SharedBackend = Arc::new(RwLock::new(Box::new(backend)));
        let cache = match config.cache_config {
            Some(cfg) => DashMapCache::with_config(cfg),
            None => DashMapCache::new(),
        };
        Ok(Self {
            storage,
            cache,
            telemetry: Arc::new(TelemetryCollector::new()),
        })
    }
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}

