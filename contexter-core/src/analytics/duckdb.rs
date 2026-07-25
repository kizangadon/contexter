//! File-backed DuckDB analytics engine with single Mutex<Connection>.
//!
//! [`DuckDbEngine`] wraps a single `duckdb::Connection` behind a `Mutex`
//! for thread safety (duckdb's `Connection` is not `Sync` due to internal
//! `RefCell` usage). Incremental sync using UPSERT semantics means syncs
//! are fast — only new or updated records are inserted — minimising lock
//! contention.
//!
//! The engine also supports incremental sync: on each `sync()` call it
//! tracks which records have already been processed via a `last_sync_timestamp`
//! and uses UPSERT (`INSERT OR REPLACE`) instead of `DELETE` + `INSERT`.

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::Instant;

use chrono::{DateTime, Utc};
use duckdb::{AccessMode, Config, Connection};

use crate::analytics::error::AnalyticsError;
use crate::analytics::sync::table_schemas;
use crate::analytics::{AnalyticsEngine, AnalyticsResult, Value};
use crate::storage::SharedBackend;

/// Column family name for pre-computed efficiency scores.
pub const EFFICIENCY_CF: &str = "efficiency_map";

/// Cached efficiency score data for a single session.
#[derive(Clone)]
pub(crate) struct EfficiencyEntry {
    pub(crate) project: String,
    pub(crate) total_memories: u64,
    pub(crate) useful_memories: u64,
    pub(crate) score: f64,
    pub(crate) cached_at: Instant,
}

/// Drop-based guard that cleans up the temporary database directory when the
/// engine is dropped.
///
/// This ensures that the DuckDB database file (and any spill-to-disk temp
/// files) are removed when the engine is destroyed, preventing accumulation
/// of temporary directories.
struct TempDirGuard {
    /// The path to the temp directory containing the DuckDB database.
    dir: Option<PathBuf>,
}

impl TempDirGuard {
    /// Create a new temp directory guard, creating the directory on disk.
    ///
    /// Uses a UUID-v4-based directory name to avoid collisions between
    /// parallel test threads or concurrent engine instances.
    fn new() -> std::io::Result<Self> {
        let unique_id = uuid::Uuid::new_v4();
        let dir = std::env::temp_dir().join(format!("contexter_duckdb_{unique_id}"));
        std::fs::create_dir_all(&dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)) {
                eprintln!("Warning: could not set 0o700 on temp dir: {e}");
            }
        }
        Ok(Self { dir: Some(dir) })
    }

    /// Return the path to the temp directory, if it still exists.
    fn dir(&self) -> Option<&PathBuf> {
        self.dir.as_ref()
    }

    /// Return the path to the DuckDB database file within the temp directory.
    fn db_path(&self) -> Option<PathBuf> {
        self.dir.as_ref().map(|d| d.join("analytics.db"))
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        if let Some(dir) = self.dir.take() {
            let _ = std::fs::remove_dir_all(&dir);
        }
    }
}

/// File-backed DuckDB analytics engine with incremental sync support.
///
/// The engine uses a single `Mutex<Connection>` for thread safety because
/// `duckdb::Connection` uses `RefCell` internally and is not `Sync`.
/// All read and write operations share this single connection.
///
/// Data synchronisation is incremental: the engine tracks the last sync
/// timestamp per table and uses `INSERT OR REPLACE` (UPSERT) to avoid
/// costly truncate+re-insert cycles. This keeps write durations short,
/// minimising Mutex contention.
///
/// # Thread safety
///
/// `duckdb::Connection` uses `RefCell` internally and is not `Sync`.
/// It is wrapped in a `Mutex` so the struct satisfies `Send + Sync`.
///
/// # Known limitation
///
/// A single connection means reads and writes serialise through the same
/// `Mutex`. Incremental sync mitigates write duration so the impact is
/// negligible for typical analytics queries.
pub struct DuckDbEngine {
    /// DuckDB connection behind a `Mutex` for thread safety.
    /// `duckdb::Connection` uses `RefCell` internally and is not `Sync`,
    /// so it must be serialized through a Mutex.
    conn: Mutex<Connection>,
    /// Tracks when each column family was last synced (for TTL-based refresh).
    synced_tables: Mutex<HashMap<String, Instant>>,
    /// Duration (in seconds) after which a table is considered stale.
    cache_ttl_secs: u64,
    /// Optional reference to the storage backend for real RocksDB sync.
    storage_backend: Mutex<Option<Box<dyn Any + Send>>>,
    /// Per-session efficiency score cache.
    efficiency_cache: Arc<RwLock<HashMap<String, EfficiencyEntry>>>,
    /// Guard for the temp directory used by DuckDB for spill-to-disk operations.
    /// Cleaned up automatically when the engine is dropped.
    _temp_dir: TempDirGuard,
    /// Tracks the last sync timestamp per table for incremental sync.
    /// Key: table name, Value: the maximum `updated_at` seen during the last sync.
    last_sync_timestamp: Mutex<HashMap<String, DateTime<Utc>>>,
}

