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
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use uuid::Uuid;

use crate::analytics::AnalyticsEngine;
use crate::cache::{CacheConfig, DashMapCache};
use crate::error::{EngineError, EngineResult};
use crate::fts::FullTextSearch;
use crate::storage::column_families::CF_MEMORY_ITEMS;
use crate::storage::rocksdb::RocksDbBackend;
use crate::storage::SharedBackend;
use crate::telemetry::TelemetryCollector;
use crate::vector::VectorIndex;

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
#[derive(Debug, Clone, Default)]
pub struct StorageConfig {
    /// Filesystem path to the storage directory.
    pub path: PathBuf,
    /// Optional cache configuration. When `None`, the default cache
    /// configuration (10 000 entries per entity type) is used.
    pub cache_config: Option<CacheConfig>,
}

/// Configuration for the full Contexter engine, including optional storage
/// tiers for vector indexing (L3), full-text search (L4), and analytics (L5).
///
/// All optional tiers default to **disabled** (`false`). Enable them explicitly
/// when you need the feature.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub storage: StorageConfig,
    // L3: Vector index
    pub enable_vector_index: bool,
    pub vector_dimension: u32,
    pub snapshot_path: Option<PathBuf>,
    /// Maximum number of connections per element in the HNSW graph (M parameter).
    /// Higher values improve recall at the cost of memory and build time.
    /// Currently stored for forward-compatibility; the underlying
    /// `instant_distance` library hardcodes M=32 internally.
    pub hnsw_m: usize,
    /// Number of candidate nearest-neighbours during HNSW construction
    /// (efConstruction parameter). Higher values improve recall at the cost
    /// of longer build time.
    pub hnsw_ef_construction: usize,
    /// Number of candidate nearest-neighbours during HNSW search
    /// (ef parameter). Higher values improve recall at the cost of
    /// slower searches.
    pub hnsw_ef_search: usize,
    // L4: Full-text search
    pub enable_fulltext_search: bool,
    pub tantivy_path: Option<PathBuf>,
    // L5: Analytics
    pub enable_analytics: bool,
    pub analytics_cache_ttl_secs: u64,
    /// Interval in seconds between periodic vector index snapshots
    /// (default: 300 / 5 minutes). Only active when `enable_vector_index`
    /// and `snapshot_path` are both set.
    pub snapshot_interval_secs: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig::default(),
            enable_vector_index: false,
            vector_dimension: 384,
            snapshot_path: None,
            hnsw_m: 16,
            hnsw_ef_construction: 200,
            hnsw_ef_search: 50,
            enable_fulltext_search: false,
            tantivy_path: None,
            enable_analytics: false,
            analytics_cache_ttl_secs: 300,
            snapshot_interval_secs: 300,
        }
    }
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
///
/// # Optional tiers (Phase 2)
///
/// - `vector_index` — ANN vector index (L3, [`VectorIndex`])
/// - `fts_index` — Full-text search index (L4, [`FullTextSearch`])
/// - `analytics_engine` — Analytics aggregation (L5, [`AnalyticsEngine`])
///
/// All three are `None` until explicitly initialised via the [`EngineConfig`].
pub struct Engine {
    pub(crate) storage: SharedBackend,
    pub(crate) cache: DashMapCache,
    pub(crate) telemetry: Arc<TelemetryCollector>,
    // L3: Vector index
    pub(crate) vector_index: Option<Arc<dyn crate::vector::VectorIndex>>,
    // L4: Full-text search
    pub(crate) fts_index: Option<Arc<dyn crate::fts::FullTextSearch>>,
    // L5: Analytics
    pub(crate) analytics_engine: Option<Arc<dyn crate::analytics::AnalyticsEngine>>,
    // Snapshot lifecycle
    pub(crate) snapshot_path: Option<PathBuf>,
    pub(crate) snapshot_handle: Option<JoinHandle<()>>,
    pub(crate) snapshot_cancel: Option<Arc<AtomicBool>>,
}

impl Engine {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Open or create a database at `path` with the default cache configuration
    /// (10 000 entries per entity type) and all optional tiers disabled.
    pub fn open(path: impl AsRef<std::path::Path>) -> EngineResult<Self> {
        let backend = RocksDbBackend::open(path)?;
        let storage: SharedBackend = Arc::new(RwLock::new(Box::new(backend)));
        let cache = DashMapCache::new();
        Ok(Self {
            storage,
            cache,
            telemetry: Arc::new(TelemetryCollector::new()),
            vector_index: None,
            fts_index: None,
            analytics_engine: None,
            snapshot_path: None,
            snapshot_handle: None,
            snapshot_cancel: None,
        })
    }

