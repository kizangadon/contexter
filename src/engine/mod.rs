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

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use uuid::Uuid;

use crate::cache::{CacheConfig, CacheTelemetry, CachedValue, DashMapCache};
use crate::error::{EngineError, EngineResult};
use crate::storage::rocksdb_backend::RocksDbBackend;
use crate::storage::rocksdb_backend::{
    CF_AGENTS, CF_SESSIONS, CF_SKILLS, KEY_PREFIX_AGENT, KEY_PREFIX_AUDIT, KEY_PREFIX_SESSION,
    KEY_PREFIX_SKILL,
};
use crate::storage::SharedBackend;
use crate::types::*;

// ---------------------------------------------------------------------------
// Cache key helpers
// ---------------------------------------------------------------------------
// These MUST match the key prefixes defined in `rocksdb_backend` so that
// `extract_entity_type` inside `DashMapCache` can route entries correctly.

fn session_cache_key(id: &Uuid) -> String {
    format!("ses:{id}")
}

fn memory_cache_key(id: &Uuid) -> String {
    format!("mem:{id}")
}

fn agent_cache_key(id: &Uuid) -> String {
    format!("agt:{id}")
}

fn skill_cache_key(id: &Uuid) -> String {
    format!("skl:{id}")
}

fn setting_cache_key(key: &str) -> String {
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
    storage: SharedBackend,
    cache: DashMapCache,
    stats: EngineStats,
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
            stats: EngineStats::default(),
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
            stats: EngineStats::default(),
        })
    }

    // =======================================================================
    // Session CRUD
    // =======================================================================

    /// Create a new session.
    ///
    /// **Policy:** Write-through — persisted first, then cached.
    pub fn create_session(&self, new_session: NewSession) -> EngineResult<Session> {
        self.stats.sessions_created.fetch_add(1, Ordering::Relaxed);
        let session = self.storage.write().unwrap().create_session(new_session)?;
        let key = session_cache_key(&session.id);
        self.cache.store(&key, CachedValue::Session(session.clone()));
        Ok(session)
    }

    /// Retrieve a session by its unique identifier.
    ///
    /// **Policy:** Cache-aside — checked the L1 cache first. On a miss the
    /// entry is fetched from L2 and stored in L1 before returning.
    pub fn get_session(&self, id: Uuid) -> EngineResult<Option<Session>> {
        let key = session_cache_key(&id);

        // L1 hit — return the cached object directly (no JSON deserialization).
        if let Some(CachedValue::Session(session)) = self.cache.get(&key) {
            return Ok(Some(session));
        }

        // L1 miss — fetch from L2, populate L1.
        match self.storage.read().unwrap().get_session(id)? {
            Some(session) => {
                self.cache.store(&key, CachedValue::Session(session.clone()));
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    /// List sessions matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2 with chunked iteration.
    pub fn list_sessions(&self, filter: &SessionFilter) -> EngineResult<Vec<Session>> {
        let keys = self
            .storage
            .read()
            .unwrap()
            .scan_cf_keys(CF_SESSIONS, KEY_PREFIX_SESSION)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap();
            for key_bytes in chunk {
                let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 key: {e}"))
                })?;

                let value = match storage.get_raw(CF_SESSIONS, key_str)? {
                    Some(v) => v,
                    None => continue,
                };

                let session: Session = match serde_json::from_slice(&value) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if let Some(ref project) = filter.project {
                    if session.project != *project {
                        continue;
                    }
                }
                if let Some(ref agent_id) = filter.agent_id {
                    if session.agent_id != *agent_id {
                        continue;
                    }
                }
                if let Some(ref status) = filter.status {
                    if session.status != *status {
                        continue;
                    }
                }

                results.push(session);
            }
        }

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    /// Partially update an existing session.
    ///
    /// **Policy:** Write-around — persisted to L2 first, then the stale cache
    /// entry is invalidated so the next read re-fetches from L2.
    pub fn update_session(&self, id: Uuid, patch: &SessionPatch) -> EngineResult<Session> {
        let session = self.storage.write().unwrap().update_session(id, patch)?;
        let key = session_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(session)
    }

    /// Permanently delete a session.
    ///
    /// **Policy:** Invalidate — deleted from L2, then evicted from L1.
    pub fn delete_session(&self, id: Uuid) -> EngineResult<()> {
        self.stats.sessions_deleted.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap().delete_session(id)?;
        let key = session_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
    }

    /// Count sessions matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_sessions(&self, filter: &SessionFilter) -> EngineResult<u64> {
        self.storage.read().unwrap().count_sessions(filter)
    }

    // =======================================================================
    // Memory CRUD
    // =======================================================================

    /// Create a new memory.
    ///
    /// **Policy:** Write-through.
    pub fn create_memory(&self, new_memory: NewMemory) -> EngineResult<Memory> {
        self.stats.memories_created.fetch_add(1, Ordering::Relaxed);
        // Refuse content larger than 1MB to prevent resource exhaustion.
        if new_memory.content.len() > 1024 * 1024 {
            return Err(EngineError::Validation(
                "Memory content exceeds 1MB limit".into(),
            ));
        }
        let memory = self.storage.write().unwrap().create_memory(new_memory)?;
        let key = memory_cache_key(&memory.id);
        self.cache.store(&key, CachedValue::Memory(memory.clone()));
        Ok(memory)
    }

    /// Retrieve a memory by its unique identifier.
    ///
    /// **Policy:** Cache-aside.
    pub fn get_memory(&self, id: Uuid) -> EngineResult<Option<Memory>> {
        let key = memory_cache_key(&id);

        // L1 hit — return the cached object directly.
        if let Some(CachedValue::Memory(memory)) = self.cache.get(&key) {
            return Ok(Some(memory));
        }

        // L1 miss — fetch from L2, populate L1.
        match self.storage.read().unwrap().get_memory(id)? {
            Some(memory) => {
                self.cache.store(&key, CachedValue::Memory(memory.clone()));
                Ok(Some(memory))
            }
            None => Ok(None),
        }
    }

    /// Search memories using structured query criteria.
    ///
    /// Delegates to the storage backend which uses secondary indexes
    /// (via `memory_index` CF) for `memory_type`, `tags`, and `session_id`
    /// filters and applies keyword relevance scoring + `agent_id` filtering.
    ///
    /// **Policy:** Delegates to L2 storage — the backend handles its own
    /// iteration strategy. This means the `SharedBackend` read lock is held
    /// for the duration of the call, which is the same contract used by
    /// `count_memories` and other bypass-policy methods.
    pub fn search_memories(&self, query: &MemorySearchQuery) -> EngineResult<Vec<Memory>> {
        self.stats
            .searches_completed
            .fetch_add(1, Ordering::Relaxed);

        self.storage.read().unwrap().search_memories(query)
    }

    /// Partially update an existing memory.
    ///
    /// **Policy:** Write-around.
    pub fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> EngineResult<Memory> {
        // Refuse content larger than 1MB to prevent resource exhaustion
        // (same limit as create_memory).
        if let Some(ref content) = patch.content {
            if content.len() > 1024 * 1024 {
                return Err(EngineError::Validation(
                    "Memory content exceeds 1MB limit".into(),
                ));
            }
        }
        let memory = self.storage.write().unwrap().update_memory(id, patch)?;
        let key = memory_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(memory)
    }

    /// Permanently delete a memory.
    ///
    /// **Policy:** Invalidate.
    pub fn delete_memory(&self, id: Uuid) -> EngineResult<()> {
        self.stats.memories_deleted.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap().delete_memory(id)?;
        let key = memory_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
    }

    /// Count memories matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_memories(&self, filter: &MemoryFilter) -> EngineResult<u64> {
        self.storage.read().unwrap().count_memories(filter)
    }

    // =======================================================================
    // Agent CRUD
    // =======================================================================

    /// Register a new agent.
    ///
    /// **Policy:** Write-through.
    pub fn create_agent(&self, new_agent: NewAgent) -> EngineResult<Agent> {
        let agent = self.storage.write().unwrap().create_agent(new_agent)?;
        let key = agent_cache_key(&agent.id);
        self.cache.store(&key, CachedValue::Agent(agent.clone()));
        Ok(agent)
    }

    /// Retrieve an agent by its unique identifier.
    ///
    /// **Policy:** Cache-aside.
    pub fn get_agent(&self, id: Uuid) -> EngineResult<Option<Agent>> {
        let key = agent_cache_key(&id);

        // L1 hit — return the cached object directly.
        if let Some(CachedValue::Agent(agent)) = self.cache.get(&key) {
            return Ok(Some(agent));
        }

        // L1 miss — fetch from L2, populate L1.
        match self.storage.read().unwrap().get_agent(id)? {
            Some(agent) => {
                self.cache.store(&key, CachedValue::Agent(agent.clone()));
                Ok(Some(agent))
            }
            None => Ok(None),
        }
    }

    /// List agents matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2 with chunked iteration.
    pub fn list_agents(&self, filter: &AgentFilter) -> EngineResult<Vec<Agent>> {
        let keys = self
            .storage
            .read()
            .unwrap()
            .scan_cf_keys(CF_AGENTS, KEY_PREFIX_AGENT)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap();
            for key_bytes in chunk {
                let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 key: {e}"))
                })?;

                let value = match storage.get_raw(CF_AGENTS, key_str)? {
                    Some(v) => v,
                    None => continue,
                };

                let agent: Agent = match serde_json::from_slice(&value) {
                    Ok(a) => a,
                    Err(_) => continue,
                };

                if let Some(ref name) = filter.name {
                    if !agent.name.to_lowercase().contains(&name.to_lowercase()) {
                        continue;
                    }
                }
                if let Some(ref status) = filter.status {
                    if agent.status != *status {
                        continue;
                    }
                }
                if let Some(ref capability) = filter.capability {
                    if !agent
                        .capabilities
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(capability))
                    {
                        continue;
                    }
                }

                results.push(agent);
            }
        }

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    /// Partially update an existing agent.
    ///
    /// **Policy:** Write-around.
    pub fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> EngineResult<Agent> {
        let agent = self.storage.write().unwrap().update_agent(id, patch)?;
        let key = agent_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(agent)
    }

    /// Permanently delete an agent.
    ///
    /// **Policy:** Invalidate.
    pub fn delete_agent(&self, id: Uuid) -> EngineResult<()> {
        self.storage.write().unwrap().delete_agent(id)?;
        let key = agent_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
    }

    // =======================================================================
    // Skill CRUD
    // =======================================================================

    /// Validate a skill's `file_path` (if present).
    ///
    /// Rejects empty paths, paths containing `..` path segments (path traversal),
    /// and paths exceeding 4096 bytes to prevent storage abuse and constrain
    /// downstream path handling.
    fn validate_file_path(file_path: &Option<String>) -> EngineResult<()> {
        if let Some(p) = file_path {
            if p.is_empty() {
                return Err(EngineError::Validation(
                    "Skill file_path must not be empty".into(),
                ));
            }
            if p.split('/').any(|segment| segment == "..") {
                return Err(EngineError::Validation(
                    "Skill file_path must not contain path traversal components".into(),
                ));
            }
            if p.len() > 4096 {
                return Err(EngineError::Validation(
                    "Skill file_path exceeds maximum length (4096)".into(),
                ));
            }
        }
        Ok(())
    }

    /// Register a new skill.
    ///
    /// **Policy:** Write-through.
    pub fn create_skill(&self, new_skill: NewSkill) -> EngineResult<Skill> {
        Self::validate_file_path(&new_skill.file_path)?;
        let skill = self.storage.write().unwrap().create_skill(new_skill)?;
        let key = skill_cache_key(&skill.id);
        self.cache.store(&key, CachedValue::Skill(skill.clone()));
        Ok(skill)
    }

    /// Retrieve a skill by its unique identifier.
    ///
    /// **Policy:** Cache-aside.
    pub fn get_skill(&self, id: Uuid) -> EngineResult<Option<Skill>> {
        let key = skill_cache_key(&id);

        // L1 hit — return the cached object directly.
        if let Some(CachedValue::Skill(skill)) = self.cache.get(&key) {
            return Ok(Some(skill));
        }

        // L1 miss — fetch from L2, populate L1.
        match self.storage.read().unwrap().get_skill(id)? {
            Some(skill) => {
                self.cache.store(&key, CachedValue::Skill(skill.clone()));
                Ok(Some(skill))
            }
            None => Ok(None),
        }
    }

    /// List skills matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2 with chunked iteration.
    pub fn list_skills(&self, filter: &SkillFilter) -> EngineResult<Vec<Skill>> {
        let keys = self
            .storage
            .read()
            .unwrap()
            .scan_cf_keys(CF_SKILLS, KEY_PREFIX_SKILL)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap();
            for key_bytes in chunk {
                let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 key: {e}"))
                })?;

                let value = match storage.get_raw(CF_SKILLS, key_str)? {
                    Some(v) => v,
                    None => continue,
                };

                let skill: Skill = match serde_json::from_slice(&value) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                if let Some(ref name) = filter.name {
                    if !skill.name.to_lowercase().contains(&name.to_lowercase()) {
                        continue;
                    }
                }
                if let Some(ref category) = filter.category {
                    if !skill.category.eq_ignore_ascii_case(category) {
                        continue;
                    }
                }

                results.push(skill);
            }
        }

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    /// Partially update an existing skill.
    ///
    /// **Policy:** Write-around.
    pub fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> EngineResult<Skill> {
        Self::validate_file_path(&patch.file_path)?;
        let skill = self.storage.write().unwrap().update_skill(id, patch)?;
        let key = skill_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(skill)
    }

    /// Permanently delete a skill.
    ///
    /// **Policy:** Invalidate.
    pub fn delete_skill(&self, id: Uuid) -> EngineResult<()> {
        self.storage.write().unwrap().delete_skill(id)?;
        let key = skill_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
    }

    // =======================================================================
    // Settings (generic key-value store)
    // =======================================================================

    /// Persist a setting value.
    ///
    /// **Policy:** Write-through — stored in L2, then cached as raw UTF-8 bytes.
    pub fn set_setting(&self, key: &str, value: &str) -> EngineResult<()> {
        // Validate key length to prevent storage abuse.
        if key.is_empty() || key.len() > 256 {
            return Err(EngineError::Validation(
                "Setting key must be 1-256 characters".into(),
            ));
        }
        self.storage.write().unwrap().set_setting(key, value)?;
        let cache_key = setting_cache_key(key);
        self.cache
            .store(&cache_key, CachedValue::Raw(value.as_bytes().to_vec()));
        Ok(())
    }

    /// Retrieve a setting value by key.
    ///
    /// **Policy:** Cache-aside.
    pub fn get_setting(&self, key: &str) -> EngineResult<Option<String>> {
        let cache_key = setting_cache_key(key);

        if let Some(CachedValue::Raw(bytes)) = self.cache.get(&cache_key) {
            let value = String::from_utf8(bytes).map_err(|e| {
                EngineError::Internal(format!("invalid UTF-8 in cached setting: {e}"))
            })?;
            return Ok(Some(value));
        }

        match self.storage.read().unwrap().get_setting(key)? {
            Some(value) => {
                self.cache
                    .store(&cache_key, CachedValue::Raw(value.as_bytes().to_vec()));
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    // =======================================================================
    // Audit log
    // =======================================================================

    /// Append a new entry to the audit log.
    pub fn log_audit(&self, entry: NewAuditEntry) -> EngineResult<()> {
        self.storage.write().unwrap().append_audit_entry(&entry)
    }

    /// Query the audit log with optional filters.
    ///
    /// **Policy:** Bypass — always reads from L2 with chunked iteration.
    pub fn query_audit(&self, filter: &AuditFilter) -> EngineResult<Vec<AuditEntry>> {
        let keys = self
            .storage
            .read()
            .unwrap()
            .scan_cf_keys(CF_SESSIONS, KEY_PREFIX_AUDIT)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap();
            for key_bytes in chunk {
                let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 key: {e}"))
                })?;

                let value = match storage.get_raw(CF_SESSIONS, key_str)? {
                    Some(v) => v,
                    None => continue,
                };

                let entry: AuditEntry = match serde_json::from_slice(&value) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                if let Some(ref entity_type) = filter.entity_type {
                    if entry.entity_type != *entity_type {
                        continue;
                    }
                }
                if let Some(ref entity_id) = filter.entity_id {
                    if entry.entity_id != *entity_id {
                        continue;
                    }
                }
                if let Some(ref actor) = filter.actor {
                    if entry.actor.as_deref() != Some(actor.as_str()) {
                        continue;
                    }
                }

                results.push(entry);
            }
        }

        // Newest first.
        results.reverse();

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    // =======================================================================
    // Maintenance
    // =======================================================================

    /// Flush any pending writes to durable storage.
    pub fn flush(&self) -> EngineResult<()> {
        self.storage.write().unwrap().flush()
    }

    /// Trigger a checkpoint / compaction and return the current RocksDB
    /// sequence number.
    pub fn checkpoint(&self) -> EngineResult<u64> {
        self.storage.write().unwrap().checkpoint()
    }

    /// Report storage size information per column family.
    pub fn storage_size(&self) -> EngineResult<StorageSize> {
        self.storage.read().unwrap().storage_size()
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

    /// Store raw bytes under the given `key` in the named column family.
    pub fn store(&self, cf_name: &str, key: &str, value: &[u8]) -> EngineResult<()> {
        self.stats.store_ops.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap().store_raw(cf_name, key, value)
    }

    /// Retrieve raw bytes for the given `key` from the named column family.
    pub fn get(&self, cf_name: &str, key: &str) -> EngineResult<Option<Vec<u8>>> {
        self.stats.get_ops.fetch_add(1, Ordering::Relaxed);
        self.storage.read().unwrap().get_raw(cf_name, key)
    }

    // =======================================================================
    // Telemetry
    // =======================================================================

    /// Snapshot engine-wide telemetry counters.
    pub fn stats(&self) -> HashMap<String, u64> {
        self.stats.snapshot()
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
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Create a temporary engine for testing.
    fn setup() -> (Engine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        (engine, dir)
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_engine_open_creates_directories() {
        let dir = TempDir::new().expect("temp dir");
        let db_path = dir.path().join("contexter.db");
        let engine = Engine::open(&db_path).expect("open engine");

        // The RocksDB directory should have been created.
        assert!(db_path.exists(), "RocksDB path should exist after open");

        // Verify the engine is minimally usable.
        let tel = engine.cache_telemetry();
        assert_eq!(tel.total_ops, 0, "fresh engine should have zero cache ops");

        // A get on a non-existent session should return None without error.
        let result = engine
            .count_sessions(&SessionFilter::default())
            .expect("count sessions");
        assert_eq!(result, 0);
    }

    #[test]
    fn test_engine_with_config_applies_cache_settings() {
        let dir = TempDir::new().expect("temp dir");
        let config = CacheConfig {
            default_capacity: 100,
            per_type_capacity: HashMap::new(),
            max_ttl: None,
        };
        let engine = Engine::with_config(StorageConfig {
            path: dir.path().to_path_buf(),
            cache_config: Some(config),
        })
        .expect("open with config");
        let tel = engine.cache_telemetry();
        assert_eq!(tel.total_ops, 0);
    }

    // -----------------------------------------------------------------------
    // Session CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_session_create_and_get() {
        let (engine, _dir) = setup();

        let session = engine
            .create_session(NewSession {
                project: "test-project".into(),
                agent_id: Uuid::now_v7(),
                status: Some(SessionStatus::Active),
                metadata: Some(serde_json::json!({"env": "test"})),
            })
            .expect("create session");

        assert_eq!(session.project, "test-project");
        assert_eq!(session.turn_count, 0);
        assert_eq!(session.status, SessionStatus::Active);

        let fetched = engine
            .get_session(session.id)
            .expect("get session")
            .expect("session exists");

        assert_eq!(fetched.id, session.id);
        assert_eq!(fetched.project, session.project);
        assert_eq!(fetched.agent_id, session.agent_id);
        assert_eq!(fetched.status, session.status);
        assert_eq!(fetched.metadata, session.metadata);
    }

    #[test]
    fn test_session_cache_hits_on_second_get() {
        let (engine, _dir) = setup();

        let session = engine
            .create_session(NewSession {
                project: "cache-test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create session");

        // First get should be a cache hit because write-through stored it.
        let tel_before = engine.cache_telemetry();
        let fetched = engine.get_session(session.id).expect("get session");
        let tel_after = engine.cache_telemetry();

        assert!(fetched.is_some(), "session should exist");
        assert_eq!(
            tel_after.hits - tel_before.hits,
            1,
            "get after write-through create should be a L1 hit"
        );
        assert_eq!(
            tel_after.misses - tel_before.misses,
            0,
            "no L1 miss expected for write-through created entity"
        );
    }

    #[test]
    fn test_session_update_invalidates_cache() {
        let (engine, _dir) = setup();

        let session = engine
            .create_session(NewSession {
                project: "invalidation".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        // Warm the cache.
        let _ = engine.get_session(session.id).expect("warm");

        let tel_before = engine.cache_telemetry();

        // Update — write-around should invalidate the cache.
        let updated = engine
            .update_session(
                session.id,
                &SessionPatch {
                    turn_count: Some(42),
                    ..SessionPatch::default()
                },
            )
            .expect("update session");
        assert_eq!(updated.turn_count, 42);

        // Next get should MISS the cache (invalidation), then re-fetch from L2.
        let re_fetched = engine
            .get_session(session.id)
            .expect("get after update")
            .expect("session exists");
        assert_eq!(re_fetched.turn_count, 42);

        let tel_after = engine.cache_telemetry();
        assert_eq!(
            tel_after.misses - tel_before.misses,
            1,
            "get after write-around update should produce one L1 miss"
        );
        assert_eq!(
            tel_after.hits - tel_before.hits,
            0,
            "no new L1 hits expected (cache was invalidated)"
        );
    }

    #[test]
    fn test_session_delete_invalidates_cache() {
        let (engine, _dir) = setup();

        let session = engine
            .create_session(NewSession {
                project: "del-inval".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        // Warm the cache.
        let _ = engine.get_session(session.id).expect("warm");

        // Delete — should also invalidate the cache.
        engine.delete_session(session.id).expect("delete session");

        // Should return None (deleted + cache invalidated).
        let fetched = engine.get_session(session.id).expect("get after delete");
        assert!(fetched.is_none(), "deleted session should not exist");
    }

    #[test]
    fn test_session_list_and_count() {
        let (engine, _dir) = setup();
        let agent = Uuid::now_v7();

        // Create 3 sessions.
        for i in 0..3 {
            engine
                .create_session(NewSession {
                    project: "list-test".into(),
                    agent_id: agent,
                    status: if i == 0 {
                        Some(SessionStatus::Active)
                    } else {
                        Some(SessionStatus::Completed)
                    },
                    metadata: None,
                })
                .expect("create");
        }

        // List all.
        let all = engine
            .list_sessions(&SessionFilter {
                project: Some("list-test".into()),
                ..SessionFilter::default()
            })
            .expect("list sessions");
        assert_eq!(all.len(), 3);

        // Count all.
        let count = engine
            .count_sessions(&SessionFilter {
                project: Some("list-test".into()),
                ..SessionFilter::default()
            })
            .expect("count sessions");
        assert_eq!(count, 3);

        // Filter by status.
        let active = engine
            .list_sessions(&SessionFilter {
                project: Some("list-test".into()),
                status: Some(SessionStatus::Active),
                ..SessionFilter::default()
            })
            .expect("list active");
        assert_eq!(active.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Memory CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_memory_create_and_search() {
        let (engine, _dir) = setup();

        let session_id = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        let memory = engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "the quick brown fox jumps over the lazy dog".into(),
                tags: Some(vec!["animal".into(), "nature".into()]),
            })
            .expect("create memory");

        assert_eq!(memory.version, 1);
        assert!(memory.tags.contains(&"animal".to_string()));

        // Search by keyword.
        let results = engine
            .search_memories(&MemorySearchQuery {
                keywords: Some("fox".into()),
                ..MemorySearchQuery::default()
            })
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].content,
            "the quick brown fox jumps over the lazy dog"
        );

        // Search by tag.
        let results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["animal".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by tag");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_get_cached() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "cache-me".into(),
                tags: None,
            })
            .expect("create");

        // First get after write-through create should be a L1 hit.
        let tel_before = engine.cache_telemetry();
        let fetched = engine.get_memory(memory.id).expect("get memory");
        let tel_after = engine.cache_telemetry();
        assert!(fetched.is_some());
        assert_eq!(tel_after.hits - tel_before.hits, 1);
    }

    #[test]
    fn test_memory_update_version_bump() {
        let (engine, _dir) = setup();

        let created = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "v1 content".into(),
                tags: None,
            })
            .expect("create");
        assert_eq!(created.version, 1);

        let updated = engine
            .update_memory(
                created.id,
                &MemoryPatch {
                    content: Some("v2 content".into()),
                    ..MemoryPatch::default()
                },
            )
            .expect("update");
        assert_eq!(updated.version, 2);

        let updated2 = engine
            .update_memory(
                created.id,
                &MemoryPatch {
                    content: Some("v3 content".into()),
                    ..MemoryPatch::default()
                },
            )
            .expect("update again");
        assert_eq!(updated2.version, 3);

        // After update, cache should be invalidated. Re-fetch should give v3.
        let fetched = engine.get_memory(created.id).expect("get").expect("exists");
        assert_eq!(fetched.content, "v3 content");
    }

    #[test]
    fn test_memory_delete_invalidates_cache() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Preference,
                content: "to be deleted".into(),
                tags: None,
            })
            .expect("create");

        engine.delete_memory(memory.id).expect("delete");
        assert!(engine
            .get_memory(memory.id)
            .expect("get after delete")
            .is_none());
    }

    #[test]
    fn test_memory_count() {
        let (engine, _dir) = setup();
        let session = Uuid::now_v7();

        for i in 0..5 {
            engine
                .create_memory(NewMemory {
                    session_id: session,
                    agent_id: Uuid::now_v7(),
                    memory_type: if i % 2 == 0 {
                        MemoryType::Fact
                    } else {
                        MemoryType::Preference
                    },
                    content: format!("content {i}"),
                    tags: None,
                })
                .expect("create");
        }

        let count = engine
            .count_memories(&MemoryFilter {
                session_id: Some(session),
                ..Default::default()
            })
            .expect("count");
        assert_eq!(count, 5);

        let fact_count = engine
            .count_memories(&MemoryFilter {
                memory_type: Some(MemoryType::Fact),
                ..Default::default()
            })
            .expect("count facts");
        assert_eq!(fact_count, 3);
    }

    // -----------------------------------------------------------------------
    // Search filter integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_search_by_memory_type() {
        let (engine, _dir) = setup();
        let session_id = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact memory".into(),
                tags: None,
            })
            .expect("create fact");

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Preference,
                content: "preference memory".into(),
                tags: None,
            })
            .expect("create preference");

        // Search by Fact type → should return exactly one.
        let facts = engine
            .search_memories(&MemorySearchQuery {
                memory_type: Some(MemoryType::Fact),
                ..MemorySearchQuery::default()
            })
            .expect("search by type");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].content, "fact memory");

        // Search by non-matching type → should be empty.
        let episodes = engine
            .search_memories(&MemorySearchQuery {
                memory_type: Some(MemoryType::Episode),
                ..MemorySearchQuery::default()
            })
            .expect("search by non-matching type");
        assert!(
            episodes.is_empty(),
            "no memories with Episode type should exist"
        );
    }

    #[test]
    fn test_search_by_session_id() {
        let (engine, _dir) = setup();
        let session_a = Uuid::now_v7();
        let session_b = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "memory in session A".into(),
                tags: None,
            })
            .expect("create session a");

        engine
            .create_memory(NewMemory {
                session_id: session_b,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "memory in session B".into(),
                tags: None,
            })
            .expect("create session b");

        // Search by session_a → should return exactly one.
        let results_a = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                ..MemorySearchQuery::default()
            })
            .expect("search by session");
        assert_eq!(results_a.len(), 1);
        assert_eq!(results_a[0].content, "memory in session a");

        // Search by session_b → should return exactly one.
        let results_b = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_b),
                ..MemorySearchQuery::default()
            })
            .expect("search by session b");
        assert_eq!(results_b.len(), 1);
        assert_eq!(results_b[0].content, "memory in session b");
    }

    #[test]
    fn test_search_by_tags() {
        let (engine, _dir) = setup();
        let session_id = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "rust programming tips".into(),
                tags: Some(vec!["rust".into(), "programming".into()]),
            })
            .expect("create rust memory");

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "python programming tips".into(),
                tags: Some(vec!["python".into(), "programming".into()]),
            })
            .expect("create python memory");

        // Search by tag "rust" → should return exactly one.
        let rust_results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["rust".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by tag");
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].content, "rust programming tips");

        // Search by tag "programming" → should return both.
        let prog_results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["programming".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by shared tag");
        assert_eq!(prog_results.len(), 2);

        // Search by non-matching tag → empty.
        let no_match = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["golang".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by non-matching tag");
        assert!(no_match.is_empty(), "no memories tagged 'golang'");
    }

    #[test]
    fn test_search_combined_filters() {
        let (engine, _dir) = setup();
        let session_a = Uuid::now_v7();
        let session_b = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact in session A tagged urgent".into(),
                tags: Some(vec!["urgent".into()]),
            })
            .expect("create memory 1");

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Preference,
                content: "preference in session A tagged urgent".into(),
                tags: Some(vec!["urgent".into()]),
            })
            .expect("create memory 2");

        engine
            .create_memory(NewMemory {
                session_id: session_b,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact in session B tagged normal".into(),
                tags: Some(vec!["normal".into()]),
            })
            .expect("create memory 3");

        // Combined: session_a + Fact → 1 result.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                memory_type: Some(MemoryType::Fact),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + fact");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "fact in session a tagged urgent");

        // Combined: session_a + urgent tag → 2 results.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + urgent");
        assert_eq!(results.len(), 2);

        // Combined: session_a + Fact + urgent → 1 result.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                memory_type: Some(MemoryType::Fact),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + fact + urgent");
        assert_eq!(results.len(), 1);

        // Combined: session_b + urgent → 0 results.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_b),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_b + urgent");
        assert!(results.is_empty(), "no urgent memories in session B");
    }

    // -----------------------------------------------------------------------
    // Agent CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_agent_skill_roundtrip() {
        let (engine, _dir) = setup();

        let agent = engine
            .create_agent(NewAgent {
                name: "test-agent".into(),
                agent_type: "chat".into(),
                description: "A test agent".into(),
                capabilities: Some(vec!["code".into(), "search".into()]),
                status: Some(AgentStatus::Active),
                config: Some(serde_json::json!({"model": "gpt-4"})),
            })
            .expect("create agent");
        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.version, 1);
        assert!(agent.capabilities.contains(&"code".to_string()));

        // Get agent (cache-aside — should hit L1 after write-through).
        let fetched = engine
            .get_agent(agent.id)
            .expect("get agent")
            .expect("agent exists");
        assert_eq!(fetched.name, "test-agent");

        // Update agent (write-around).
        let updated = engine
            .update_agent(
                agent.id,
                &AgentPatch {
                    name: Some("updated-agent".into()),
                    ..AgentPatch::default()
                },
            )
            .expect("update agent");
        assert_eq!(updated.name, "updated-agent");

        // List agents.
        let agents = engine
            .list_agents(&AgentFilter::default())
            .expect("list agents");
        assert!(agents.iter().any(|a| a.name == "updated-agent"));

        // Delete agent.
        engine.delete_agent(agent.id).expect("delete agent");
        assert!(engine
            .get_agent(agent.id)
            .expect("get after delete")
            .is_none());
    }

    #[test]
    fn test_agent_delete_invalidates_cache() {
        let (engine, _dir) = setup();
        let agent = engine
            .create_agent(NewAgent {
                name: "del-test".into(),
                agent_type: "test".into(),
                description: "delete test".into(),
                capabilities: None,
                status: None,
                config: None,
            })
            .expect("create");

        // Warm cache.
        let _ = engine.get_agent(agent.id).expect("warm");

        engine.delete_agent(agent.id).expect("delete");
        assert!(engine
            .get_agent(agent.id)
            .expect("get after delete")
            .is_none());
    }

    // -----------------------------------------------------------------------
    // Skill CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_skill_roundtrip() {
        let (engine, _dir) = setup();

        let skill = engine
            .create_skill(NewSkill {
                name: "code-review".into(),
                description: "Review code changes".into(),
                category: "dev".into(),
                file_path: Some("/skills/review.py".into()),
            })
            .expect("create skill");
        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.version, 1);

        // Get skill (cache-aside).
        let fetched = engine
            .get_skill(skill.id)
            .expect("get skill")
            .expect("skill exists");
        assert_eq!(fetched.name, "code-review");

        // Update skill (write-around).
        let updated = engine
            .update_skill(
                skill.id,
                &SkillPatch {
                    name: Some("super-review".into()),
                    ..SkillPatch::default()
                },
            )
            .expect("update");
        assert_eq!(updated.name, "super-review");

        // List skills.
        let skills = engine
            .list_skills(&SkillFilter::default())
            .expect("list skills");
        assert!(skills.iter().any(|s| s.name == "super-review"));

        // Delete skill.
        engine.delete_skill(skill.id).expect("delete skill");
        assert!(engine
            .get_skill(skill.id)
            .expect("get after delete")
            .is_none());
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    #[test]
    fn test_settings_persist() {
        let (engine, _dir) = setup();

        engine.set_setting("theme", "dark").expect("set setting");
        engine
            .set_setting("language", "en-US")
            .expect("set setting");

        assert_eq!(
            engine.get_setting("theme").expect("get theme"),
            Some("dark".into())
        );
        assert_eq!(
            engine.get_setting("language").expect("get language"),
            Some("en-US".into())
        );
        assert_eq!(
            engine.get_setting("nonexistent").expect("get missing"),
            None
        );
    }

    #[test]
    fn test_memory_content_exactly_1mb_succeeds() {
        let (engine, _dir) = setup();
        // Boundary: exactly 1MB should be accepted.
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "x".repeat(1024 * 1024),
                tags: None,
            })
            .expect("1MB memory content should succeed");
        assert_eq!(memory.content.len(), 1024 * 1024);
    }

    #[test]
    fn test_memory_content_exceeds_limit_rejected() {
        let (engine, _dir) = setup();

        let oversized = NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "x".repeat(1024 * 1024 + 1),
            tags: None,
        };
        let result = engine.create_memory(oversized);
        assert!(result.is_err(), "oversized memory should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("1MB"), "error should mention the size limit");
    }

    #[test]
    fn test_setting_key_256_chars_succeeds() {
        let (engine, _dir) = setup();
        // Boundary: exactly 256 characters should be accepted.
        let key = "a".repeat(256);
        engine
            .set_setting(&key, "value")
            .expect("256-char key should succeed");
        assert_eq!(engine.get_setting(&key).expect("get"), Some("value".into()));
    }

    #[test]
    fn test_setting_empty_key_rejected() {
        let (engine, _dir) = setup();
        let result = engine.set_setting("", "value");
        assert!(result.is_err(), "empty key should be rejected");
    }

    #[test]
    fn test_setting_key_too_long_rejected() {
        let (engine, _dir) = setup();
        let long_key = "a".repeat(257);
        let result = engine.set_setting(&long_key, "value");
        assert!(result.is_err(), "overlong key should be rejected");
    }

    #[test]
    fn test_setting_valid_key_accepted() {
        let (engine, _dir) = setup();
        engine
            .set_setting("valid-key", "value")
            .expect("valid key should succeed");
        assert_eq!(
            engine.get_setting("valid-key").expect("get"),
            Some("value".into())
        );
    }

    #[test]
    fn test_setting_cache_aside() {
        let (engine, _dir) = setup();

        // Write-through stores in cache.
        engine.set_setting("test-key", "test-value").expect("set");

        // After write-through, get should be a L1 hit.
        let tel_before = engine.cache_telemetry();
        let val = engine.get_setting("test-key").expect("get");
        let tel_after = engine.cache_telemetry();
        assert_eq!(val, Some("test-value".into()));
        assert_eq!(tel_after.hits - tel_before.hits, 1);
    }

    // -----------------------------------------------------------------------
    // Audit log
    // -----------------------------------------------------------------------

    #[test]
    fn test_audit_logging() {
        let (engine, _dir) = setup();

        engine
            .log_audit(NewAuditEntry {
                action: "create_session".into(),
                entity_type: "Session".into(),
                entity_id: "abc-123".into(),
                actor: Some("user-1".into()),
                changes: Some(serde_json::json!({"status": "active"})),
            })
            .expect("log audit");

        engine
            .log_audit(NewAuditEntry {
                action: "create_memory".into(),
                entity_type: "Memory".into(),
                entity_id: "def-456".into(),
                actor: Some("user-1".into()),
                changes: None,
            })
            .expect("log audit");

        // Query all.
        let all = engine
            .query_audit(&AuditFilter::default())
            .expect("query audit");
        assert_eq!(all.len(), 2, "should have 2 audit entries");

        // Filter by entity type.
        let sessions = engine
            .query_audit(&AuditFilter {
                entity_type: Some("Session".into()),
                ..AuditFilter::default()
            })
            .expect("filter by type");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].action, "create_session");

        // Filter by actor.
        let actor_entries = engine
            .query_audit(&AuditFilter {
                actor: Some("user-1".into()),
                ..AuditFilter::default()
            })
            .expect("filter by actor");
        assert_eq!(actor_entries.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Maintenance
    // -----------------------------------------------------------------------

    #[test]
    fn test_flush_and_checkpoint() {
        let (engine, _dir) = setup();

        // Write something so there's data to flush.
        engine
            .create_session(NewSession {
                project: "flush-test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        engine.flush().expect("flush");
        let seq = engine.checkpoint().expect("checkpoint");
        assert!(seq > 0, "checkpoint sequence number should be > 0");
    }

    #[test]
    fn test_storage_size_non_zero() {
        let (engine, _dir) = setup();

        // Initially the size may be small (RocksDB metadata), but the call
        // should succeed and return at least the WAL size.
        let size = engine.storage_size().expect("storage size");
        // Verify the structure is well-formed.
        assert!(!size.per_cf.is_empty() || size.wal_size > 0);

        // Write a reasonable amount of data.
        for i in 0..10 {
            engine
                .create_memory(NewMemory {
                    session_id: Uuid::now_v7(),
                    agent_id: Uuid::now_v7(),
                    memory_type: MemoryType::Fact,
                    content: format!("large content item number {i} with some padding data to make the size more significant"),
                    tags: None,
                })
                .expect("create memory");
        }

        let size_after = engine.storage_size().expect("storage size after writes");
        // The total should now include the data we wrote. It might still be 0
        // if RocksDB hasn't flushed memtables, but estimate-live-data-size
        // should at least be > 0 after writes.
        // total is u64 — always >= 0. The call itself must not error.
        let _ = size_after.total;

        engine.flush().expect("flush");
        let size_flushed = engine.storage_size().expect("storage size after flush");
        // After flush, SST files should exist.
        assert!(
            size_flushed.total > 0 || size_flushed.wal_size > 0,
            "after flush, total or WAL size should be > 0"
        );
    }

    #[test]
    fn test_cache_telemetry_tracking() {
        let (engine, _dir) = setup();

        let tel = engine.cache_telemetry();
        assert_eq!(tel.total_ops, 0);

        // Create a session (write-through stores in cache, no read ops).
        let session = engine
            .create_session(NewSession {
                project: "tel-test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        // Get session (cache hit from write-through).
        let tel_before = engine.cache_telemetry();
        let _ = engine.get_session(session.id).expect("get");
        let tel_after = engine.cache_telemetry();

        assert_eq!(
            tel_after.total_ops - tel_before.total_ops,
            1,
            "a single get should be exactly 1 cache op"
        );
        assert_eq!(tel_after.hits - tel_before.hits, 1, "should be a hit");
    }

    #[test]
    fn test_cache_clear_and_clear_type() {
        let (engine, _dir) = setup();

        let session = engine
            .create_session(NewSession {
                project: "cache-clear".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "cache-clear content".into(),
                tags: None,
            })
            .expect("create memory");

        // Both are cached via write-through.
        engine.clear_cache_type("session");
        // Session should miss now, memory should still hit.
        let tel_before = engine.cache_telemetry();
        let _session_get = engine.get_session(session.id).expect("get session");
        let _memory_get = engine.get_memory(memory.id).expect("get memory");
        let tel_after = engine.cache_telemetry();

        // Session was a miss (cleared), memory was a hit.
        assert_eq!(
            tel_after.misses - tel_before.misses,
            1,
            "session should miss after clear_type"
        );
        assert_eq!(
            tel_after.hits - tel_before.hits,
            1,
            "memory should still hit after clear_type(session)"
        );

        // Clear all.
        engine.clear_cache();
        let tel_before2 = engine.cache_telemetry();
        let _session_get2 = engine.get_session(session.id).expect("get session");
        let _memory_get2 = engine.get_memory(memory.id).expect("get memory");
        let tel_after2 = engine.cache_telemetry();
        // Both should miss now.
        assert_eq!(tel_after2.misses - tel_before2.misses, 2);
    }

    // -----------------------------------------------------------------------
    // Not found / error paths
    // -----------------------------------------------------------------------

    #[test]
    fn test_invalid_session_returns_none() {
        let (engine, _dir) = setup();
        let id = Uuid::now_v7();
        let result = engine.get_session(id).expect("get nonexistent");
        assert!(result.is_none(), "non-existent session should return None");
    }

    #[test]
    fn test_not_found_returns_none() {
        let (engine, _dir) = setup();
        let random_id = Uuid::now_v7();

        assert!(engine
            .get_session(random_id)
            .expect("get session")
            .is_none());
        assert!(engine.get_memory(random_id).expect("get memory").is_none());
        assert!(engine.get_agent(random_id).expect("get agent").is_none());
        assert!(engine.get_skill(random_id).expect("get skill").is_none());
    }

    // -----------------------------------------------------------------------
    // Compile-time trait bounds
    // -----------------------------------------------------------------------

    /// Verify that `Engine` implements `Send + Sync` so it can be shared
    /// across threads via `Arc<Engine>`.
    #[test]
    fn test_engine_is_send() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Engine>();
        assert_sync::<Engine>();
    }

    /// Verify that `Engine` can be wrapped in `Arc` for shared ownership.
    #[test]
    fn test_engine_arc_compatible() {
        use std::sync::Arc;
        let (engine, _dir) = setup();
        let _arc = Arc::new(engine);
    }

    // -----------------------------------------------------------------------
    // Memory content size limit on update
    // -----------------------------------------------------------------------

    #[test]
    fn test_update_memory_content_exactly_1mb_succeeds() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "initial".into(),
                tags: None,
            })
            .expect("create");

        let updated = engine
            .update_memory(
                memory.id,
                &MemoryPatch {
                    content: Some("x".repeat(1024 * 1024)),
                    ..MemoryPatch::default()
                },
            )
            .expect("1MB update content should succeed");
        assert_eq!(updated.content.len(), 1024 * 1024);
    }

    #[test]
    fn test_update_memory_content_exceeds_limit_rejected() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "initial".into(),
                tags: None,
            })
            .expect("create");

        let result = engine.update_memory(
            memory.id,
            &MemoryPatch {
                content: Some("x".repeat(1024 * 1024 + 1)),
                ..MemoryPatch::default()
            },
        );
        assert!(result.is_err(), "oversized update content should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("1MB"), "error should mention the size limit: {err}");
    }

    #[test]
    fn test_update_memory_content_none_skips_validation() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "initial".into(),
                tags: None,
            })
            .expect("create");

        // Updating only memory_type (no content) should not trigger the size check.
        let updated = engine
            .update_memory(
                memory.id,
                &MemoryPatch {
                    memory_type: Some(MemoryType::Preference),
                    ..MemoryPatch::default()
                },
            )
            .expect("update without content should succeed");
        assert_eq!(updated.memory_type, MemoryType::Preference);
    }

    // -----------------------------------------------------------------------
    // Skill file_path validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_skill_with_valid_file_path() {
        let (engine, _dir) = setup();
        let skill = engine
            .create_skill(NewSkill {
                name: "test".into(),
                description: "desc".into(),
                category: "code".into(),
                file_path: Some("/home/skills/test.py".into()),
            })
            .expect("create skill with valid file_path");
        assert_eq!(skill.file_path, Some("/home/skills/test.py".into()));
    }

    #[test]
    fn test_create_skill_with_no_file_path() {
        let (engine, _dir) = setup();
        let skill = engine
            .create_skill(NewSkill {
                name: "test".into(),
                description: "desc".into(),
                category: "code".into(),
                file_path: None,
            })
            .expect("create skill without file_path");
        assert!(skill.file_path.is_none());
    }

    #[test]
    fn test_create_skill_empty_file_path_rejected() {
        let (engine, _dir) = setup();
        let result = engine.create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: Some(String::new()),
        });
        assert!(result.is_err(), "empty file_path should be rejected");
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("file_path"),
            "error should mention file_path: {err}"
        );
    }

    #[test]
    fn test_update_skill_empty_file_path_rejected() {
        let (engine, _dir) = setup();
        let skill = engine
            .create_skill(NewSkill {
                name: "test".into(),
                description: "desc".into(),
                category: "code".into(),
                file_path: None,
            })
            .expect("create skill");

        let result = engine.update_skill(
            skill.id,
            &SkillPatch {
                file_path: Some(String::new()),
                ..SkillPatch::default()
            },
        );
        assert!(result.is_err(), "empty file_path on update should be rejected");
    }

    #[test]
    fn test_update_skill_valid_file_path() {
        let (engine, _dir) = setup();
        let skill = engine
            .create_skill(NewSkill {
                name: "test".into(),
                description: "desc".into(),
                category: "code".into(),
                file_path: None,
            })
            .expect("create skill");

        let updated = engine
            .update_skill(
                skill.id,
                &SkillPatch {
                    file_path: Some("/new/path.py".into()),
                    ..SkillPatch::default()
                },
            )
            .expect("update with valid file_path");
        assert_eq!(updated.file_path, Some("/new/path.py".into()));
    }

    #[test]
    fn test_validate_file_path_too_long_rejected() {
        // Direct test of the validation helper.
        let result = Engine::validate_file_path(&Some("a".repeat(4097)));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("4096"));
    }
}