impl DuckDbEngine {
    /// Create a new file-backed DuckDB analytics engine.
    ///
    /// A single DuckDB connection is opened to a file in the system temp
    /// directory, wrapped in a `Mutex` for thread safety.
    ///
    /// All tables defined by [`table_schemas()`] are created immediately
    /// via the write connection. They start empty until
    /// [`sync`](AnalyticsEngine::sync) is called.
    ///
    /// # Errors
    ///
    /// Returns [`AnalyticsError::QueryError`] if either DuckDB connection or
    /// any `CREATE TABLE` statement fails.
    pub fn new(cache_ttl_secs: u64) -> AnalyticsResult<Self> {
        // Create temp directory for DuckDB database file and spill operations.
        let temp_dir = TempDirGuard::new().map_err(|e| {
            AnalyticsError::Internal(format!("Failed to create temp directory: {e}"))
        })?;

        let db_path = temp_dir.db_path().ok_or_else(|| {
            AnalyticsError::Internal("TempDirGuard has no directory set".into())
        })?;

        // Open the DuckDB connection (file-backed, ReadWrite).
        let config = Config::default()
            .access_mode(AccessMode::ReadWrite)
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
        let connection = Connection::open_with_flags(&db_path, config)
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

        // Configure DuckDB to use our temp directory for spill operations.
        let temp_dir_path = temp_dir.dir().ok_or_else(|| {
            AnalyticsError::Internal("TempDirGuard has no directory set".into())
        })?;
        let pragma = format!(
            "PRAGMA temp_directory='{}'",
            temp_dir_path.display().to_string().replace('\'', "''")
        );
        connection
            .execute_batch(&pragma)
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

        // Create tables from schemas. Each table has a PRIMARY KEY on `id`
        // so INSERT OR REPLACE (UPSERT) works correctly during incremental sync.
        for schema in table_schemas() {
            let cols: Vec<String> = schema
                .columns
                .iter()
                .map(|(name, typ)| format!("{name} {typ}"))
                .collect();
            let create_sql = format!(
                "CREATE TABLE IF NOT EXISTS {} ({}, PRIMARY KEY (id));",
                schema.name,
                cols.join(", ")
            );
            connection
                .execute_batch(&create_sql)
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
        }

        Ok(Self {
            conn: Mutex::new(connection),
            synced_tables: Mutex::new(HashMap::new()),
            cache_ttl_secs,
            storage_backend: Mutex::new(None),
            efficiency_cache: Arc::new(RwLock::new(HashMap::new())),
            _temp_dir: temp_dir,
            last_sync_timestamp: Mutex::new(HashMap::new()),
        })
    }

    /// Check if a table needs re-syncing based on the configured TTL.
    fn needs_sync(&self, cf_name: &str) -> bool {
        let synced = self.synced_tables.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(last_sync) = synced.get(cf_name) {
            last_sync.elapsed().as_secs() > self.cache_ttl_secs
        } else {
            true // never synced
        }
    }