    /// Open or create a database with the given [`EngineConfig`].
    ///
    /// Controls storage path, cache settings, and enables / disables optional
    /// storage tiers (vector index L3, full-text search L4, analytics L5).
    pub fn with_config(config: EngineConfig) -> EngineResult<Self> {
        // Guard: vector dimension must be >= 1 when the vector index is enabled.
        if config.enable_vector_index && config.vector_dimension == 0 {
            return Err(EngineError::InvalidConfig(
                "embedding_dim must be >= 1".into(),
            ));
        }

        let backend = RocksDbBackend::open(&config.storage.path)?;
        let storage: SharedBackend = Arc::new(RwLock::new(Box::new(backend)));
        let cache = match config.storage.cache_config {
            Some(cfg) => DashMapCache::with_config(cfg),
            None => DashMapCache::new(),
        };

        // L3: Vector index
        let vector_index = if config.enable_vector_index {
            let idx = crate::vector::HnswVectorIndex::new(
                config.vector_dimension as usize,
                config.hnsw_m,
                config.hnsw_ef_construction,
                config.hnsw_ef_search,
            );
            if let Some(ref path) = config.snapshot_path {
                // Try loading existing snapshot — ignore NotFound errors (first run).
                if path.exists() {
                    idx.load_snapshot(path).map_err(|e| {
                        EngineError::Internal(format!("Failed to load vector snapshot: {e}"))
                    })?;
                }
            }
            Some(Arc::new(idx) as Arc<dyn VectorIndex>)
        } else {
            None
        };

        // Startup consistency check: compare L2 memory count with HNSW
        // entry count. A warning is logged if they differ — this may be
        // expected during migration or after a partial snapshot restore.
        if let Some(ref idx) = vector_index {
            let l2_count = {
                let backend = storage.read().map_err(|e| {
                    EngineError::Internal(format!("Failed to acquire storage lock: {e}"))
                })?;
                let keys = backend.scan_cf_keys(CF_MEMORY_ITEMS, "").map_err(|e| {
                    EngineError::Internal(format!("Failed to scan memories CF: {e}"))
                })?;
                keys.len()
            };
            let hnsw_count = idx.len();
            if l2_count != hnsw_count {
                eprintln!(
                    "[contexter] WARNING: L2 memory count ({}) differs from HNSW \
                     entry count ({}). The indexes may be out of sync.",
                    l2_count, hnsw_count
                );
            }
        }

        // L4: Full-text search
        let fts_index = if config.enable_fulltext_search {
            if let Some(ref path) = config.tantivy_path {
                let idx = crate::fts::TantivyIndex::open(path, "memory")
                    .map_err(|e| EngineError::Internal(format!("FTS init: {e}")))?;
                Some(Arc::new(idx) as Arc<dyn FullTextSearch>)
            } else {
                return Err(EngineError::Validation(
                    "enable_fulltext_search requires tantivy_path to be set".into(),
                ));
            }
        } else {
            None
        };

        // L5: Analytics engine
        let analytics_engine = if config.enable_analytics {
            let engine = crate::analytics::DuckDbEngine::new(config.analytics_cache_ttl_secs)
                .map_err(|e| EngineError::Internal(format!("Analytics init: {e}")))?;
            // Wire the storage backend so sync() can iterate real RocksDB data.
            engine.set_storage_backend(Box::new(storage.clone()));
            Some(Arc::new(engine) as Arc<dyn AnalyticsEngine>)
        } else {
            None
        };

        // Start periodic snapshot thread when both vector index and
        // snapshot path are configured.
        let (snapshot_handle, snapshot_cancel) =
            if let (Some(ref idx), Some(ref snap_path)) = (vector_index.clone(), config.snapshot_path.clone()) {
                let cancel = Arc::new(AtomicBool::new(false));
                let cancel_clone = cancel.clone();
                let idx_clone = idx.clone();
                let path = snap_path.clone();
                let interval = config.snapshot_interval_secs.max(1);

                let handle = std::thread::Builder::new()
                    .name("vector-snapshot".into())
                    .spawn(move || {
                        // Give the engine a moment to finish initialisation
                        // before the first snapshot.
                        loop {
                            thread::sleep(Duration::from_secs(interval));
                            if cancel_clone.load(Ordering::Relaxed) {
                                break;
                            }
                            if let Err(e) = idx_clone.save_snapshot(&path) {
                                eprintln!(
                                    "[contexter] periodic snapshot error: {e}"
                                );
                            }
                        }
                    })
                    .map_err(|e| {
                        EngineError::Internal(format!("failed to spawn snapshot thread: {e}"))
                    })?;

                (Some(handle), Some(cancel))
            } else {
                (None, None)
            };

        Ok(Self {
            storage,
            cache,
            telemetry: Arc::new(TelemetryCollector::new()),
            vector_index,
            fts_index,
            analytics_engine,
            snapshot_path: config.snapshot_path.clone(),
            snapshot_handle,
            snapshot_cancel,
        })
    }

