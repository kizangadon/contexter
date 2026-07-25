//! Contexter Core — foundational storage engine for the Contexter platform.
//!
//! This crate defines the data types, error types, storage backend trait,
//! compression utilities, and unified engine API used by all higher layers.
//!
//! # Architecture
//!
//! - `models` — Domain entities (DDD per-type files)
//! - `error` — Unified error type (`EngineError`) via thiserror
//! - `storage` — `StorageBackend` trait + RocksDB implementation
//! - `compression` — Compression trait + Zstd/LZ4/Noop implementations
//! - `engine` — Unified API composing storage + cache layers
//! - `cache` — Per-type LRU hot cache (L1) backed by DashMap
//! - `bridge` — PyO3 bridge for Python callers
//! - `wal` — Write-ahead log wrapper over RocksDB WAL
//! - `telemetry` — Self-observability (metrics, tracing)
//! - `crdt` — CRDT conflict resolution (LWW-Register)
//! - `versioning` — Content-addressed version store
//! - `util` — Shared helpers (UUID, time)
//! - `vector` — Vector indexing (Phase 2 stub)
//! - `fts` — Full-text search (Phase 2 stub)
//! - `analytics` — Analytics aggregation (Phase 2 stub)

pub mod cache;
pub mod cli;
pub mod compression;
pub mod engine;
pub mod error;
pub mod models;
pub mod storage;

#[cfg(feature = "python")]
pub mod bridge;
pub mod crdt;
pub mod telemetry;
pub mod util;
pub mod versioning;
pub mod wal;

// Phase 2 stubs
pub mod analytics;
pub mod fts;
pub mod vector;

// Re-export key types for convenience.
pub use cache::{CacheConfig, CacheTelemetry, DashMapCache};
pub use engine::{Engine, EngineConfig, EngineStats, StorageConfig};
pub use error::*;
pub use models::{
    Agent, AgentFilter, AgentPatch, AgentStatus, AuditEntry, AuditFilter, Correlation, Feedback,
    Memory, MemoryFilter, MemoryPatch, MemorySearchQuery, MemoryType, NewAgent, NewAuditEntry,
    NewMemory, NewSession, NewSkill, Notification, Session, SessionFilter, SessionPatch,
    SessionStatus, Skill, SkillFilter, SkillPatch, StorageSize, TelemetryEvent,
};
pub use storage::StorageBackend;

// ---------------------------------------------------------------------------
// Compile-time verification that key types are re-exported at crate root.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod re_export_tests {
    #[test]
    fn test_engine_error_accessible() {
        // EngineError and EngineResult must be accessible at crate root
        fn _assert(_: crate::EngineResult<crate::EngineError>) {}
        let err = crate::EngineError::Internal("check".into());
        assert_eq!(err.to_string(), "Internal error: check");
    }

    #[test]
    fn test_model_types_accessible() {
        // Key model types must be accessible at crate root
        let _status = crate::SessionStatus::Active;
        let _mtype = crate::MemoryType::Fact;
        let _astatus = crate::AgentStatus::Active;
        let _size = crate::StorageSize {
            total: 0,
            wal_size: 0,
            per_cf: std::collections::HashMap::new(),
        };
    }
}