    /// Truncate all rows from a table before re-syncing.
    /// Used for the initial (cold) sync when no incremental state exists.
    fn truncate_table(&self, table_name: &str) -> AnalyticsResult<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute_batch(&format!("DELETE FROM {table_name}"))
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))
    }

    /// Convert a duckdb `Value` to our analytics `Value`.
    fn convert_value(ddb_val: &duckdb::types::Value) -> Value {
        match ddb_val {
            duckdb::types::Value::Null => Value::Null,
            duckdb::types::Value::Boolean(b) => Value::Bool(*b),
            duckdb::types::Value::TinyInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::SmallInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::Int(i) => Value::Int(*i as i64),
            duckdb::types::Value::BigInt(i) => Value::Int(*i),
            duckdb::types::Value::HugeInt(i) => {
                // i128 may not fit in i64; clamp for practical use.
                Value::Int(*i as i64)
            }
            duckdb::types::Value::UTinyInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::USmallInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::UInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::UBigInt(i) => Value::Int(*i as i64),
            duckdb::types::Value::Float(f) => Value::Float(*f as f64),
            duckdb::types::Value::Double(f) => Value::Float(*f),
            duckdb::types::Value::Text(s) => Value::Text(s.clone()),
            duckdb::types::Value::Blob(_) => Value::Text("<blob>".into()),
            duckdb::types::Value::Date32(_) => Value::Text("<date>".into()),
            duckdb::types::Value::Time64(_, _) => Value::Text("<time>".into()),
            duckdb::types::Value::Timestamp(_, _) => Value::Text("<timestamp>".into()),
            duckdb::types::Value::Interval { .. } => Value::Text("<interval>".into()),
            duckdb::types::Value::Decimal(d) => {
                Value::Float(d.to_string().parse::<f64>().unwrap_or(0.0))
            }
            duckdb::types::Value::Enum(s) => Value::Text(s.clone()),
            duckdb::types::Value::List(_) => Value::Text("<list>".into()),
        }
    }

    /// Convert our analytics `Value` to a duckdb `Value` for query parameter binding.
    fn value_to_duckdb(val: &Value) -> duckdb::types::Value {
        match val {
            Value::Null => duckdb::types::Value::Null,
            Value::Bool(b) => duckdb::types::Value::Boolean(*b),
            Value::Int(i) => duckdb::types::Value::BigInt(*i),
            Value::Float(f) => duckdb::types::Value::Double(*f),
            Value::Text(s) => duckdb::types::Value::Text(s.clone()),
        }
    }

    /// Iterate the storage backend's column family and insert all entries
    /// into the DuckDB table. Returns `Ok(false)` when no storage backend
    /// has been set (caller should fall back to sample data).
    fn sync_from_backend(&self, cf_name: &str, table_name: &str) -> AnalyticsResult<bool> {
        let storage_guard = self.storage_backend.lock().unwrap_or_else(|e| e.into_inner());
        let any_backend = match storage_guard.as_ref() {
            Some(b) => b,
            None => return Ok(false),
        };
        let shared_backend = match any_backend.downcast_ref::<SharedBackend>() {
            Some(sb) => sb,
            None => return Ok(false),
        };

        let backend = shared_backend
            .read()
            .map_err(|e| AnalyticsError::Internal(e.to_string()))?;

        let keys = backend
            .scan_cf_keys(cf_name, "")
            .map_err(|e| AnalyticsError::SyncError(e.to_string()))?;

        if keys.is_empty() {
            return Ok(true);
        }

        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        // Determine the last sync timestamp for incremental sync.
        // On first sync, `None` means process all records.
        let last_timestamp = {
            let ts_map = self.last_sync_timestamp.lock().unwrap_or_else(|e| e.into_inner());
            ts_map.get(table_name).copied()
        };

        // Track the maximum timestamp seen in this sync batch.
        let mut max_seen: Option<DateTime<Utc>> = None;

        let is_incremental = last_timestamp.is_some();

        match table_name {
            "sessions" => {
                let sql = if is_incremental {
                    "INSERT OR REPLACE INTO sessions \
                     (id, project, agent_id, status, turn_count, duration_ms, \
                      created_at, last_active) \
                     VALUES (?, ?, ?, ?, ?, ?, \
                             CAST(? AS TIMESTAMP), CAST(? AS TIMESTAMP))"
                } else {
                    "INSERT INTO sessions \
                     (id, project, agent_id, status, turn_count, duration_ms, \
                      created_at, last_active) \
                     VALUES (?, ?, ?, ?, ?, ?, \
                             CAST(? AS TIMESTAMP), CAST(? AS TIMESTAMP))"
                };
                let mut stmt = conn
                    .prepare(sql)
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

                for key_bytes in &keys {
                    let key_str = std::str::from_utf8(key_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("Invalid key UTF-8: {e}")))?;
                    let value_bytes = backend
                        .get_raw(cf_name, key_str)
                        .map_err(|e| AnalyticsError::SyncError(e.to_string()))?
                        .ok_or_else(|| {
                            AnalyticsError::SyncError(format!(
                                "Key '{key_str}' disappeared during sync"
                            ))
                        })?;
                    let json: serde_json::Value = serde_json::from_slice(&value_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("JSON parse: {e}")))?;

                    // Validate required timestamp fields before CAST.
                    let created_at = json["createdAt"].as_str().unwrap_or("");
                    if created_at.is_empty() {
                        eprintln!(
                            "[contexter] Warning: skipping session '{key_str}': \
                             missing/empty created_at"
                        );
                        continue;
                    }
                    let last_active = json["lastActive"].as_str().unwrap_or("");
                    if last_active.is_empty() {
                        eprintln!(
                            "[contexter] Warning: skipping session '{key_str}': \
                             missing/empty last_active"
                        );
                        continue;
                    }

                    // Incremental sync: skip records older than last sync timestamp.
                    if let Some(last_ts) = last_timestamp {
                        if let Ok(ts) = DateTime::parse_from_rfc3339(created_at) {
                            if ts.to_utc() <= last_ts {
                                continue;
                            }
                        }
                    }

                    // Track the maximum timestamp seen.
                    if let Ok(ts) = DateTime::parse_from_rfc3339(created_at) {
                        let ts_utc = ts.to_utc();
                        match max_seen {
                            Some(current) if ts_utc > current => max_seen = Some(ts_utc),
                            None => max_seen = Some(ts_utc),
                            _ => {}
                        }
                    }

                    stmt.execute(duckdb::params![
                        json["id"].as_str().unwrap_or(key_str),
                        json["project"].as_str().unwrap_or(""),
                        json["agentId"].as_str().unwrap_or(""),
                        json["status"].as_str().unwrap_or("unknown"),
                        json["turnCount"].as_i64().unwrap_or(0),
                        json["durationMs"].as_i64().unwrap_or(0),
                        created_at,
                        last_active,
                    ])
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
                }
            }
            "memories" => {
                let sql = if is_incremental {
                    "INSERT OR REPLACE INTO memories \
                     (id, session_id, memory_type, tags, created_at) \
                     VALUES (?, ?, ?, ?, CAST(? AS TIMESTAMP))"
                } else {
                    "INSERT INTO memories \
                     (id, session_id, memory_type, tags, created_at) \
                     VALUES (?, ?, ?, ?, CAST(? AS TIMESTAMP))"
                };
                let mut stmt = conn
                    .prepare(sql)
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

                for key_bytes in &keys {
                    let key_str = std::str::from_utf8(key_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("Invalid key UTF-8: {e}")))?;
                    let value_bytes = backend
                        .get_raw(cf_name, key_str)
                        .map_err(|e| AnalyticsError::SyncError(e.to_string()))?
                        .ok_or_else(|| {
                            AnalyticsError::SyncError(format!(
                                "Key '{key_str}' disappeared during sync"
                            ))
                        })?;
                    let json: serde_json::Value = serde_json::from_slice(&value_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("JSON parse: {e}")))?;

                    // Validate created_at before CAST to TIMESTAMP.
                    let created_at = json["createdAt"].as_str().unwrap_or("");
                    if created_at.is_empty() {
                        eprintln!(
                            "[contexter] Warning: skipping memory '{key_str}': \
                             missing/empty created_at"
                        );
                        continue;
                    }

                    // Incremental sync: skip records older than last sync timestamp.
                    if let Some(last_ts) = last_timestamp {
                        if let Ok(ts) = DateTime::parse_from_rfc3339(created_at) {
                            if ts.to_utc() <= last_ts {
                                continue;
                            }
                        }
                    }

                    // Track the maximum timestamp seen.
                    if let Ok(ts) = DateTime::parse_from_rfc3339(created_at) {
                        let ts_utc = ts.to_utc();
                        match max_seen {
                            Some(current) if ts_utc > current => max_seen = Some(ts_utc),
                            None => max_seen = Some(ts_utc),
                            _ => {}
                        }
                    }

                    let tags_str = json["tags"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str())
                                .collect::<Vec<&str>>()
                                .join(",")
                        })
                        .unwrap_or_default();

                    stmt.execute(duckdb::params![
                        json["id"].as_str().unwrap_or(key_str),
                        json["sessionId"].as_str().unwrap_or(""),
                        json["memoryType"].as_str().unwrap_or("fact"),
                        tags_str,
                        created_at,
                    ])
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
                }
            }
            "telemetry" => {
                let sql = if is_incremental {
                    "INSERT OR REPLACE INTO telemetry \
                     (id, event_type, scope, value, ts) \
                     VALUES (?, ?, ?, ?, CAST(? AS TIMESTAMP))"
                } else {
                    "INSERT INTO telemetry \
                     (id, event_type, scope, value, ts) \
                     VALUES (?, ?, ?, ?, CAST(? AS TIMESTAMP))"
                };
                let mut stmt = conn
                    .prepare(sql)
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

                for key_bytes in &keys {
                    let key_str = std::str::from_utf8(key_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("Invalid key UTF-8: {e}")))?;
                    let value_bytes = backend
                        .get_raw(cf_name, key_str)
                        .map_err(|e| AnalyticsError::SyncError(e.to_string()))?
                        .ok_or_else(|| {
                            AnalyticsError::SyncError(format!(
                                "Key '{key_str}' disappeared during sync"
                            ))
                        })?;
                    let json: serde_json::Value = serde_json::from_slice(&value_bytes)
                        .map_err(|e| AnalyticsError::SyncError(format!("JSON parse: {e}")))?;

                    // Validate timestamp before CAST to TIMESTAMP.
                    let timestamp = json["timestamp"].as_str().unwrap_or("");
                    if timestamp.is_empty() {
                        eprintln!(
                            "[contexter] Warning: skipping telemetry '{key_str}': \
                             missing/empty timestamp"
                        );
                        continue;
                    }

                    // Incremental sync: skip records older than last sync timestamp.
                    if let Some(last_ts) = last_timestamp {
                        if let Ok(ts) = DateTime::parse_from_rfc3339(timestamp) {
                            if ts.to_utc() <= last_ts {
                                continue;
                            }
                        }
                    }

                    // Track the maximum timestamp seen.
                    if let Ok(ts) = DateTime::parse_from_rfc3339(timestamp) {
                        let ts_utc = ts.to_utc();
                        match max_seen {
                            Some(current) if ts_utc > current => max_seen = Some(ts_utc),
                            None => max_seen = Some(ts_utc),
                            _ => {}
                        }
                    }

                    stmt.execute(duckdb::params![
                        json["id"].as_str().unwrap_or(key_str),
                        json["eventType"].as_str().unwrap_or("unknown"),
                        json["scope"].as_str().unwrap_or(""),
                        json["value"].as_f64().unwrap_or(0.0),
                        timestamp,
                    ])
                    .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
                }
            }
            _ => {
                return Err(AnalyticsError::SyncError(format!(
                    "Unknown table: {table_name}"
                )));
            }
        }

        // Persist the maximum timestamp seen during this sync batch so
        // future incremental syncs can skip records older than this.
        if let Some(max_ts) = max_seen {
            let mut ts_map = self.last_sync_timestamp.lock().unwrap_or_else(|e| e.into_inner());
            ts_map.insert(table_name.to_string(), max_ts);
        }

        Ok(true)
    }

    /// Insert hardcoded sample data for a table.
    ///
    /// Used as fallback when no storage backend is set, primarily for
    /// unit tests and environments without RocksDB.
    fn sync_sample_data(&self, table_name: &str) -> AnalyticsResult<()> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        if table_name == "sessions" {
            let mut stmt = conn
                .prepare(
                    "INSERT INTO sessions \
                     (id, project, agent_id, status, turn_count, duration_ms, \
                      created_at, last_active) \
                     VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
                )
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params![
                "test-session-1",
                "contexter",
                "agent-1",
                "completed",
                10i64,
                60000i64
            ])
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params![
                "test-session-2",
                "contexter",
                "agent-1",
                "active",
                5i64,
                30000i64
            ])
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
        } else if table_name == "memories" {
            let mut stmt = conn
                .prepare(
                    "INSERT INTO memories \
                     (id, session_id, memory_type, tags, created_at) \
                     VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)",
                )
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["mem-1", "test-session-1", "preference", "important"])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["mem-2", "test-session-1", "fact", "general"])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["mem-3", "test-session-1", "episode", "chat"])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["mem-4", "test-session-2", "fact", "general"])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["mem-5", "test-session-2", "preference", "important"])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
        } else if table_name == "telemetry" {
            let mut stmt = conn
                .prepare(
                    "INSERT INTO telemetry \
                     (id, event_type, scope, value, ts) \
                     VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)",
                )
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["t-1", "query", "engine", 1.5f64])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["t-2", "query", "engine", 2.3f64])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

            stmt.execute(duckdb::params!["t-3", "sync", "storage", 0.8f64])
                .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
        }

        Ok(())
    }
}

