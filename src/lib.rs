//! Contexter Core — foundational storage engine for the Contexter platform.
//!
//! This crate defines the data types, error types, storage backend trait,
//! and compression utilities used by all higher layers of the system.
//!
//! # Architecture
//!
//! - `types` — Domain data types (Session, Memory, Agent, Skill, Audit)
//! - `error` — Unified error type (`EngineError`) via thiserror
//! - `storage` — `StorageBackend` trait with full CRUD operations
//! - `compression` — Compression trait + Zstd/LZ4/Noop implementations (feature-gated)
//! - `engine` — Unified API composing `RocksDbBackend` + `DashMapCache` with cache policies
//! - `cache` — Per-type LRU hot cache (L1) backed by DashMap

pub mod cache;
pub mod cli;
pub mod compression;
pub mod engine;
pub mod error;
pub mod storage;
pub mod types;

#[cfg(feature = "python")]
pub mod python;

// Re-export key types for convenience.
pub use cache::{CacheConfig, CacheTelemetry, DashMapCache};
pub use engine::{Engine, EngineStats, StorageConfig};
pub use error::EngineError;
pub use storage::StorageBackend;
pub use types::*;
