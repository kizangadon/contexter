//! RocksDB-backed implementation of the [`StorageBackend`] trait.
//!
//! Uses 8 column families with per-CF compression and file-size tuning.
//! All keys are string-encoded with entity-type prefixes to allow
//! multiple logical entity kinds to share a single column family.

use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use chrono::Utc;
use rocksdb::{
    BlockBasedOptions, Cache, ColumnFamily, ColumnFamilyDescriptor, DBCompressionType,
    IteratorMode, WriteBatch, DB,
};
use uuid::Uuid;

use super::column_families::{
    CF_AGENTS, CF_AUDIT, CF_EFFICIENCY_MAP, CF_INDEX_STATE, CF_MEMORY_INDEX, CF_MEMORY_ITEMS,
    CF_SESSIONS, CF_SESSION_INDEX, CF_SETTINGS, CF_SKILLS, CF_TELEMETRY, CF_CONFLICTS,
    KEY_PREFIX_AGENT, KEY_PREFIX_AUDIT, KEY_PREFIX_SESSION, KEY_PREFIX_MEMORY,
    KEY_PREFIX_SETTING, KEY_PREFIX_SKILL, ColumnFamilyMap,
};
use super::types::RocksDbConfig;
use super::StorageBackend;
use crate::error::{EngineError, EngineResult};
use crate::models::*;

// ---------------------------------------------------------------------------
// RocksDbBackend
// ---------------------------------------------------------------------------

/// A RocksDB-backed implementation of [`StorageBackend`].
///
/// Uses 9 column families (8 primary + 1 secondary index) with per-CF
/// compression tuning. Key encoding uses entity-type prefixes for logical
/// key routing within shared CFs.
pub struct RocksDbBackend {
    db: DB,
    cfs: ColumnFamilyMap,
    config: RocksDbConfig,
    // Test-only seam (count-fallback-test): when set, `estimated_session_count`
    // reports the estimate as unavailable so `count_sessions` exercises the
    // exact full-scan fallback. Absent from production builds.
    #[cfg(test)]
    force_session_count_fallback: bool,
}

impl RocksDbBackend {
    /// Open a RocksDB database at the given path with all 8 column families.
    ///
    /// Creates the database directory if it does not exist.
    pub fn open(path: impl AsRef<Path>) -> EngineResult<Self> {
        let config = RocksDbConfig {
            path: path.as_ref().to_string_lossy().into_owned(),
            create_if_missing: true,
            wal_sync: true,
        };
        Self::open_with_config(config)
    }

    /// Open a RocksDB database with the given configuration.
    pub fn open_with_config(config: RocksDbConfig) -> EngineResult<Self> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(config.create_if_missing);
        opts.create_missing_column_families(true);

        // Build column family descriptors with per-CF compression tuning.
        // Each entry: (name, compression_type, target_file_size, use_block_cache, zstd_level).
        // zstd_level: `None` uses RocksDB's default (level 3 for Zstd).
        let cf_configs: [(&str, DBCompressionType, u64, bool, Option<i32>); 12] = [
            (
                CF_MEMORY_ITEMS,
                DBCompressionType::Zstd,
                64 * 1024 * 1024,
                true,
                None,
            ),
            (
                CF_SESSIONS,
                DBCompressionType::Zstd,
                32 * 1024 * 1024,
                false,
                None,
            ),
            (
                CF_AGENTS,
                DBCompressionType::Lz4,
                16 * 1024 * 1024,
                false,
                None,
            ),
            (
                CF_SKILLS,
                DBCompressionType::Lz4,
                16 * 1024 * 1024,
                false,
                None,
            ),
            (
                CF_EFFICIENCY_MAP,
                DBCompressionType::Lz4,
                8 * 1024 * 1024,
                false,
                None,
            ),
            (
                CF_TELEMETRY,
                DBCompressionType::Lz4,
                4 * 1024 * 1024,
                false,
                None,
            ),
            // REQ-S-007: conflicts CF uses Zstd level 1 (fastest compression).
            (
                CF_CONFLICTS,
                DBCompressionType::Zstd,
                8 * 1024 * 1024,
                false,
                Some(1),
            ),
            (
                CF_INDEX_STATE,
                DBCompressionType::Lz4,
                4 * 1024 * 1024,
                false,
                None,
            ),
            // memory_index: LZ4 compression, 16 MB target file size for secondary index entries.
            (
                CF_MEMORY_INDEX,
                DBCompressionType::Lz4,
                16 * 1024 * 1024,
                false,
                None,
            ),
            // CF_SETTINGS: LZ4 compression, 4 MB target file size.
            (
                CF_SETTINGS,
                DBCompressionType::Lz4,
                4 * 1024 * 1024,
                false,
                None,
            ),
            // CF_AUDIT: Zstd compression, 8 MB target file size.
            (
                CF_AUDIT,
                DBCompressionType::Zstd,
                8 * 1024 * 1024,
                false,
                None,
            ),
            // CF_SESSION_INDEX: LZ4 compression, 8 MB target file size.
            (
                CF_SESSION_INDEX,
                DBCompressionType::Lz4,
                8 * 1024 * 1024,
                false,
                None,
            ),
        ];

        let descriptors: Vec<ColumnFamilyDescriptor> = cf_configs
            .iter()
            .map(|(name, compression, target_size, use_cache, zstd_level)| {
                let mut cf_opts = rocksdb::Options::default();
                cf_opts.set_compression_type(*compression);
                cf_opts.set_target_file_size_base(*target_size);

                if let Some(level) = zstd_level {
                    // w_bits=-1 (default), level=requested, strategy=0, max_dict_bytes=0.
                    cf_opts.set_compression_options(-1, *level, 0, 0);
                }

                if *use_cache {
                    let mut block_opts = BlockBasedOptions::default();
                    let cache = Cache::new_lru_cache(256 * 1024 * 1024);
                    block_opts.set_block_cache(&cache);
                    cf_opts.set_block_based_table_factory(&block_opts);
                }

                ColumnFamilyDescriptor::new(*name, cf_opts)
            })
            .collect();

        // Ensure the data directory exists with restrictive permissions (0o700)
        // so that other users cannot read the RocksDB data files.
        let data_path = Path::new(&config.path);
        std::fs::create_dir_all(data_path)
            .map_err(|e| EngineError::Internal(format!("failed to create db dir: {e}")))?;
        std::fs::set_permissions(data_path, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| EngineError::Internal(format!("failed to set db dir perms: {e}")))?;

        let db =
            DB::open_cf_descriptors(&opts, &config.path, descriptors).map_err(EngineError::from)?;

        Ok(Self {
            db,
            cfs: ColumnFamilyMap::new(),
            config,
            #[cfg(test)]
            force_session_count_fallback: false,
        })
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    /// Resolve a column family name to a [`ColumnFamily`] reference.
    ///
    /// Returns `EngineError::Storage` if the named column family does not
    /// exist — propagating the error upward instead of panicking.
    fn cf(&self, name: &str) -> EngineResult<&ColumnFamily> {
        self.db
            .cf_handle(name)
            .ok_or_else(|| EngineError::Storage(format!("column family '{name}' not found")))
    }

    /// Best-effort O(1) estimate of session rows read from the
    /// `rocksdb.estimate-num-keys` column-family property.
    ///
    /// Returns `Ok(None)` when the property is unavailable or unparseable —
    /// callers must fall back to an exact full scan (see `count_sessions`).
    fn estimated_session_count(&self) -> EngineResult<Option<u64>> {
        // Test-only seam (count-fallback-test): make the property appear
        // unavailable so tests exercise the exact full-scan fallback.
        #[cfg(test)]
        if self.force_session_count_fallback {
            return Ok(None);
        }
        Ok(self
            .db
            .property_value_cf(self.cf(self.cfs.sessions)?, "rocksdb.estimate-num-keys")
            .ok()
            .flatten()
            .and_then(|v| v.parse::<u64>().ok()))
    }