    // -------------------------------------------------------------------
    // Shutdown
    // -------------------------------------------------------------------

    /// Gracefully shut down the engine.
    ///
    /// Signals the periodic snapshot thread to stop, waits for it to finish,
    /// then performs a final save of the vector index (if active).
    ///
    /// # Idempotency
    ///
    /// This method is safe to call multiple times.  After the first call the
    /// join handle is consumed via `take()`, so subsequent calls are no-ops
    /// (the cancel flag is already set, the handle is `None`).
    pub fn shutdown(&mut self) -> EngineResult<()> {
        // 1. Signal the periodic snapshot thread to stop.
        if let Some(ref cancel) = self.snapshot_cancel {
            cancel.store(true, Ordering::Relaxed);
        }

        // 2. Join the snapshot thread (safe to call multiple times — take()
        //    returns None on second call).
        if let Some(handle) = self.snapshot_handle.take() {
            handle.join().map_err(|_| {
                EngineError::Internal("snapshot thread panicked".into())
            })?;
        }

        // 3. Final save of the vector index (after the thread has stopped,
        //    so there is no race with a concurrent periodic save).
        if let Some(ref path) = self.snapshot_path {
            if let Some(ref idx) = self.vector_index {
                idx.save_snapshot(path).map_err(|e| {
                    EngineError::Internal(format!("failed to save vector snapshot on shutdown: {e}"))
                })?;
            }
        }

        Ok(())
    }
}

// -----------------------------------------------------------------------
// Drop — ensure shutdown is called when the Engine is dropped.
// -----------------------------------------------------------------------

impl Drop for Engine {
    fn drop(&mut self) {
        // Best-effort shutdown.  Errors are logged via eprintln (already
        // done inside shutdown) and ignored — Drop must not panic.
        let _ = self.shutdown();
    }
}

impl std::fmt::Debug for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Engine").finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::EngineError;

    /// Guard rejects zero vector_dimension when vector index is enabled.
    #[test]
    fn with_config_rejects_zero_dimension_when_vector_enabled() {
        let config = EngineConfig {
            enable_vector_index: true,
            vector_dimension: 0,
            ..EngineConfig::default()
        };
        let result = Engine::with_config(config);
        assert!(result.is_err(), "expected Err for zero dimension with vector enabled");
        match result {
            Err(EngineError::InvalidConfig(msg)) => {
                assert!(
                    msg.contains("embedding_dim must be >= 1"),
                    "error message should mention constraint: {msg}"
                );
            }
            Err(other) => panic!("expected InvalidConfig, got: {other:?}"),
            Ok(_) => panic!("expected Err, got Ok"),
        }
    }

    /// Succeeds when vector_dimension is explicitly set to a valid value.
    #[test]
    fn with_config_succeeds_with_valid_dimension() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let config = EngineConfig {
            storage: StorageConfig {
                path: dir.path().join("contexter.db"),
                cache_config: None,
            },
            enable_vector_index: true,
            vector_dimension: 384,
            ..EngineConfig::default()
        };
        let result = Engine::with_config(config);
        assert!(result.is_ok(), "expected Ok for valid dimension: {result:?}");
    }

    /// Surrenders even when vector_dimension is zero if the vector index is
    /// disabled — the guard only fires when `enable_vector_index` is true.
    #[test]
    fn with_config_skips_guard_when_vector_disabled() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let config = EngineConfig {
            storage: StorageConfig {
                path: dir.path().join("contexter.db"),
                cache_config: None,
            },
            enable_vector_index: false,
            vector_dimension: 0,
            ..EngineConfig::default()
        };
        let result = Engine::with_config(config);
        assert!(result.is_ok(), "expected Ok when vector index disabled: {result:?}");
    }

    /// Default config (vector_dimension = 384, enable_vector_index = false)
    /// should succeed.
    #[test]
    fn with_config_default_config_succeeds() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("temp dir");
        let config = EngineConfig {
            storage: StorageConfig {
                path: dir.path().join("contexter.db"),
                cache_config: None,
            },
            ..EngineConfig::default()
        };
        let result = Engine::with_config(config);
        assert!(result.is_ok(), "expected Ok for default config: {result:?}");
    }
}