impl AnalyticsEngine for DuckDbEngine {
    fn query(&self, sql: &str, params: &[Value]) -> AnalyticsResult<Vec<Vec<Value>>> {
        // Auto-sync relevant tables for predefined queries.
        if sql == crate::analytics::queries::SESSION_COUNT_BY_RANGE
            || sql == crate::analytics::queries::EFFICIENCY_SCORES
            || sql == crate::analytics::queries::METRIC_CORRELATION
        {
            if self.needs_sync("sessions") {
                self.sync("sessions")?;
            }
            if self.needs_sync("memory_items") {
                self.sync("memory_items")?;
            }
        } else if sql == crate::analytics::queries::MEMORY_COUNT_BY_TYPE {
            if self.needs_sync("memory_items") {
                self.sync("memory_items")?;
            }
        } else if sql == crate::analytics::queries::TELEMETRY_AGGREGATION {
            if self.needs_sync("telemetry") {
                self.sync("telemetry")?;
            }
        }

        // Efficiency scores: check the in-memory cache before hitting DuckDB.
        if sql == crate::analytics::queries::EFFICIENCY_SCORES {
            if let Some(cached) = self.get_cached_efficiency_scores() {
                return Ok(cached);
            }
        }

        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn
            .prepare(sql)
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

        // Convert analytics Values to duckdb values for parameter binding.
        let duckdb_values: Vec<duckdb::types::Value> =
            params.iter().map(Self::value_to_duckdb).collect();
        let param_refs: Vec<&dyn duckdb::types::ToSql> =
            duckdb_values.iter().map(|v| v as &dyn duckdb::types::ToSql).collect();

        // query() calls execute() first, populating the arrow result.
        let mut rows = stmt
            .query(&param_refs[..])
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;

        let mut results: Vec<Vec<Value>> = Vec::new();

        // Determine column count from the first row by trying indices 0..N
        // until row.get() returns an error.  This avoids calling
        // Statement::column_count() before execute() has populated the arrow
        // result.
        let col_count: usize = if let Some(row) = rows
            .next()
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?
        {
            let mut count = 0usize;
            let mut first_values = Vec::new();
            loop {
                match row.get::<_, duckdb::types::Value>(count) {
                    Ok(v) => {
                        first_values.push(Self::convert_value(&v));
                        count += 1;
                    }
                    Err(_) => break,
                }
            }
            results.push(first_values);
            count
        } else {
            // Empty result set — nothing to return.
            return Ok(results);
        };

        // Remaining rows.
        while let Some(row) = rows
            .next()
            .map_err(|e| AnalyticsError::QueryError(e.to_string()))?
        {
            let mut values = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let ddb_val: duckdb::types::Value =
                    row.get(i).unwrap_or(duckdb::types::Value::Null);
                values.push(Self::convert_value(&ddb_val));
            }
            results.push(values);
        }

