//! Maintenance, raw-storage, and telemetry operations on [`Engine`].

use std::collections::HashMap;

use super::Engine;
use crate::cache::CacheTelemetry;
use crate::error::EngineResult;
use crate::models::StorageSize;

use std::sync::atomic::Ordering;

impl Engine {
    /// Flush any pending writes to durable storage.
    pub fn flush(&self) -> EngineResult<()> {
        self.storage.write().unwrap_or_else(|e| e.into_inner()).flush()
    }

    /// Trigger a checkpoint / compaction and return the current RocksDB
    /// sequence number.
    pub fn checkpoint(&self) -> EngineResult<u64> {
        self.storage.write().unwrap_or_else(|e| e.into_inner()).checkpoint()
    }

    /// Report storage size information per column family.
    pub fn storage_size(&self) -> EngineResult<StorageSize> {
        self.storage.read().unwrap_or_else(|e| e.into_inner()).storage_size()
    }

    /// Snapshot of L1 cache performance counters.
    pub fn cache_telemetry(&self) -> CacheTelemetry {
        self.cache.telemetry()
    }

    /// Clear **all** entries from the L1 cache.
    pub fn clear_cache(&self) {
        self.cache.clear_all();
    }

    /// Clear all cached entries for a specific entity type
    /// (e.g. `"session"`, `"memory"`, `"agent"`, `"skill"`).
    pub fn clear_cache_type(&self, entity_type: &str) {
        self.cache.clear_type(entity_type);
    }

    // =======================================================================
    // Generic raw storage (for testing and low-level access)
    // =======================================================================

    /// Store a string value under the given `key` in the named column family.
    pub fn store(&self, cf_name: &str, key: &str, value: &str) -> EngineResult<()> {
        self.telemetry.stats.store_ops.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap_or_else(|e| e.into_inner()).store_raw(cf_name, key, value.as_bytes())
    }

    /// Retrieve a string value for the given `key` from the named column family.
    pub fn get(&self, cf_name: &str, key: &str) -> EngineResult<Option<String>> {
        self.telemetry.stats.get_ops.fetch_add(1, Ordering::Relaxed);
        self.storage
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .get_raw(cf_name, key)?
            .map(|v| {
                String::from_utf8(v)
                    .map_err(|e| crate::error::EngineError::Internal(e.to_string()))
            })
            .transpose()
    }

    // =======================================================================
    // Telemetry
    // =======================================================================

    /// Snapshot engine-wide telemetry counters.
    pub fn stats(&self) -> HashMap<String, u64> {
        self.telemetry.stats.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a temporary Engine.
    fn setup() -> (Engine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        (engine, dir)
    }

    /// Verify flush succeeds on a fresh engine.
    #[test]
    fn test_flush_succeeds() {
        let (engine, _dir) = setup();
        engine.flush().expect("flush");
    }

    /// Verify checkpoint returns a sequence number.
    #[test]
    fn test_checkpoint_returns_seq() {
        let (engine, _dir) = setup();
        // A fresh engine returns a sequence number without panicking
        let _seq = engine.checkpoint().expect("checkpoint");
    }

    /// Verify storage_size returns non-empty results.
    #[test]
    fn test_storage_size_returns_data() {
        let (engine, _dir) = setup();
        let size = engine.storage_size().expect("storage size");
        // Storage should report at least one column family with a size entry
        assert!(
            !size.per_cf.is_empty(),
            "storage size should report at least one column family"
        );
    }

    /// Verify cache_telemetry returns a snapshot.
    #[test]
    fn test_cache_telemetry() {
        let (engine, _dir) = setup();
        let telemetry = engine.cache_telemetry();
        // Fresh engine: all counters start at 0
        assert_eq!(telemetry.gets, 0);
    }
}
