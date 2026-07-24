//! Abstract storage backend trait for Contexter.
//!
//! The [`StorageBackend`] trait defines the complete data-access contract
//! that all storage implementations must satisfy. It is synchronous — any
//! async wrapping happens at the PyO3 bridge layer above this crate.
//!
//! # Domain-Driven Design
//!
//! The trait methods use the Ubiquitous Language of the Contexter domain:
//! Sessions, Memories, Agents, Skills, and Audit entries. Each method is
//! self-contained and returns domain types or [`EngineError`].

pub mod rocksdb_backend;

use std::sync::{Arc, RwLock};

use crate::error::EngineResult;
use crate::types::*;
use uuid::Uuid;

/// Shared storage backend for use across threads.
pub type SharedBackend = Arc<RwLock<Box<dyn StorageBackend>>>;

/// Storage backend interface for Contexter.
///
/// Implementations provide durable persistence for all domain entities.
/// The trait is `Send + Sync` so it can be shared across threads.
pub trait StorageBackend: Send + Sync {
    // -----------------------------------------------------------------------
    // Session CRUD
    // -----------------------------------------------------------------------

    /// Create a new session from `NewSession` input.
    fn create_session(&self, session: NewSession) -> EngineResult<Session>;

    /// Retrieve a session by its unique identifier.
    fn get_session(&self, id: Uuid) -> EngineResult<Option<Session>>;

    /// List sessions matching the supplied filter criteria.
    fn list_sessions(&self, filter: &SessionFilter) -> EngineResult<Vec<Session>>;

    /// Partially update an existing session.
    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> EngineResult<Session>;

    /// Permanently delete a session.
    fn delete_session(&self, id: Uuid) -> EngineResult<()>;

    /// Count sessions matching the supplied filter criteria.
    fn count_sessions(&self, filter: &SessionFilter) -> EngineResult<u64>;

    // -----------------------------------------------------------------------
    // Memory CRUD
    // -----------------------------------------------------------------------

    /// Create a new memory from `NewMemory` input.
    fn create_memory(&self, memory: NewMemory) -> EngineResult<Memory>;

    /// Retrieve a memory by its unique identifier.
    fn get_memory(&self, id: Uuid) -> EngineResult<Option<Memory>>;

    /// Search memories using structured query criteria.
    fn search_memories(&self, query: &MemorySearchQuery) -> EngineResult<Vec<Memory>>;

    /// Partially update an existing memory.
    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> EngineResult<Memory>;

    /// Permanently delete a memory.
    fn delete_memory(&self, id: Uuid) -> EngineResult<()>;

    /// Count memories matching the supplied filter criteria.
    fn count_memories(&self, filter: &MemoryFilter) -> EngineResult<u64>;

    // -----------------------------------------------------------------------
    // Agent CRUD
    // -----------------------------------------------------------------------

    /// Register a new agent.
    fn create_agent(&self, agent: NewAgent) -> EngineResult<Agent>;

    /// Retrieve an agent by its unique identifier.
    fn get_agent(&self, id: Uuid) -> EngineResult<Option<Agent>>;

    /// List agents matching the supplied filter criteria.
    fn list_agents(&self, filter: &AgentFilter) -> EngineResult<Vec<Agent>>;

    /// Partially update an existing agent.
    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> EngineResult<Agent>;

    /// Permanently delete an agent.
    fn delete_agent(&self, id: Uuid) -> EngineResult<()>;

    // -----------------------------------------------------------------------
    // Skill CRUD
    // -----------------------------------------------------------------------

    /// Register a new skill.
    fn create_skill(&self, skill: NewSkill) -> EngineResult<Skill>;

    /// Retrieve a skill by its unique identifier.
    fn get_skill(&self, id: Uuid) -> EngineResult<Option<Skill>>;

    /// List skills matching the supplied filter criteria.
    fn list_skills(&self, filter: &SkillFilter) -> EngineResult<Vec<Skill>>;

    /// Partially update an existing skill.
    fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> EngineResult<Skill>;

    /// Permanently delete a skill.
    fn delete_skill(&self, id: Uuid) -> EngineResult<()>;

    // -----------------------------------------------------------------------
    // Settings (generic key-value store)
    // -----------------------------------------------------------------------

    /// Retrieve a setting value by key.
    fn get_setting(&self, key: &str) -> EngineResult<Option<String>>;

    /// Persist a setting value.
    fn set_setting(&self, key: &str, value: &str) -> EngineResult<()>;

    // -----------------------------------------------------------------------
    // Audit log
    // -----------------------------------------------------------------------

    /// Append a new entry to the audit log.
    fn append_audit_entry(&self, entry: &NewAuditEntry) -> EngineResult<()>;

    /// Query the audit log with optional filters.
    fn query_audit_log(&self, filter: &AuditFilter) -> EngineResult<Vec<AuditEntry>>;

    // -----------------------------------------------------------------------
    // Generic key-value access (raw bytes, column-family-aware)
    // -----------------------------------------------------------------------

    /// Store an arbitrary key-value pair in the given column family.
    fn store_raw(&self, cf: &str, key: &str, value: &[u8]) -> EngineResult<()>;

    /// Retrieve a value by key from the given column family.
    fn get_raw(&self, cf: &str, key: &str) -> EngineResult<Option<Vec<u8>>>;

    /// Write multiple key-value pairs atomically in the given column family.
    ///
    /// This is more efficient than N individual `store_raw` calls because the
    /// entries are grouped into a single RocksDB `WriteBatch` and committed
    /// with one WAL write.
    fn write_batch(&self, cf: &str, entries: Vec<(String, Vec<u8>)>) -> EngineResult<()>;

    /// Return all keys in a column family that start with the given prefix.
    ///
    /// Keys are returned in RocksDB iteration order.  This is useful for
    /// chunked iteration at the Engine layer where callers want to release
    /// the `SharedBackend` read lock between batches.
    fn scan_cf_keys(&self, cf: &str, prefix: &str) -> EngineResult<Vec<Vec<u8>>>;

    // -----------------------------------------------------------------------
    // Maintenance
    // -----------------------------------------------------------------------

    /// Flush any pending writes to durable storage.
    fn flush(&self) -> EngineResult<()>;

    /// Trigger a checkpoint/compaction and return the resulting sequence number.
    fn checkpoint(&self) -> EngineResult<u64>;

    /// Report storage size information.
    fn storage_size(&self) -> EngineResult<StorageSize>;

    // -----------------------------------------------------------------------
    // Raw storage (for testing and low-level access)
    // -----------------------------------------------------------------------

    /// Store raw bytes under the given `key` in the named column family.
    fn store(&self, cf_name: &str, key: &str, value: &[u8]) -> EngineResult<()>;

    /// Retrieve raw bytes for the given `key` from the named column family.
    fn get(&self, cf_name: &str, key: &str) -> EngineResult<Option<Vec<u8>>>;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the trait is object-safe by constructing a
    /// `Box<dyn StorageBackend>`. This will fail to compile if any method
    /// violates object safety rules.
    #[test]
    fn storage_backend_is_object_safe() {
        fn _assert_object_safe(_: Box<dyn StorageBackend>) {}
        // The mere fact that this compiles proves object safety.
        _ = _assert_object_safe;
    }

    /// Verify that EngineResult is a usable alias.
    #[test]
    fn engine_result_alias_works() {
        fn returns_result() -> EngineResult<()> {
            Ok(())
        }
        assert!(returns_result().is_ok());
    }
}