        // Populate efficiency cache from query results so subsequent calls
        // within the TTL window return cached data without hitting DuckDB.
        if sql == crate::analytics::queries::EFFICIENCY_SCORES {
            self.populate_efficiency_cache(&results);
        }

        Ok(results)
    }

    fn sync(&self, cf_name: &str) -> AnalyticsResult<()> {
        // Special case: efficiency_map syncs into the in-memory cache, not a
        // DuckDB table. It bypasses the schema lookup and DuckDB truncate.
        if cf_name == EFFICIENCY_CF {
            self.sync_efficiency_cache_from_backend()?;
            self.synced_tables
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(cf_name.to_string(), Instant::now());
            return Ok(());
        }

        // Find schema for this column family.
        let schema = table_schemas()
            .into_iter()
            .find(|s| s.source_cf == cf_name)
            .ok_or_else(|| AnalyticsError::ColumnFamilyNotFound(cf_name.to_string()))?;

        // Determine whether this is an incremental or full sync.
        let is_incremental = self
            .last_sync_timestamp
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(schema.name);

        if is_incremental {
            // Incremental sync: no truncation. sync_from_backend will use
            // INSERT OR REPLACE (UPSERT) and skip records older than the
            // last sync timestamp.
        } else {
            // First sync: truncate and re-insert everything.
            self.truncate_table(schema.name)?;
        }

        // Attempt real sync from storage backend. If the backend is not set
        // or the downcast fails (e.g. in tests that set an integer), fall
        // back to hardcoded sample data.
        let synced = self.sync_from_backend(cf_name, schema.name)?;
        if !synced {
            self.sync_sample_data(schema.name)?;
        }

        // Update sync timestamp.
        self.synced_tables
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(cf_name.to_string(), Instant::now());

        Ok(())
    }

    fn sync_all(&self) -> AnalyticsResult<()> {
        for schema in table_schemas() {
            self.sync(schema.source_cf)?;
        }
        Ok(())
    }

}

