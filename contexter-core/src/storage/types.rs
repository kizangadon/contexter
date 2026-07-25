//! Storage-specific types and configuration.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration for opening a [`super::rocksdb::RocksDbBackend`].
#[derive(Debug, Clone)]
pub struct RocksDbConfig {
    /// Filesystem path to the RocksDB directory.
    pub path: String,
    /// Whether to create the database directory if it does not exist.
    pub create_if_missing: bool,
    /// Whether to synchronously flush the WAL after each write operation.
    ///
    /// When `true` (the default), every mutating operation calls
    /// [`DB::flush_wal(true)`], which issues an `fsync` syscall (1-10 ms per
    /// write). Set to `false` to batch multiple writes before a single explicit
    /// [`super::StorageBackend::checkpoint()`] call, dramatically improving write
    /// throughput at the cost of durability granularity.
    pub wal_sync: bool,
}

impl Default for RocksDbConfig {
    fn default() -> Self {
        Self {
            path: "contexter.db".into(),
            create_if_missing: true,
            wal_sync: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 2 stub types
// ---------------------------------------------------------------------------

/// Filter criteria for vector (kNN) search queries.
/// Phase 2 feature — currently unused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFilter {
    /// Optional list of memory IDs to restrict the search space.
    pub memory_ids: Option<Vec<Uuid>>,
    /// Optional list of tags to filter by.
    pub tags: Option<Vec<String>>,
}

/// A scored memory ID result from a similarity search.
/// Phase 2 feature — currently unused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemoryId {
    /// The matching memory ID.
    pub memory_id: Uuid,
    /// Similarity score (higher is more similar).
    pub score: f64,
}

/// A recorded WAL entry for replay.
/// Phase 2 feature — currently unused.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    /// Log sequence number.
    pub lsn: u64,
    /// Operation type (e.g. "put", "delete").
    pub operation: String,
    /// Column family name.
    pub cf: String,
    /// Key that was written or deleted.
    pub key: Vec<u8>,
    /// Optional value (present for "put" operations).
    pub value: Option<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify RocksDbConfig default values.
    #[test]
    fn rocksdb_config_defaults() {
        let config = RocksDbConfig::default();
        assert_eq!(config.path, "contexter.db");
        assert!(config.create_if_missing);
        assert!(config.wal_sync);
    }

    /// Verify VectorFilter serialization.
    #[test]
    fn vector_filter_serialization() {
        let filter = VectorFilter {
            memory_ids: Some(vec![Uuid::now_v7()]),
            tags: Some(vec!["important".into()]),
        };
        let json = serde_json::to_value(&filter).expect("serialize VectorFilter");
        assert!(json.get("memory_ids").is_some());
        assert!(json.get("tags").is_some());
    }

    /// Verify ScoredMemoryId serialization.
    #[test]
    fn scored_memory_id_serialization() {
        let scored = ScoredMemoryId {
            memory_id: Uuid::now_v7(),
            score: 0.95,
        };
        let json = serde_json::to_value(&scored).expect("serialize ScoredMemoryId");
        assert_eq!(json["score"], 0.95);
    }

    /// Verify WalRecord serialization round-trip.
    #[test]
    fn wal_record_serialization() {
        let record = WalRecord {
            lsn: 42,
            operation: "put".into(),
            cf: "default".into(),
            key: b"my_key".to_vec(),
            value: Some(b"my_value".to_vec()),
        };
        let json = serde_json::to_value(&record).expect("serialize WalRecord");
        assert_eq!(json["lsn"], 42);
        assert_eq!(json["operation"], "put");
        assert_eq!(json["cf"], "default");
    }
}