    fn session_key(id: &Uuid) -> String {
        format!("{KEY_PREFIX_SESSION}{id}")
    }

    fn memory_key(id: &Uuid) -> String {
        format!("{KEY_PREFIX_MEMORY}{id}")
    }

    fn agent_key(id: &Uuid) -> String {
        format!("{KEY_PREFIX_AGENT}{id}")
    }

    fn skill_key(id: &Uuid) -> String {
        format!("{KEY_PREFIX_SKILL}{id}")
    }

    fn setting_key(key: &str) -> String {
        format!("{KEY_PREFIX_SETTING}{key}")
    }

    fn audit_key(id: &Uuid) -> String {
        format!("{KEY_PREFIX_AUDIT}{id}")
    }

    // -----------------------------------------------------------------------
    // Secondary index helpers
    // -----------------------------------------------------------------------

    /// Return the string representation of a [`MemoryType`] for index keys.
    fn memory_type_str(mt: &MemoryType) -> &'static str {
        match mt {
            MemoryType::Fact => "fact",
            MemoryType::Preference => "preference",
            MemoryType::Procedure => "procedure",
            MemoryType::Context => "context",
            MemoryType::Episode => "episode",
        }
    }

    /// Index key prefix for session-id lookups.
    fn session_index_prefix(session_id: &Uuid) -> String {
        format!("idx:ses:{session_id}:")
    }

    /// Index key for a single `session_id → memory_id` entry.
    fn session_index_key(memory_id: &Uuid, session_id: &Uuid) -> String {
        format!("idx:ses:{session_id}:{memory_id}")
    }

    /// Index key prefix for tag lookups.
    fn tag_index_prefix(tag: &str) -> String {
        format!("idx:tag:{}:", tag.to_lowercase())
    }

    /// Index key for a single `tag → memory_id` entry.
    fn tag_index_key(memory_id: &Uuid, tag: &str) -> String {
        format!("idx:tag:{}:{}", tag.to_lowercase(), memory_id)
    }

    /// Index key prefix for memory-type lookups.
    fn type_index_prefix(memory_type: &MemoryType) -> String {
        format!("idx:typ:{}:", Self::memory_type_str(memory_type))
    }

    /// Index key for a single `memory_type → memory_id` entry.
    fn type_index_key(memory_id: &Uuid, memory_type: &MemoryType) -> String {
        format!(
            "idx:typ:{}:{}",
            Self::memory_type_str(memory_type),
            memory_id
        )
    }

    /// Parse the trailing memory ID from an index key of the form
    /// `idx:<kind>:<value>:<memory_id>`.
    fn parse_memory_id_from_index_key(key: &[u8]) -> Option<Uuid> {
        let key_str = std::str::from_utf8(key).ok()?;
        // The last colon separates the index value from the UUID.
        let last_colon = key_str.rfind(':')?;
        let uuid_str = &key_str[last_colon + 1..];
        Uuid::parse_str(uuid_str).ok()
    }

    /// Write all secondary index entries for a [`Memory`] into `batch`.
    fn write_index_entries(
        &self,
        batch: &mut WriteBatch,
        memory: &Memory,
    ) -> EngineResult<()> {
        let cf = self.cf(self.cfs.memory_index)?;

        // session_id → memory_id
        let sk = Self::session_index_key(&memory.id, &memory.session_id);
        batch.put_cf(cf, sk.as_bytes(), b"");

        // memory_type → memory_id
        let tk = Self::type_index_key(&memory.id, &memory.memory_type);
        batch.put_cf(cf, tk.as_bytes(), b"");

        // tag → memory_id (one per tag)
        for tag in &memory.tags {
            let tag_k = Self::tag_index_key(&memory.id, tag);
            batch.put_cf(cf, tag_k.as_bytes(), b"");
        }

        Ok(())
    }

    /// Delete all secondary index entries for a [`Memory`] from `batch`.
    fn delete_index_entries(
        &self,
        batch: &mut WriteBatch,
        memory: &Memory,
    ) -> EngineResult<()> {
        let cf = self.cf(self.cfs.memory_index)?;

        // session_id → memory_id
        let sk = Self::session_index_key(&memory.id, &memory.session_id);
        batch.delete_cf(cf, sk.as_bytes());

        // memory_type → memory_id
        let tk = Self::type_index_key(&memory.id, &memory.memory_type);
        batch.delete_cf(cf, tk.as_bytes());

        // tag → memory_id (one per tag)
        for tag in &memory.tags {
            let tag_k = Self::tag_index_key(&memory.id, tag);
            batch.delete_cf(cf, tag_k.as_bytes());
        }

        Ok(())
    }

    /// Scan an index prefix, collecting all memory IDs from matching entries.
    fn scan_index_prefix(&self, prefix: &str) -> EngineResult<Vec<Uuid>> {
        let cf = self.cf(self.cfs.memory_index)?;
        let mut ids = Vec::new();

        let iter = self
            .db
            .iterator_cf(cf, IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));

        for item in iter {
            let (key, _value) = item?;
            if !key.starts_with(prefix.as_bytes()) {
                break;
            }
            if let Some(id) = Self::parse_memory_id_from_index_key(&key) {
                ids.push(id);
            }
        }

        Ok(ids)
    }

    /// Resolve matching memory IDs by intersecting secondary index lookups.
    fn resolve_memory_ids_via_index(&self, query: &MemorySearchQuery) -> EngineResult<Vec<Uuid>> {
        let mut sets: Vec<Vec<Uuid>> = Vec::new();

        if let Some(ref sid) = query.session_id {
            let prefix = Self::session_index_prefix(sid);
            sets.push(self.scan_index_prefix(&prefix)?);
        }

        if let Some(ref mt) = query.memory_type {
            let prefix = Self::type_index_prefix(mt);
            sets.push(self.scan_index_prefix(&prefix)?);
        }

        if let Some(ref tags) = query.tags {
            for tag in tags {
                let prefix = Self::tag_index_prefix(tag);
                sets.push(self.scan_index_prefix(&prefix)?);
            }
        }

        if sets.is_empty() {
            return Ok(Vec::new());
        }

        // Intersect all sets: start with the smallest.
        sets.sort_by_key(|s| s.len());
        let mut result = sets.remove(0);
        for other in &sets {
            result.retain(|id| other.contains(id));
        }

        Ok(result)
    }

    /// Conditionally flush the WAL after a mutating operation.
    ///
    /// When `config.wal_sync` is `true` (the default), this calls
    /// [`DB::flush_wal(true)`] to fsync the WAL, guaranteeing the write
    /// is durable on disk before returning.
    ///
    /// When `config.wal_sync` is `false`, this is a no-op — the write is
    /// buffered in the OS page cache and will be flushed on the next
    /// explicit [`StorageBackend::checkpoint()`] call or RocksDB internal
    /// WAL sync.
    fn maybe_flush_wal(&self) -> EngineResult<()> {
        if self.config.wal_sync {
            self.db.flush_wal(true)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Session secondary index helpers (REQ-CFA-003)
    // -----------------------------------------------------------------------

    /// Build the session index key for a session.
    ///
    /// Format: `idx:session:{project}:{agent_id}:{status}:{uuid}`
    fn session_index_entry(session: &Session) -> String {
        format!(
            "idx:session:{}:{}:{}:{}",
            session.project, session.agent_id, session.status, session.id
        )
    }

    /// Build a session index key from individual fields.
    #[allow(dead_code)]
    fn session_index_key_fields(
        id: &Uuid,
        project: &str,
        agent_id: &Uuid,
        status: &SessionStatus,
    ) -> String {
        format!(
            "idx:session:{}:{}:{}:{}",
            project, agent_id, status, id
        )
    }

    /// Build a prefix for scanning the session index from [`SessionFilter`].
    ///
    /// Returns `Some(prefix)` when at least the project filter is set – the
    /// only case where prefix-based lookup is unambiguous. For other filter
    /// combinations (agent-only, status-only) the caller falls back to a
    /// full scan with in-memory filtering.
    fn session_index_prefix_from_filter(filter: &SessionFilter) -> Option<String> {
        let project = filter.project.as_deref()?;
        let mut prefix = format!("idx:session:{project}:");
        if let Some(ref agent_id) = filter.agent_id {
            prefix.push_str(&agent_id.to_string());
            prefix.push(':');
            if let Some(ref status) = filter.status {
                prefix.push_str(&status.to_string());
                prefix.push(':');
            }
        }
        Some(prefix)
    }

    /// Resolve a [`Session`] from a session index key.
    ///
    /// The index key format is `idx:session:{project}:{agent_id}:{status}:{uuid}`.
    /// We extract the UUID (the part after the last `:`) and fetch the session.
    fn resolve_session_from_index_key(
        &self,
        key: &[u8],
        sessions_cf: &ColumnFamily,
    ) -> EngineResult<Option<Session>> {
        let key_str = std::str::from_utf8(key).map_err(|e| {
            EngineError::Internal(format!("invalid UTF-8 in index key: {e}"))
        })?;
        let last_colon = key_str.rfind(':').ok_or_else(|| {
            EngineError::Internal(format!("malformed index key: {key_str}"))
        })?;
        let uuid_str = &key_str[last_colon + 1..];
        let uuid = Uuid::parse_str(uuid_str).map_err(|e| {
            EngineError::Internal(format!("invalid UUID in index key: {e}"))
        })?;
        let session_key = Self::session_key(&uuid);
        match self.db.get_cf(sessions_cf, session_key.as_bytes())? {
            Some(bytes) => {
                let session = serde_json::from_slice(&bytes)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }
}

impl std::fmt::Debug for RocksDbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RocksDbBackend")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

// ---------------------------------------------------------------------------
// StorageBackend implementation
// ---------------------------------------------------------------------------

impl StorageBackend for RocksDbBackend {
    // =======================================================================
    // Session CRUD
    // =======================================================================

    fn create_session(&self, session: NewSession) -> EngineResult<Session> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let session = Session {
            id,
            project: session.project,
            agent_id: session.agent_id,
            status: session.status.unwrap_or(SessionStatus::Active),
            turn_count: 0,
            duration_ms: 0,
            metadata: session
                .metadata
                .unwrap_or(serde_json::Value::Object(Default::default())),
            efficiency_score: None,
            created_at: now,
            last_active: now,
        };

        let key = Self::session_key(&id);
        let value = serde_json::to_vec(&session)?;

        // Atomic batch: main entry + session index entry.
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf(self.cfs.sessions)?, key.as_bytes(), &value);
        let idx_key = Self::session_index_entry(&session);
        batch.put_cf(self.cf(self.cfs.session_index)?, idx_key.as_bytes(), b"");
        self.db.write(batch)?;

        self.maybe_flush_wal()?;

        Ok(session)
    }

    fn get_session(&self, id: Uuid) -> EngineResult<Option<Session>> {
        let key = Self::session_key(&id);
        match self.db.get_cf(self.cf(self.cfs.sessions)?, key.as_bytes())? {
            Some(bytes) => {
                let session = serde_json::from_slice(&bytes)?;
                Ok(Some(session))
            }
            None => Ok(None),
        }
    }

    fn list_sessions(&self, filter: &SessionFilter) -> EngineResult<Vec<Session>> {
        // Use session secondary index when project filter is available.
        // For agent-only or status-only filters, fall back to full scan.
        if filter.project.is_some() {
            let prefix = Self::session_index_prefix_from_filter(filter)
                .expect("project is Some so prefix is Some");
            let idx_cf = self.cf(self.cfs.session_index)?;
            let sessions_cf = self.cf(self.cfs.sessions)?;
            let mut results = Vec::new();

            let iter = self.db.iterator_cf(
                idx_cf,
                IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward),
            );

            for item in iter {
                let (key, _value) = item?;
                if !key.starts_with(prefix.as_bytes()) {
                    break;
                }
                if let Some(session) = self.resolve_session_from_index_key(&key, sessions_cf)? {
                    results.push(session);
                }
            }

            let offset = filter.offset as usize;
            let limit = filter.limit as usize;
            return Ok(results.into_iter().skip(offset).take(limit).collect());
        }

        // Unfiltered / agent-only / status-only: full scan with in-memory filter.
        let cf = self.cf(self.cfs.sessions)?;
        let mut results = Vec::new();
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_SESSION.as_bytes()) {
                continue;
            }
            let session: Session = serde_json::from_slice(&value)?;

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

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    fn update_session(&self, id: Uuid, patch: &SessionPatch) -> EngineResult<Session> {
        let key = Self::session_key(&id);
        let existing = self
            .db
            .get_cf(self.cf(self.cfs.sessions)?, key.as_bytes())?
            .ok_or_else(|| EngineError::NotFound {
                entity_type: "Session".into(),
                id: id.to_string(),
            })?;

        let old_session: Session = serde_json::from_slice(&existing)?;
        let mut session = old_session.clone();

        // Capture old index key BEFORE applying patches (status may change).
        let old_idx_key = Self::session_index_entry(&old_session);

        if let Some(ref status) = patch.status {
            session.status = status.clone();
        }
        if let Some(turn_count) = patch.turn_count {
            session.turn_count = turn_count;
        }
        if let Some(duration_ms) = patch.duration_ms {
            session.duration_ms = duration_ms;
        }
        if let Some(ref metadata) = patch.metadata {
            session.metadata = metadata.clone();
        }

        session.last_active = Utc::now();

        let value = serde_json::to_vec(&session)?;

        // Atomic batch: delete old index entry, write new data + new index entry.
        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf(self.cfs.session_index)?, old_idx_key.as_bytes());
        batch.put_cf(self.cf(self.cfs.sessions)?, key.as_bytes(), &value);
        let new_idx_key = Self::session_index_entry(&session);
        batch.put_cf(self.cf(self.cfs.session_index)?, new_idx_key.as_bytes(), b"");
        self.db.write(batch)?;

        self.maybe_flush_wal()?;

        Ok(session)
    }

    fn delete_session(&self, id: Uuid) -> EngineResult<()> {
        let key = Self::session_key(&id);

        // Read the session first to clean up its index entry.
        // If not found, delete is a no-op (idempotent).
        let session: Option<Session> = match self
            .db
            .get_cf(self.cf(self.cfs.sessions)?, key.as_bytes())?
        {
            Some(bytes) => Some(serde_json::from_slice(&bytes)?),
            None => None,
        };

        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf(self.cfs.sessions)?, key.as_bytes());
        if let Some(ref s) = session {
            let idx_key = Self::session_index_entry(s);
            batch.delete_cf(self.cf(self.cfs.session_index)?, idx_key.as_bytes());
        }
        self.db.write(batch)?;

        self.maybe_flush_wal()?;
        Ok(())
    }

    fn count_sessions(&self, filter: &SessionFilter) -> EngineResult<u64> {
        // Use session secondary index when project filter is available.
        if filter.project.is_some() {
            let prefix = Self::session_index_prefix_from_filter(filter)
                .expect("project is Some so prefix is Some");
            let cf = self.cf(self.cfs.session_index)?;
            let mut count = 0u64;

            let iter = self.db.iterator_cf(
                cf,
                IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward),
            );

            for item in iter {
                let (key, _value) = item?;
                if !key.starts_with(prefix.as_bytes()) {
                    break;
                }
                count += 1;
            }

            return Ok(count);
        }

        // When no filters are set, use the RocksDB estimate-num-keys property
        // for a fast O(1) count instead of a full scan (mirrors count_agents
        // and count_skills). The sessions CF holds only session keys — index
        // entries live in the companion session_index CF — so the estimate is
        // valid ONLY under this invariant; if it breaks, unfiltered counts
        // must not use the estimate.
        if filter.agent_id.is_none() && filter.status.is_none() {
            if let Some(count) = self.estimated_session_count()? {
                return Ok(count);
            }
            // Fall through to full scan if the property is unavailable.
        }

        // Unfiltered / agent-only / status-only: full scan with in-memory filter.
        let cf = self.cf(self.cfs.sessions)?;
        let mut count = 0u64;
        let iter = self.db.iterator_cf(cf, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_SESSION.as_bytes()) {
                continue;
            }

            let session: Session = serde_json::from_slice(&value)?;

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

            count += 1;
        }

        Ok(count)
    }

    // =======================================================================
    // Memory CRUD
    // =======================================================================

    fn create_memory(&self, memory: NewMemory) -> EngineResult<Memory> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        // Pre-lowercase content for performant keyword search (REQ-S-003).
        let content = memory.content.to_lowercase();
        let memory = Memory {
            id,
            session_id: memory.session_id,
            agent_id: memory.agent_id,
            memory_type: memory.memory_type,
            content,
            embedding: None,
            tags: memory.tags.unwrap_or_default(),
            version: 1,
            created_at: now,
            updated_at: now,
        };

        let key = Self::memory_key(&id);
        let value = serde_json::to_vec(&memory)?;

        // Use WriteBatch for atomic main-write + index-write.
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf(self.cfs.memory_items)?, key.as_bytes(), &value);
        self.write_index_entries(&mut batch, &memory)?;
        self.db.write(batch)?;

        self.maybe_flush_wal()?;

        Ok(memory)
    }

    fn get_memory(&self, id: Uuid) -> EngineResult<Option<Memory>> {
        let key = Self::memory_key(&id);
        match self
            .db
            .get_cf(self.cf(self.cfs.memory_items)?, key.as_bytes())?
        {
            Some(bytes) => {
                let memory = serde_json::from_slice(&bytes)?;
                Ok(Some(memory))
            }
            None => Ok(None),
        }
    }

    fn get_memories(&self, ids: &[Uuid]) -> EngineResult<Vec<Option<Memory>>> {
        let cf = self.cf(self.cfs.memory_items)?;
        let keys: Vec<String> = ids.iter().map(Self::memory_key).collect();
        let cf_and_keys: Vec<(&ColumnFamily, &[u8])> = keys
            .iter()
            .map(|k| (cf, k.as_bytes()))
            .collect();

        let results = self.db.multi_get_cf(cf_and_keys);

        let mut memories = Vec::with_capacity(ids.len());
        for result in results {
            match result {
                Ok(Some(bytes)) => {
                    let memory = serde_json::from_slice(&bytes)?;
                    memories.push(Some(memory));
                }
                Ok(None) => memories.push(None),
                Err(e) => return Err(EngineError::Storage(e.to_string())),
            }
        }
        Ok(memories)
    }

    fn search_memories(&self, query: &MemorySearchQuery) -> EngineResult<Vec<Memory>> {
        let has_indexed_filter = query.session_id.is_some()
            || query.memory_type.is_some()
            || query.tags.is_some();

        // Use secondary indexes to pre-filter memory IDs when any indexed
        // filter is set. Keyword-only queries still need a full scan.
        let filtered_ids: Option<Vec<Uuid>> = if has_indexed_filter {
            Some(self.resolve_memory_ids_via_index(query)?)
        } else {
            None
        };

        let mut results: Vec<(i32, Memory)> = Vec::new();

        // When only filters are present (no keywords), fetch memories directly
        // by ID — no full scan and no relevance scoring needed.
        if filtered_ids.is_some() && query.keywords.is_none() {
            if let Some(ref ids) = filtered_ids {
                for mem_id in ids {
                    if let Some(memory) = self.get_memory(*mem_id)? {
                        results.push((0, memory));
                    }
                }
            }
        } else {
            // Full scan (with optional index pre-filter or keyword scoring).
            let iter = self
                .db
                .iterator_cf(self.cf(self.cfs.memory_items)?, IteratorMode::Start);

            for item in iter {
                let (key, value) = item?;
                if !key.starts_with(KEY_PREFIX_MEMORY.as_bytes()) {
                    continue;
                }

                let memory: Memory = serde_json::from_slice(&value)?;

                // Index pre-filter: skip if not in the resolved ID set.
                if let Some(ref ids) = filtered_ids {
                    if !ids.contains(&memory.id) {
                        continue;
                    }
                }

                // Keyword relevance scoring (content is pre-lowercased).
                let mut score = 0i32;

                if let Some(ref keywords) = query.keywords {
                    let keywords_lower = keywords.to_lowercase();

                    // Multi-keyword scoring: each keyword contributes.
                    for kw in keywords_lower.split_whitespace() {
                        if kw.is_empty() {
                            continue;
                        }
                        if memory.content == kw {
                            score += 3;
                        } else if memory.content.starts_with(kw) {
                            score += 2;
                        } else if memory.content.contains(kw) {
                            score += 1;
                        }
                    }

                    if score == 0 {
                        continue; // No keyword match.
                    }
                }

                // Agent ID filter (not indexed — applied post-scan).
                if let Some(ref aid) = query.agent_id {
                    if memory.agent_id != *aid {
                        continue;
                    }
                }

                // NOTE: `project` filter skipped — Memory does not carry a
                // project field. Future phases may resolve project via Session
                // join.

                results.push((score, memory));
            }
        }

        // Sort by relevance descending, then by updated_at descending for ties.
        results.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| b.1.updated_at.cmp(&a.1.updated_at))
        });

        let offset = query.offset as usize;
        let limit = query.limit as usize;

        Ok(results
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(_, m)| m)
            .collect())
    }

    fn update_memory(&self, id: Uuid, patch: &MemoryPatch) -> EngineResult<Memory> {
        let key = Self::memory_key(&id);
        let existing = self
            .db
            .get_cf(self.cf(self.cfs.memory_items)?, key.as_bytes())?
            .ok_or_else(|| EngineError::NotFound {
                entity_type: "Memory".into(),
                id: id.to_string(),
            })?;

        let old_memory: Memory = serde_json::from_slice(&existing)?;

        let mut memory = old_memory.clone();

        if let Some(ref content) = patch.content {
            // Pre-lowercase content on write.
            memory.content = content.to_lowercase();
        }
        if let Some(ref memory_type) = patch.memory_type {
            memory.memory_type = memory_type.clone();
        }
        if let Some(ref tags) = patch.tags {
            memory.tags = tags.clone();
        }

        memory.version += 1;
        memory.updated_at = Utc::now();

        let value = serde_json::to_vec(&memory)?;

        // Atomic batch: delete old indexes, write main entry, write new indexes.
        let mut batch = WriteBatch::default();
        self.delete_index_entries(&mut batch, &old_memory)?;
        batch.put_cf(self.cf(self.cfs.memory_items)?, key.as_bytes(), &value);
        self.write_index_entries(&mut batch, &memory)?;
        self.db.write(batch)?;

        self.maybe_flush_wal()?;

        Ok(memory)
    }

    fn delete_memory(&self, id: Uuid) -> EngineResult<()> {
        let key = Self::memory_key(&id);

        // Read the memory first to know which index entries to clean.
        // If the memory doesn't exist, the delete is a no-op (idempotent).
        let memory: Option<Memory> = match self
            .db
            .get_cf(self.cf(self.cfs.memory_items)?, key.as_bytes())?
        {
            Some(bytes) => Some(serde_json::from_slice(&bytes)?),
            None => None,
        };

        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf(self.cfs.memory_items)?, key.as_bytes());

        if let Some(ref m) = memory {
            self.delete_index_entries(&mut batch, m)?;
        }

        self.db.write(batch)?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    fn count_memories(&self, filter: &MemoryFilter) -> EngineResult<u64> {
        // When no filters are set, use the RocksDB estimate-num-keys property
        // for a fast O(1) count instead of a full scan (REQ-S-004). The
        // memory_items CF holds only memory keys — index entries live in the
        // companion memory_index CF — so the estimate is valid ONLY under this
        // invariant; if it breaks, unfiltered counts must not use the estimate.
        if filter.session_id.is_none()
            && filter.agent_id.is_none()
            && filter.memory_type.is_none()
            && filter.tags.is_none()
        {
            if let Some(val) = self
                .db
                .property_value_cf(self.cf(self.cfs.memory_items)?, "rocksdb.estimate-num-keys")
                .ok()
                .flatten()
            {
                if let Ok(count) = val.parse::<u64>() {
                    return Ok(count);
                }
            }
            // Fall through to full scan if the property is unavailable.
        }

        // Use indexes for filtered columns when possible.
        let has_indexed_filter = filter.session_id.is_some()
            || filter.memory_type.is_some()
            || filter.tags.is_some();

        if has_indexed_filter && filter.agent_id.is_none() {
            // Convert to a search query and count via index intersection.
            let search_query = MemorySearchQuery {
                session_id: filter.session_id,
                memory_type: filter.memory_type.clone(),
                tags: filter.tags.clone(),
                ..MemorySearchQuery::default()
            };
            let ids = self.resolve_memory_ids_via_index(&search_query)?;
            return Ok(ids.len() as u64);
        }

        // Full scan for agent_id filter or mixed prefix-+unindexed filters.
        let mut count = 0u64;
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.memory_items)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_MEMORY.as_bytes()) {
                continue;
            }

            let memory: Memory = serde_json::from_slice(&value)?;

            if let Some(ref session_id) = filter.session_id {
                if memory.session_id != *session_id {
                    continue;
                }
            }
            if let Some(ref agent_id) = filter.agent_id {
                if memory.agent_id != *agent_id {
                    continue;
                }
            }
            if let Some(ref mem_type) = filter.memory_type {
                if memory.memory_type != *mem_type {
                    continue;
                }
            }
            if let Some(ref tags) = filter.tags {
                let any_match = tags
                    .iter()
                    .any(|t| memory.tags.iter().any(|mt| mt.eq_ignore_ascii_case(t)));
                if !any_match {
                    continue;
                }
            }

            count += 1;
        }

        Ok(count)
    }

    // =======================================================================
    // Agent CRUD
    // =======================================================================

    fn create_agent(&self, agent: NewAgent) -> EngineResult<Agent> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let agent = Agent {
            id,
            name: agent.name,
            agent_type: agent.agent_type,
            description: agent.description,
            capabilities: agent.capabilities.unwrap_or_default(),
            status: agent.status.unwrap_or(AgentStatus::Active),
            config: agent
                .config
                .unwrap_or(serde_json::Value::Object(Default::default())),
            version: 1,
            created_at: now,
            updated_at: now,
        };

        let key = Self::agent_key(&id);
        let value = serde_json::to_vec(&agent)?;
        self.db
            .put_cf(self.cf(self.cfs.agents)?, key.as_bytes(), value)?;
        self.maybe_flush_wal()?;

        Ok(agent)
    }

    fn get_agent(&self, id: Uuid) -> EngineResult<Option<Agent>> {
        let key = Self::agent_key(&id);
        match self.db.get_cf(self.cf(self.cfs.agents)?, key.as_bytes())? {
            Some(bytes) => {
                let agent = serde_json::from_slice(&bytes)?;
                Ok(Some(agent))
            }
            None => Ok(None),
        }
    }

    fn list_agents(&self, filter: &AgentFilter) -> EngineResult<Vec<Agent>> {
        let mut results = Vec::new();
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.agents)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_AGENT.as_bytes()) {
                continue;
            }

            let agent: Agent = serde_json::from_slice(&value)?;

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

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    fn count_agents(&self, filter: &AgentFilter) -> EngineResult<u64> {
        // When no filters are set, use the RocksDB estimate-num-keys property
        // for a fast O(1) count instead of a full scan (mirrors count_memories).
        // The agents CF holds only agent keys (no separate index CF), so the
        // estimate is valid ONLY under this invariant; if the CF ever holds
        // non-agent keys, unfiltered counts must not use the estimate.
        if filter.name.is_none() && filter.status.is_none() && filter.capability.is_none() {
            if let Some(val) = self
                .db
                .property_value_cf(self.cf(self.cfs.agents)?, "rocksdb.estimate-num-keys")
                .ok()
                .flatten()
            {
                if let Ok(count) = val.parse::<u64>() {
                    return Ok(count);
                }
            }
            // Fall through to full scan if the property is unavailable.
        }

        // Filtered counts have no secondary index — full scan with
        // in-memory filtering (same semantics as list_agents).
        let mut count = 0u64;
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.agents)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_AGENT.as_bytes()) {
                continue;
            }

            let agent: Agent = serde_json::from_slice(&value)?;

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

            count += 1;
        }

        Ok(count)
    }

    fn update_agent(&self, id: Uuid, patch: &AgentPatch) -> EngineResult<Agent> {
        let key = Self::agent_key(&id);
        let existing = self
            .db
            .get_cf(self.cf(self.cfs.agents)?, key.as_bytes())?
            .ok_or_else(|| EngineError::NotFound {
                entity_type: "Agent".into(),
                id: id.to_string(),
            })?;

        let mut agent: Agent = serde_json::from_slice(&existing)?;

        if let Some(ref name) = patch.name {
            agent.name = name.clone();
        }
        if let Some(ref agent_type) = patch.agent_type {
            agent.agent_type = agent_type.clone();
        }
        if let Some(ref description) = patch.description {
            agent.description = description.clone();
        }
        if let Some(ref capabilities) = patch.capabilities {
            agent.capabilities = capabilities.clone();
        }
        if let Some(ref status) = patch.status {
            agent.status = status.clone();
        }
        if let Some(ref config) = patch.config {
            agent.config = config.clone();
        }

        agent.version += 1;
        agent.updated_at = Utc::now();

        let value = serde_json::to_vec(&agent)?;
        self.db
            .put_cf(self.cf(self.cfs.agents)?, key.as_bytes(), value)?;
        self.maybe_flush_wal()?;

        Ok(agent)
    }

    fn delete_agent(&self, id: Uuid) -> EngineResult<()> {
        let key = Self::agent_key(&id);
        self.db
            .delete_cf(self.cf(self.cfs.agents)?, key.as_bytes())?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    // =======================================================================
    // Skill CRUD
    // =======================================================================

    fn create_skill(&self, skill: NewSkill) -> EngineResult<Skill> {
        let id = Uuid::now_v7();
        let now = Utc::now();
        let skill = Skill {
            id,
            name: skill.name,
            description: skill.description,
            category: skill.category,
            version: 1,
            file_path: skill.file_path,
            created_at: now,
            updated_at: now,
        };

        let key = Self::skill_key(&id);
        let value = serde_json::to_vec(&skill)?;
        self.db
            .put_cf(self.cf(self.cfs.skills)?, key.as_bytes(), value)?;
        self.maybe_flush_wal()?;

        Ok(skill)
    }

    fn get_skill(&self, id: Uuid) -> EngineResult<Option<Skill>> {
        let key = Self::skill_key(&id);
        match self.db.get_cf(self.cf(self.cfs.skills)?, key.as_bytes())? {
            Some(bytes) => {
                let skill = serde_json::from_slice(&bytes)?;
                Ok(Some(skill))
            }
            None => Ok(None),
        }
    }

    fn list_skills(&self, filter: &SkillFilter) -> EngineResult<Vec<Skill>> {
        let mut results = Vec::new();
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.skills)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_SKILL.as_bytes()) {
                continue;
            }

            let skill: Skill = serde_json::from_slice(&value)?;

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

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    fn count_skills(&self, filter: &SkillFilter) -> EngineResult<u64> {
        // When no filters are set, use the RocksDB estimate-num-keys property
        // for a fast O(1) count instead of a full scan (mirrors count_memories).
        // The skills CF holds only skill keys (no separate index CF), so the
        // estimate is valid ONLY under this invariant; if the CF ever holds
        // non-skill keys, unfiltered counts must not use the estimate.
        if filter.name.is_none() && filter.category.is_none() {
            if let Some(val) = self
                .db
                .property_value_cf(self.cf(self.cfs.skills)?, "rocksdb.estimate-num-keys")
                .ok()
                .flatten()
            {
                if let Ok(count) = val.parse::<u64>() {
                    return Ok(count);
                }
            }
            // Fall through to full scan if the property is unavailable.
        }

        // Filtered counts have no secondary index — full scan with
        // in-memory filtering (same semantics as list_skills).
        let mut count = 0u64;
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.skills)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_SKILL.as_bytes()) {
                continue;
            }

            let skill: Skill = serde_json::from_slice(&value)?;

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

            count += 1;
        }

        Ok(count)
    }

    fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> EngineResult<Skill> {
        let key = Self::skill_key(&id);
        let existing = self
            .db
            .get_cf(self.cf(self.cfs.skills)?, key.as_bytes())?
            .ok_or_else(|| EngineError::NotFound {
                entity_type: "Skill".into(),
                id: id.to_string(),
            })?;

        let mut skill: Skill = serde_json::from_slice(&existing)?;

        if let Some(ref name) = patch.name {
            skill.name = name.clone();
        }
        if let Some(ref description) = patch.description {
            skill.description = description.clone();
        }
        if let Some(ref category) = patch.category {
            skill.category = category.clone();
        }
        if let Some(ref file_path) = patch.file_path {
            skill.file_path = Some(file_path.clone());
        }

        skill.version += 1;
        skill.updated_at = Utc::now();

        let value = serde_json::to_vec(&skill)?;
        self.db
            .put_cf(self.cf(self.cfs.skills)?, key.as_bytes(), value)?;
        self.maybe_flush_wal()?;

        Ok(skill)
    }

    fn delete_skill(&self, id: Uuid) -> EngineResult<()> {
        let key = Self::skill_key(&id);
        self.db
            .delete_cf(self.cf(self.cfs.skills)?, key.as_bytes())?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    // =======================================================================
    // Settings
    // =======================================================================

    fn get_setting(&self, key: &str) -> EngineResult<Option<String>> {
        let db_key = Self::setting_key(key);
        match self
            .db
            .get_cf(self.cf(self.cfs.settings)?, db_key.as_bytes())?
        {
            Some(bytes) => {
                let value = String::from_utf8(bytes.to_vec()).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 in setting value: {e}"))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    fn set_setting(&self, key: &str, value: &str) -> EngineResult<()> {
        let db_key = Self::setting_key(key);
        self.db.put_cf(
            self.cf(self.cfs.settings)?,
            db_key.as_bytes(),
            value.as_bytes(),
        )?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    // =======================================================================
    // Audit log
    // =======================================================================

    fn append_audit_entry(&self, entry: &NewAuditEntry) -> EngineResult<()> {
        let id = Uuid::now_v7();
        let audit_entry = AuditEntry {
            id,
            action: entry.action.clone(),
            entity_type: entry.entity_type.clone(),
            entity_id: entry.entity_id.clone(),
            actor: entry.actor.clone(),
            summary: entry.summary.clone(),
            metadata: std::collections::HashMap::new(),
            created_at: Utc::now(),
        };

        let key = Self::audit_key(&id);
        let value = serde_json::to_vec(&audit_entry)?;
        self.db
            .put_cf(self.cf(self.cfs.audit)?, key.as_bytes(), value)?;
        self.maybe_flush_wal()?;

        Ok(())
    }

    fn query_audit_log(&self, filter: &AuditFilter) -> EngineResult<Vec<AuditEntry>> {
        let mut results = Vec::new();
        let iter = self
            .db
            .iterator_cf(self.cf(self.cfs.audit)?, IteratorMode::Start);

        for item in iter {
            let (key, value) = item?;
            if !key.starts_with(KEY_PREFIX_AUDIT.as_bytes()) {
                continue;
            }

            let entry: AuditEntry = serde_json::from_slice(&value)?;

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

        // Newest first.
        results.reverse();

        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    // =======================================================================
    // Generic key-value access
    // =======================================================================

    fn store_raw(&self, cf: &str, key: &str, value: &[u8]) -> EngineResult<()> {
        let cf_handle = self.cf(cf)?;
        let write_opts = rocksdb::WriteOptions::default();
        self.db.put_cf_opt(cf_handle, key.as_bytes(), value, &write_opts)?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    fn get_raw(&self, cf: &str, key: &str) -> EngineResult<Option<Vec<u8>>> {
        let cf_handle = self.cf(cf)?;
        match self.db.get_cf(cf_handle, key.as_bytes())? {
            Some(bytes) => Ok(Some(bytes.to_vec())),
            None => Ok(None),
        }
    }

    fn write_batch(&self, cf: &str, entries: Vec<(String, Vec<u8>)>) -> EngineResult<()> {
        let cf_handle = self.cf(cf)?;
        let mut batch = WriteBatch::default();
        for (key, value) in entries {
            batch.put_cf(cf_handle, key.as_bytes(), &value);
        }
        self.db.write(batch)?;
        self.maybe_flush_wal()?;
        Ok(())
    }

    fn scan_cf_keys(&self, cf: &str, prefix: &str) -> EngineResult<Vec<Vec<u8>>> {
        let cf_handle = self.cf(cf)?;
        let mut keys = Vec::new();
        let iter = self.db.iterator_cf(
            cf_handle,
            IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward),
        );
        for item in iter {
            let (key, _value) = item?;
            if !key.starts_with(prefix.as_bytes()) {
                break;
            }
            keys.push(key.to_vec());
        }
        Ok(keys)
    }

    // =======================================================================
    // Maintenance
    // =======================================================================

    fn flush(&self) -> EngineResult<()> {
        self.db.flush().map_err(EngineError::from)
    }

    fn checkpoint(&self) -> EngineResult<u64> {
        // Always flush WAL on checkpoint regardless of wal_sync setting.
        // Users who disable wal_sync for write throughput rely on explicit
        // checkpoint() calls to guarantee durability.
        self.db.flush_wal(true)?;
        let seq = self.db.latest_sequence_number();
        Ok(seq)
    }

    fn storage_size(&self) -> EngineResult<StorageSize> {
        let cf_pairs: [(&str, &'static str); 12] = [
            ("memory_items", self.cfs.memory_items),
            ("sessions", self.cfs.sessions),
            ("agents", self.cfs.agents),
            ("skills", self.cfs.skills),
            ("efficiency_map", self.cfs.efficiency_map),
            ("telemetry", self.cfs.telemetry),
            ("conflicts", self.cfs.conflicts),
            ("index_state", self.cfs.index_state),
            ("memory_index", self.cfs.memory_index),
            ("settings", self.cfs.settings),
            ("audit", self.cfs.audit),
            ("session_index", self.cfs.session_index),
        ];

        let mut per_cf = HashMap::new();
        let mut total = 0u64;

        // Batched into 2 property-value calls per CF (was 3 per CF previously),
        // covering both memtable-resident and SST-resident data.
        for (label, cf_name) in &cf_pairs {
            let cf = self.cf(cf_name)?;
            let live_size = self
                .db
                .property_value_cf(cf, "rocksdb.estimate-live-data-size")
                .ok()
                .flatten()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let mem_size = self
                .db
                .property_value_cf(cf, "rocksdb.cur-size-all-mem-tables")
                .ok()
                .flatten()
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(0);
            let size = live_size.max(mem_size);
            per_cf.insert(label.to_string(), size);
            total += size;
        }

        let wal_size = self
            .db
            .property_value("rocksdb.wal-size")
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        total += wal_size;

        Ok(StorageSize {
            per_cf,
            wal_size,
            total,
        })
    }

    // -----------------------------------------------------------------------
    // Raw storage (for testing and low-level access)
    // -----------------------------------------------------------------------

    fn store(&self, cf_name: &str, key: &str, value: &[u8]) -> EngineResult<()> {
        let cf = self.cf(cf_name)?;
        self.db.put_cf(cf, key.as_bytes(), value)?;
        Ok(())
    }

    fn get(&self, cf_name: &str, key: &str) -> EngineResult<Option<Vec<u8>>> {
        let cf = self.cf(cf_name)?;
        match self.db.get_cf(cf, key.as_bytes())? {
            Some(bytes) => Ok(Some(bytes.to_vec())),
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use tempfile::TempDir;

    fn setup_db() -> (RocksDbBackend, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let backend = RocksDbBackend::open(dir.path()).expect("open RocksDB");
        (backend, dir)
    }

    // -----------------------------------------------------------------------
    // Initialization
    // -----------------------------------------------------------------------

    #[test]
    fn test_engine_init_creates_cfs() {
        let (backend, _dir) = setup_db();

        // Verify all 12 CFs are accessible.
        assert!(backend.db.cf_handle(CF_MEMORY_ITEMS).is_some());
        assert!(backend.db.cf_handle(CF_SESSIONS).is_some());
        assert!(backend.db.cf_handle(CF_AGENTS).is_some());
        assert!(backend.db.cf_handle(CF_SKILLS).is_some());
        assert!(backend.db.cf_handle(CF_EFFICIENCY_MAP).is_some());
        assert!(backend.db.cf_handle(CF_TELEMETRY).is_some());
        assert!(backend.db.cf_handle(CF_CONFLICTS).is_some());
        assert!(backend.db.cf_handle(CF_INDEX_STATE).is_some());
        assert!(backend.db.cf_handle(CF_MEMORY_INDEX).is_some());
        assert!(backend.db.cf_handle(CF_SETTINGS).is_some());
        assert!(backend.db.cf_handle(CF_AUDIT).is_some());
        assert!(backend.db.cf_handle(CF_SESSION_INDEX).is_some());
    }

    #[test]
    fn test_empty_db_initialization() {
        let (backend, _dir) = setup_db();

        assert_eq!(
            backend
                .count_sessions(&SessionFilter::default())
                .expect("count sessions"),
            0
        );
        assert_eq!(
            backend
                .count_memories(&MemoryFilter {
                    session_id: None,
                    agent_id: None,
                    memory_type: None,
                    tags: None,
                })
                .expect("count memories"),
            0
        );
    }

    // -----------------------------------------------------------------------
    // Session CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_session_create_get_roundtrip() {
        let (backend, _dir) = setup_db();

        let new_session = NewSession {
            project: "test-project".into(),
            agent_id: Uuid::now_v7(),
            status: Some(SessionStatus::Active),
            metadata: Some(serde_json::json!({"env": "test"})),
        };

        let created = backend
            .create_session(new_session.clone())
            .expect("create session");
        assert_eq!(created.project, "test-project");
        assert_eq!(created.turn_count, 0);
        assert_eq!(created.status, SessionStatus::Active);

        let fetched = backend
            .get_session(created.id)
            .expect("get session")
            .expect("session exists");
        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.project, created.project);
        assert_eq!(fetched.agent_id, created.agent_id);
        assert_eq!(fetched.status, created.status);
        assert_eq!(fetched.metadata, created.metadata);
    }

    #[test]
    fn test_session_list_with_filter() {
        let (backend, _dir) = setup_db();

        let agent_a = Uuid::now_v7();
        let agent_b = Uuid::now_v7();

        backend
            .create_session(NewSession {
                project: "proj-a".into(),
                agent_id: agent_a,
                status: None,
                metadata: None,
            })
            .expect("create");
        backend
            .create_session(NewSession {
                project: "proj-a".into(),
                agent_id: agent_b,
                status: None,
                metadata: None,
            })
            .expect("create");
        backend
            .create_session(NewSession {
                project: "proj-b".into(),
                agent_id: agent_a,
                status: None,
                metadata: None,
            })
            .expect("create");

        // Filter by project.
        let results = backend
            .list_sessions(&SessionFilter {
                project: Some("proj-a".into()),
                ..SessionFilter::default()
            })
            .expect("list sessions");
        assert_eq!(results.len(), 2);

        // Filter by project + agent.
        let results = backend
            .list_sessions(&SessionFilter {
                project: Some("proj-a".into()),
                agent_id: Some(agent_a),
                ..SessionFilter::default()
            })
            .expect("list sessions");
        assert_eq!(results.len(), 1);

        // Pagination.
        let results = backend
            .list_sessions(&SessionFilter {
                project: Some("proj-a".into()),
                limit: 1,
                offset: 0,
                ..SessionFilter::default()
            })
            .expect("list sessions");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_session_update_persists() {
        let (backend, _dir) = setup_db();

        let created = backend
            .create_session(NewSession {
                project: "test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        let updated = backend
            .update_session(
                created.id,
                &SessionPatch {
                    turn_count: Some(5),
                    duration_ms: Some(1000),
                    ..SessionPatch::default()
                },
            )
            .expect("update");
        assert_eq!(updated.turn_count, 5);
        assert_eq!(updated.duration_ms, 1000);

        // Verify persistence.
        let fetched = backend
            .get_session(created.id)
            .expect("get")
            .expect("exists");
        assert_eq!(fetched.turn_count, 5);
        assert_eq!(fetched.duration_ms, 1000);
    }

    #[test]
    fn test_session_delete_idempotent() {
        let (backend, _dir) = setup_db();

        let created = backend
            .create_session(NewSession {
                project: "test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create");

        backend.delete_session(created.id).expect("delete");
        assert!(backend
            .get_session(created.id)
            .expect("get after delete")
            .is_none());
        // Second delete is idempotent.
        backend.delete_session(created.id).expect("delete again");
    }

    // -----------------------------------------------------------------------
    // count_sessions estimate-num-keys fallback (count-fallback-test)
    // -----------------------------------------------------------------------
    //
    // The unfiltered fast path reads `rocksdb.estimate-num-keys` for an O(1)
    // count. On small stores that estimate is exact, so a plain seeded-store
    // test cannot distinguish the fast path from the fallback. These tests use
    // the test-only `force_session_count_fallback` seam to make the property
    // read report unavailable, forcing `count_sessions` down the exact
    // full-scan fallback (property unavailable -> exact scan).

    #[test]
    fn test_count_sessions_fallback_exact_on_seeded_store() {
        let (mut backend, _dir) = setup_db();

        // Mixed store: sessions across multiple projects and agents.
        for project in ["alpha", "beta", "gamma"] {
            for _ in 0..2 {
                backend
                    .create_session(NewSession {
                        project: project.into(),
                        agent_id: Uuid::now_v7(),
                        status: Some(SessionStatus::Active),
                        metadata: None,
                    })
                    .expect("create session");
            }
        }

        // Force the estimate property to be unavailable (test-only seam).
        backend.force_session_count_fallback = true;

        // Prove the fast path is disabled: the estimate read must report
        // unavailable so count_sessions cannot take the O(1) branch.
        assert_eq!(
            backend.estimated_session_count().expect("estimate read"),
            None,
            "seam must make the estimate unavailable so the fallback runs"
        );

        // Unfiltered count must therefore come from the full scan: exact total.
        let count = backend
            .count_sessions(&SessionFilter::default())
            .expect("count sessions");
        assert_eq!(count, 6, "fallback full scan must return the exact total");
    }

    #[test]
    fn test_count_sessions_fallback_empty_store_returns_zero() {
        let (mut backend, _dir) = setup_db();
        backend.force_session_count_fallback = true;

        assert_eq!(
            backend.estimated_session_count().expect("estimate read"),
            None,
            "seam must make the estimate unavailable so the fallback runs"
        );

        let count = backend
            .count_sessions(&SessionFilter::default())
            .expect("count sessions");
        assert_eq!(count, 0, "fallback on an empty store must count zero");
    }

    // -----------------------------------------------------------------------
    // Memory CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_memory_create_get() {
        let (backend, _dir) = setup_db();

        let memory = backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "test memory content".into(),
                tags: Some(vec!["important".into(), "user".into()]),
            })
            .expect("create memory");

        assert_eq!(memory.version, 1);
        assert!(memory.tags.contains(&"important".to_string()));
        assert!(memory.tags.contains(&"user".to_string()));

        let fetched = backend
            .get_memory(memory.id)
            .expect("get memory")
            .expect("memory exists");
        assert_eq!(fetched.content, "test memory content");
        assert_eq!(fetched.memory_type, MemoryType::Fact);
    }

    #[test]
    fn test_memory_search_keyword() {
        let (backend, _dir) = setup_db();

        backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "the quick brown fox jumps over the lazy dog".into(),
                tags: None,
            })
            .expect("create");
        backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Preference,
                content: "i prefer cats over dogs".into(),
                tags: None,
            })
            .expect("create");
        backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Procedure,
                content: "how to train your pet".into(),
                tags: None,
            })
            .expect("create");

        // Search by keyword.
        let results = backend
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

        // Case-insensitive.
        let results = backend
            .search_memories(&MemorySearchQuery {
                keywords: Some("FOX".into()),
                ..MemorySearchQuery::default()
            })
            .expect("search");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_search_filters() {
        let (backend, _dir) = setup_db();

        let session_a = Uuid::now_v7();

        backend
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "fact content".into(),
                tags: Some(vec!["tag1".into(), "tag2".into()]),
            })
            .expect("create");
        backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Preference,
                content: "preference content".into(),
                tags: Some(vec!["tag2".into()]),
            })
            .expect("create");

        // Filter by type + tags.
        let results = backend
            .search_memories(&MemorySearchQuery {
                memory_type: Some(MemoryType::Fact),
                tags: Some(vec!["tag1".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "fact content");

        // Filter by session.
        let results = backend
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                ..MemorySearchQuery::default()
            })
            .expect("search");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_memory_version_bump() {
        let (backend, _dir) = setup_db();

        let created = backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "v1 content".into(),
                tags: None,
            })
            .expect("create");
        assert_eq!(created.version, 1);

        let updated = backend
            .update_memory(
                created.id,
                &MemoryPatch {
                    content: Some("v2 content".into()),
                    ..MemoryPatch::default()
                },
            )
            .expect("update");
        assert_eq!(updated.version, 2);

        let updated2 = backend
            .update_memory(
                created.id,
                &MemoryPatch {
                    content: Some("v3 content".into()),
                    ..MemoryPatch::default()
                },
            )
            .expect("update");
        assert_eq!(updated2.version, 3);
    }

    #[test]
    fn test_memory_delete() {
        let (backend, _dir) = setup_db();

        let created = backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "to delete".into(),
                tags: None,
            })
            .expect("create");

        backend.delete_memory(created.id).expect("delete");
        assert!(backend
            .get_memory(created.id)
            .expect("get after delete")
            .is_none());
    }

    #[test]
    fn test_memory_large_content() {
        let (backend, _dir) = setup_db();

        let content = "x".repeat(1024 * 1024); // 1 MB
        let created = backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: content.clone(),
                tags: None,
            })
            .expect("create large memory");

        let fetched = backend
            .get_memory(created.id)
            .expect("get large memory")
            .expect("exists");
        assert_eq!(fetched.content.len(), content.len());
        assert_eq!(fetched.content, content);
    }

    // -----------------------------------------------------------------------
    // Agent & Skill CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_agent_skill_roundtrip() {
        let (backend, _dir) = setup_db();

        let agent = backend
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

        let skill = backend
            .create_skill(NewSkill {
                name: "code-review".into(),
                description: "Review code changes".into(),
                category: "dev".into(),
                file_path: Some("/skills/review.py".into()),
            })
            .expect("create skill");

        assert_eq!(skill.name, "code-review");
        assert_eq!(skill.version, 1);

        // Verify list returns both.
        let agents = backend
            .list_agents(&AgentFilter::default())
            .expect("list agents");
        assert!(agents.iter().any(|a| a.name == "test-agent"));

        let skills = backend
            .list_skills(&SkillFilter::default())
            .expect("list skills");
        assert!(skills.iter().any(|s| s.name == "code-review"));
    }

    // -----------------------------------------------------------------------
    // CF isolation
    // -----------------------------------------------------------------------

    #[test]
    fn test_generic_store_cf_isolation() {
        let (backend, _dir) = setup_db();

        // Create a memory (goes to memory_items CF).
        let mem = backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "isolated content".into(),
                tags: None,
            })
            .expect("create memory");

        // List sessions — should not see the memory since they're in different CFs.
        let sessions = backend
            .list_sessions(&SessionFilter::default())
            .expect("list sessions");
        assert!(
            sessions.is_empty(),
            "sessions CF should not contain memory entries"
        );

        // Verify memory is still accessible.
        assert!(backend.get_memory(mem.id).expect("get memory").is_some());
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    #[test]
    fn test_settings_roundtrip() {
        let (backend, _dir) = setup_db();

        backend.set_setting("theme", "dark").expect("set setting");
        backend
            .set_setting("language", "en-US")
            .expect("set setting");

        assert_eq!(
            backend.get_setting("theme").expect("get setting"),
            Some("dark".into())
        );
        assert_eq!(
            backend.get_setting("language").expect("get setting"),
            Some("en-US".into())
        );
        assert!(backend
            .get_setting("nonexistent")
            .expect("get missing setting")
            .is_none());
    }

    // -----------------------------------------------------------------------
    // Audit
    // -----------------------------------------------------------------------

    #[test]
    fn test_audit_append_query() {
        let (backend, _dir) = setup_db();

        backend
            .append_audit_entry(&NewAuditEntry {
                action: "create".into(),
                entity_type: "Session".into(),
                entity_id: "abc-123".into(),
                actor: Some("user-1".into()),
                summary: Some(serde_json::json!({"status": "active"})),
            })
            .expect("append audit");

        backend
            .append_audit_entry(&NewAuditEntry {
                action: "update".into(),
                entity_type: "Memory".into(),
                entity_id: "def-456".into(),
                actor: Some("user-1".into()),
                summary: Some(serde_json::json!({"content": "updated"})),
            })
            .expect("append audit");

        // Query all.
        let all = backend
            .query_audit_log(&AuditFilter::default())
            .expect("query audit");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].action, "update"); // Newest first.
        assert_eq!(all[1].action, "create");

        // Filter by entity type.
        let filtered = backend
            .query_audit_log(&AuditFilter {
                entity_type: Some("Memory".into()),
                ..AuditFilter::default()
            })
            .expect("query audit");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].entity_id, "def-456");

        // Filter by actor.
        let actor_filtered = backend
            .query_audit_log(&AuditFilter {
                actor: Some("user-1".into()),
                ..AuditFilter::default()
            })
            .expect("query audit");
        assert_eq!(actor_filtered.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Maintenance
    // -----------------------------------------------------------------------

    #[test]
    fn test_storage_size_report() {
        let (backend, _dir) = setup_db();

        // Populate some data.
        backend
            .create_session(NewSession {
                project: "size-test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create session");
        backend
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "size test content".into(),
                tags: None,
            })
            .expect("create memory");

        // Flush to ensure data is on disk.
        backend.flush().expect("flush");

        let size = backend.storage_size().expect("storage size");
        assert!(
            size.total > 0,
            "total storage size should be non-zero after writing data"
        );
        // At least the per_cf map should have entries.
        assert_eq!(size.per_cf.len(), 12);
    }

    // -----------------------------------------------------------------------
    // Concurrency
    // -----------------------------------------------------------------------

    #[test]
    fn test_concurrent_reads() {
        let (backend, _dir) = setup_db();
        let backend = Arc::new(backend);

        let created = backend
            .create_session(NewSession {
                project: "concurrent".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create session");

        let session_id = created.id;

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let backend = Arc::clone(&backend);
                thread::spawn(move || {
                    let fetched = backend
                        .get_session(session_id)
                        .expect("get session")
                        .expect("session exists");
                    assert_eq!(fetched.project, "concurrent");
                })
            })
            .collect();

        for h in handles {
            h.join().expect("thread panicked");
        }
    }
}