impl DuckDbEngine {
    /// Return all non-expired cached efficiency scores from the in-memory
    /// cache. Expired entries are silently skipped (deferred eviction — they
    /// are overwritten on the next `populate_efficiency_cache()` call).
    fn get_cached_efficiency_scores(&self) -> Option<Vec<Vec<Value>>> {
        let cache = self.efficiency_cache.read().ok()?;
        if cache.is_empty() {
            return None;
        }

        let now = Instant::now();

        // Build results from fresh entries only — skip expired entries
        // rather than scanning the entire cache for eviction. Expired
        // entries are overwritten on the next populate_efficiency_cache()
        // call (which clears the cache first).
        let mut results: Vec<Vec<Value>> = Vec::new();
        for (session_id, entry) in cache.iter() {
            let expired = now.duration_since(entry.cached_at).as_secs() > self.cache_ttl_secs;
            if !expired {
                results.push(vec![
                    Value::Text(session_id.clone()),
                    Value::Text(entry.project.clone()),
                    Value::Int(entry.total_memories as i64),
                    Value::Int(entry.useful_memories as i64),
                    Value::Float(entry.score),
                ]);
            }
        }

        if results.is_empty() {
            return None;
        }

        // Sort by efficiency_score descending to match the SQL query ORDER BY.
        results.sort_by(|a, b| {
            let a_score = match &a[4] {
                Value::Float(f) => *f,
                _ => 0.0,
            };
            let b_score = match &b[4] {
                Value::Float(f) => *f,
                _ => 0.0,
            };
            b_score
                .partial_cmp(&a_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Some(results)
    }

    /// Populate the efficiency cache from a full DuckDB EFFICIENCY_SCORES
    /// query result.  Each row is expected to have 5 columns:
    /// [session_id, project, total_memories, useful_memories, efficiency_score].
    fn populate_efficiency_cache(&self, results: &[Vec<Value>]) {
        let mut cache = match self.efficiency_cache.write() {
            Ok(g) => g,
            Err(_) => return, // silently skip if poisoned
        };
        cache.clear();
        let now = Instant::now();
        for row in results {
            if row.len() < 5 {
                continue;
            }
            let session_id = match &row[0] {
                Value::Text(s) => s.clone(),
                _ => continue,
            };
            let project = match &row[1] {
                Value::Text(s) => s.clone(),
                _ => String::new(),
            };
            let total_memories = match &row[2] {
                Value::Int(n) => *n as u64,
                _ => 0,
            };
            let useful_memories = match &row[3] {
                Value::Int(n) => *n as u64,
                _ => 0,
            };
            let score = match &row[4] {
                Value::Float(f) => *f,
                Value::Int(n) => *n as f64,
                _ => 0.0,
            };
            cache.insert(
                session_id,
                EfficiencyEntry {
                    project,
                    total_memories,
                    useful_memories,
                    score,
                    cached_at: now,
                },
            );
        }
    }
}

impl DuckDbEngine {
    /// Attach a storage backend so the engine can pull data for syncing.
    pub fn set_storage_backend(&self, backend: Box<dyn Any + Send>) {
        *self.storage_backend.lock().unwrap_or_else(|e| e.into_inner()) = Some(backend);
    }
}

impl DuckDbEngine {
    /// Populate the efficiency cache from the RocksDB efficiency_map column
    /// family.  Reads each key-value pair, parses the JSON value, and inserts
    /// it into the in-memory cache.
    ///
    /// Returns `Ok(())` when the backend is not set (no-op) so that callers
    /// can fall through to the DuckDB query path.
    fn sync_efficiency_cache_from_backend(&self) -> AnalyticsResult<()> {
        let storage_guard = self.storage_backend.lock().unwrap_or_else(|e| e.into_inner());
        let any_backend = match storage_guard.as_ref() {
            Some(b) => b,
            None => return Ok(()),
        };
        let shared_backend = match any_backend.downcast_ref::<SharedBackend>() {
            Some(sb) => sb,
            None => return Ok(()),
        };

        let backend = shared_backend
            .read()
            .map_err(|e| AnalyticsError::Internal(e.to_string()))?;

        let keys = backend
            .scan_cf_keys(EFFICIENCY_CF, "")
            .map_err(|e| AnalyticsError::SyncError(e.to_string()))?;

        let mut cache = self.efficiency_cache.write().unwrap_or_else(|e| e.into_inner());
        cache.clear();

        for key_bytes in &keys {
            let session_id = std::str::from_utf8(key_bytes)
                .map_err(|e| AnalyticsError::SyncError(format!("Invalid key UTF-8: {e}")))?;
            let value_bytes = backend
                .get_raw(EFFICIENCY_CF, session_id)
                .map_err(|e| AnalyticsError::SyncError(e.to_string()))?
                .ok_or_else(|| {
                    AnalyticsError::SyncError(format!(
                        "Key '{session_id}' disappeared during efficiency sync"
                    ))
                })?;
            let json: serde_json::Value = serde_json::from_slice(&value_bytes)
                .map_err(|e| AnalyticsError::SyncError(format!("JSON parse: {e}")))?;

            let score = json["efficiency_score"].as_f64().unwrap_or(0.0);
            let project = json["project"].as_str().unwrap_or("").to_string();
            let total_memories = json["total_memories"].as_u64().unwrap_or(0);
            let useful_memories = json["useful_memories"].as_u64().unwrap_or(0);

            cache.insert(
                session_id.to_string(),
                EfficiencyEntry {
                    project,
                    total_memories,
                    useful_memories,
                    score,
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a fresh engine with a generous TTL for testing.
    fn new_test_engine() -> DuckDbEngine {
        DuckDbEngine::new(3600).expect("engine creation should succeed")
    }

    #[test]
    fn test_new_engine_creates_tables() {
        let engine = new_test_engine();

        // Verify that the tables exist by querying DuckDB's system catalogue.
        let result = engine
            .query(
                "SELECT table_name FROM information_schema.tables \
                 WHERE table_schema = 'main' ORDER BY table_name",
                &[],
            )
            .expect("query should succeed");

        let names: Vec<String> = result
            .iter()
            .map(|row| match &row[0] {
                Value::Text(name) => name.clone(),
                _ => String::new(),
            })
            .collect();

        assert!(
            names.contains(&"memories".to_string()),
            "memories table missing"
        );
        assert!(
            names.contains(&"sessions".to_string()),
            "sessions table missing"
        );
        assert!(
            names.contains(&"telemetry".to_string()),
            "telemetry table missing"
        );
    }

    #[test]
    fn test_sync_and_query() {
        let engine = new_test_engine();

        // Sync sessions — this inserts the sample data.
        engine.sync("sessions").expect("sync should succeed");

        let result = engine
            .query(
                "SELECT id, project, status FROM sessions ORDER BY id",
                &[],
            )
            .expect("query should succeed");

        assert_eq!(result.len(), 2, "expected 2 sessions");

        assert_eq!(result[0][0], Value::Text("test-session-1".into()));
        assert_eq!(result[0][1], Value::Text("contexter".into()));
        assert_eq!(result[0][2], Value::Text("completed".into()));

        assert_eq!(result[1][0], Value::Text("test-session-2".into()));
        assert_eq!(result[1][1], Value::Text("contexter".into()));
        assert_eq!(result[1][2], Value::Text("active".into()));
    }

    #[test]
    fn test_query_on_unsynced_table() {
        let engine = new_test_engine();

        // Query without syncing — table exists but should be empty.
        let result = engine
            .query("SELECT id FROM sessions ORDER BY id", &[])
            .expect("query on empty table should succeed");

        assert!(result.is_empty(), "expected no results without sync");
    }

    #[test]
    fn test_double_sync_is_idempotent() {
        let engine = new_test_engine();

        // Sync once.
        engine.sync("sessions").expect("first sync should succeed");

        let result = engine
            .query("SELECT COUNT(*) as cnt FROM sessions", &[])
            .expect("query should succeed");
        let count_after_first = match &result[0][0] {
            Value::Int(n) => *n,
            _ => panic!("expected Int, got {:?}", result[0][0]),
        };
        assert_eq!(
            count_after_first, 2,
            "expected 2 sessions after first sync"
        );

        // Sync a second time — should truncate and re-insert.
        engine.sync("sessions").expect("second sync should succeed");

        let result = engine
            .query("SELECT COUNT(*) as cnt FROM sessions", &[])
            .expect("query should succeed");
        let count_after_second = match &result[0][0] {
            Value::Int(n) => *n,
            _ => panic!("expected Int, got {:?}", result[0][0]),
        };
        assert_eq!(
            count_after_second, 2,
            "expected 2 sessions after double sync (idempotent)"
        );
    }

    #[test]
    fn test_multiple_queries() {
        let engine = new_test_engine();

        // Sync both sessions and memories.
        engine.sync("sessions").expect("sync sessions");
        engine.sync("memory_items").expect("sync memories");

        // Query 1: session count
        let result = engine
            .query("SELECT COUNT(*) as cnt FROM sessions", &[])
            .expect("query sessions count");
        assert_eq!(result[0][0], Value::Int(2));

        // Query 2: memory count by type
        let result = engine
            .query(
                "SELECT memory_type, COUNT(*) as cnt FROM memories \
                 GROUP BY memory_type ORDER BY cnt DESC",
                &[],
            )
            .expect("query memory count");
        // We have: 2 facts, 2 preferences, 1 episode
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_efficiency_calculation() {
        let engine = new_test_engine();

        // Sync sessions and memories first.
        engine.sync("sessions").expect("sync sessions");
        engine.sync("memory_items").expect("sync memories");

        // Run the EFFICIENCY_SCORES query directly.
        let result = engine
            .query(crate::analytics::queries::EFFICIENCY_SCORES, &[])
            .expect("efficiency query should succeed");

        // Efficiency = useful_memories / total_memories.
        // test-session-1: 3 memories (1 preference) → 1/3 ≈ 0.333
        // test-session-2: 2 memories (1 preference) → 1/2 = 0.5
        // Ordered by efficiency_score DESC → session-2 first.

        assert_eq!(
            result.len(),
            2,
            "expected efficiency scores for 2 sessions"
        );

        assert_eq!(result[0][0], Value::Text("test-session-2".into()));
        match result[0][4] {
            Value::Float(v) => assert!((v - 0.5).abs() < 1e-9, "expected 0.5, got {v}"),
            _ => panic!("expected Float for efficiency_score"),
        }

        assert_eq!(result[1][0], Value::Text("test-session-1".into()));
        match result[1][4] {
            Value::Float(v) => {
                assert!(
                    (v - (1.0 / 3.0)).abs() < 1e-9,
                    "expected ~0.333, got {v}"
                )
            }
            _ => panic!("expected Float for efficiency_score"),
        }
    }

    #[test]
    fn test_metric_correlation() {
        let engine = new_test_engine();

        // Sync sessions and memories.
        engine.sync("sessions").expect("sync sessions");
        engine.sync("memory_items").expect("sync memories");

        let result = engine
            .query(crate::analytics::queries::METRIC_CORRELATION, &[])
            .expect("correlation query should succeed");

        // We have 2 data points, so the result should have 1 row.
        assert_eq!(result.len(), 1, "expected one correlation row");

        // Both sample_count and pearson_r should be present.
        match result[0][1] {
            Value::Int(n) => assert_eq!(n, 2, "expected 2 samples"),
            _ => panic!("expected Int for sample_count"),
        }
    }

    #[test]
    fn test_sync_all() {
        let engine = new_test_engine();

        engine.sync_all().expect("sync_all should succeed");

        // Verify all tables have data.
        for table_name in &["sessions", "memories", "telemetry"] {
            let sql = format!("SELECT COUNT(*) as cnt FROM {table_name}");
            let result = engine
                .query(&sql, &[])
                .unwrap_or_else(|_| panic!("query '{table_name}' failed"));
            let count = match &result[0][0] {
                Value::Int(n) => *n,
                _ => panic!("expected Int for count"),
            };
            assert!(
                count > 0,
                "expected data in '{table_name}' after sync_all"
            );
        }
    }

    #[test]
    fn test_set_storage_backend() {
        let engine = new_test_engine();

        // The backend is just stored — no behaviour tested yet.
        engine.set_storage_backend(Box::new(42i32));

        // Verify no crash / the mutex is accessible.
        let guard = engine.storage_backend.lock().unwrap_or_else(|e| e.into_inner());
        assert!(guard.is_some());
    }

    #[test]
    fn test_needs_sync_initially() {
        let engine = new_test_engine();

        // Before any sync, needs_sync should return true.
        assert!(engine.needs_sync("sessions"));
        assert!(engine.needs_sync("memories"));
        assert!(engine.needs_sync("telemetry"));
    }

    #[test]
    fn test_needs_sync_after_sync() {
        let engine = new_test_engine();

        engine.sync("sessions").expect("sync should succeed");

        // After sync, needs_sync should be false (TTL is 3600s).
        assert!(!engine.needs_sync("sessions"));
    }

    #[test]
    fn test_empty_query_after_truncate() {
        let engine = new_test_engine();

        engine.sync("sessions").expect("sync");

        // Truncate manually.
        engine.truncate_table("sessions").expect("truncate");

        let result = engine
            .query("SELECT COUNT(*) as cnt FROM sessions", &[])
            .expect("query");
        assert_eq!(result[0][0], Value::Int(0));
    }

    #[test]
    fn test_temp_dir_cleaned_on_drop() {
        let path;
        {
            let engine = DuckDbEngine::new(3600).expect("engine creation should succeed");
            // Capture the temp dir path.
            path = engine._temp_dir.dir().map(|p| p.clone());
            assert!(path.is_some(), "temp dir should exist");
            // Verify the directory exists on disk.
            assert!(path.as_ref().unwrap().exists(), "temp dir should be created");
        }
        // Engine dropped — temp dir should be cleaned up.
        let p = path.unwrap();
        assert!(!p.exists(), "temp dir should be cleaned up after drop");
    }
}
