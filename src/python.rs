//! PyO3 bridge for the Contexter storage engine.
//!
//! Exposes [`Engine`] as a `#[pyclass]` so Python callers can open a database,
//! create/read/update/delete sessions, memories, agents, and skills, manage
//! settings, query the audit log, and invoke maintenance operations.
//!
//! # Data boundary
//!
//! All domain data crosses the Python boundary as **JSON strings**.  Python
//! callers serialize their payloads with `json.dumps(...)` before calling into
//! the bridge and deserialize the JSON string result with `json.loads(...)`.
//! This avoids complex PyO3 type mappings and keeps the bridge thin.
//!
//! # Thread safety
//!
//! The inner [`Engine`] is wrapped in `Arc` and is `Send + Sync`.  Python
//! callers on the GIL should use `asyncio.to_thread()` with a
//! `ThreadPoolExecutor` to avoid blocking the event loop.
//!
//! # Panic safety
//!
//! Every Python-facing method wraps its body in [`catch_panic`] to prevent
//! Rust panics from unwinding across the Python FFI boundary.
//!
//! # Usage (Python)
//!
//! ```python
//! import json
//! from contexter import Engine
//!
//! engine = Engine.open("./contexter.db")
//!
//! # Create a session
//! session_json = engine.create_session(json.dumps({
//!     "project": "my-project",
//!     "agentId": "00000000-0000-0000-0000-000000000001",
//! }))
//! session = json.loads(session_json)
//! print(session["id"])
//! ```

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use serde::de::Error as SerdeError;
use uuid::Uuid;

use crate::engine::Engine;
use crate::error::EngineError;
use crate::types::*;

// ---------------------------------------------------------------------------
// Error conversion
// ---------------------------------------------------------------------------

/// Convert an [`EngineError`] into a Python [`PyRuntimeError`].
fn map_err(e: EngineError) -> PyErr {
    PyErr::new::<PyRuntimeError, _>(e.to_string())
}

/// Wrap a closure with `catch_unwind` so Rust panics never cross the
/// Python FFI boundary.
///
/// If the closure panics, the panic message is converted into a
/// [`PyRuntimeError`].  The closure is wrapped in [`AssertUnwindSafe`]
/// because every Python-facing method captures `&self` — this is sound
/// since we never leave `self` in an invalid state after a panic in the
/// bridge layer.
fn catch_panic<F, T>(f: F) -> PyResult<T>
where
    F: FnOnce() -> PyResult<T>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(panic_info) => {
            let msg = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "unknown cause".to_string()
            };
            Err(PyErr::new::<PyRuntimeError, _>(msg))
        }
    }
}

/// Maximum permitted JSON nesting depth.
///
/// Prevents stack-overflow or resource-exhaustion attacks via deeply nested
/// JSON payloads while accepting legitimate input (most real-world payloads
/// nest fewer than 16 levels).
const MAX_JSON_DEPTH: usize = 64;

/// Scan a JSON string for nesting depth without fully parsing it.
///
/// Returns an error if the nesting depth of objects or arrays exceeds
/// [`MAX_JSON_DEPTH`].  String literals (including escaped quotes) are
/// skipped correctly so braces inside strings do not affect the count.
fn check_json_depth(input: &str) -> Result<(), String> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if in_string {
            match ch {
                '\\' => {
                    // Skip the next character (escape sequence).
                    chars.next();
                }
                '"' => {
                    in_string = false;
                }
                _ => {}
            }
        } else {
            match ch {
                '"' => {
                    in_string = true;
                }
                '{' | '[' => {
                    depth += 1;
                    if depth > MAX_JSON_DEPTH {
                        return Err(format!(
                            "JSON nesting depth exceeds limit of {MAX_JSON_DEPTH}"
                        ));
                    }
                }
                '}' | ']' => {
                    if depth == 0 {
                        return Err("unexpected closing bracket/brace".into());
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
    }

    if depth != 0 {
        return Err("unterminated JSON object or array".into());
    }
    Ok(())
}

/// Parse a JSON string with recursion protection.
///
/// First checks nesting depth via [`check_json_depth`], then parses using
/// serde_json's built-in recursion limit (default depth 128).  This avoids
/// both false positives from legitimate input and stack-overflow attacks
/// from deeply nested payloads.
fn from_str<T>(s: &str) -> serde_json::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    // Depth check first — early rejection before full parsing.
    check_json_depth(s).map_err(|msg| {
        SerdeError::custom(msg)
    })?;
    serde_json::from_str(s)
}

// ---------------------------------------------------------------------------
// PyEngine
// ---------------------------------------------------------------------------

/// Python wrapper around the Contexter storage engine.
///
/// All domain data crosses the boundary as JSON strings.
#[pyclass(name = "Engine")]
pub struct PyEngine {
    inner: Arc<Engine>,
}

#[pymethods]
impl PyEngine {
    // =======================================================================
    // Construction
    // =======================================================================

    /// Open or create a database at `path`.
    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        catch_panic(|| {
            let engine = Engine::open(path).map_err(map_err)?;
            Ok(Self {
                inner: Arc::new(engine),
            })
        })
    }

    // =======================================================================
    // Session CRUD
    // =======================================================================

    /// Create a new session.
    ///
    /// `session_json` — JSON-encoded [`NewSession`].
    /// Returns JSON-encoded [`Session`].
    fn create_session(&self, session_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewSession = from_str(session_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid session JSON: {e}")))?;
            let session = self.inner.create_session(new).map_err(map_err)?;
            serde_json::to_string(&session).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Retrieve a session by its unique identifier.
    ///
    /// Returns `None` (Python `None`) when the session does not exist.
    fn get_session(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            match self.inner.get_session(uuid).map_err(map_err)? {
                Some(session) => serde_json::to_string(&session).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                None => Ok(None),
            }
        })
    }

    /// List sessions matching the given filter criteria.
    ///
    /// `filter_json` — JSON-encoded [`SessionFilter`].
    /// Returns a JSON array of [`Session`] objects.
    ///
    /// The offset and limit are embedded in the filter JSON.
    fn list_sessions(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: SessionFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let sessions = self.inner.list_sessions(&filter).map_err(map_err)?;
            serde_json::to_string(&sessions).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Partially update an existing session.
    ///
    /// Returns the updated session JSON, or `None` if no session with the
    /// given `id` exists.
    fn update_session(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            let patch: SessionPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_session(uuid, &patch) {
                Ok(session) => serde_json::to_string(&session).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    /// Permanently delete a session.
    ///
    /// Idempotent — deleting a non-existent session returns `None` without
    /// error.
    fn delete_session(&self, id: &str) -> PyResult<()> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            self.inner.delete_session(uuid).map_err(map_err)
        })
    }

    /// Count sessions matching the given filter criteria.
    fn count_sessions(&self, filter_json: &str) -> PyResult<usize> {
        catch_panic(|| {
            let filter: SessionFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let count = self.inner.count_sessions(&filter).map_err(map_err)?;
            Ok(count as usize)
        })
    }

    // =======================================================================
    // Memory CRUD
    // =======================================================================

    /// Create a new memory.
    ///
    /// `memory_json` — JSON-encoded [`NewMemory`].
    /// Returns JSON-encoded [`Memory`].
    fn create_memory(&self, memory_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewMemory = from_str(memory_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid memory JSON: {e}")))?;
            let memory = self.inner.create_memory(new).map_err(map_err)?;
            serde_json::to_string(&memory).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Create a new memory with large content passed as raw bytes.
    ///
    /// `meta_json` — JSON-encoded [`NewMemory`] **without** the `content`
    /// field (all other metadata).
    /// `content` — Raw UTF-8 bytes for the `content` field.
    ///
    /// This avoids a double JSON-encoding round-trip for memories whose
    /// content exceeds 100 KB.  The Python bridge splits the payload and
    /// sends the content as `PyBytes`.
    fn create_memory_bytes(&self, meta_json: &str, content: &[u8]) -> PyResult<String> {
        catch_panic(|| {
            let mut meta: NewMemory = from_str(meta_json).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid memory metadata JSON: {e}"))
            })?;
            meta.content = String::from_utf8(content.to_vec()).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid UTF-8 content: {e}"))
            })?;
            let memory = self.inner.create_memory(meta).map_err(map_err)?;
            serde_json::to_string(&memory).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Retrieve a memory by its unique identifier.
    fn get_memory(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            match self.inner.get_memory(uuid).map_err(map_err)? {
                Some(memory) => serde_json::to_string(&memory).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                None => Ok(None),
            }
        })
    }

    /// Search memories using structured query criteria.
    ///
    /// `query_json` — JSON-encoded [`MemorySearchQuery`].
    /// Returns a JSON array of [`Memory`] objects.
    fn search_memories(&self, query_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let query: MemorySearchQuery = from_str(query_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid query JSON: {e}")))?;
            let memories = self.inner.search_memories(&query).map_err(map_err)?;
            serde_json::to_string(&memories).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Partially update an existing memory.
    fn update_memory(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            let patch: MemoryPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_memory(uuid, &patch) {
                Ok(memory) => serde_json::to_string(&memory).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    /// Partially update an existing memory with large content as raw bytes.
    ///
    /// `id` — The UUID of the memory to update.
    /// `patch_meta_json` — JSON-encoded [`MemoryPatch`] **without** the
    /// `content` field.
    /// `content` — Raw UTF-8 bytes for the new `content` value.
    fn update_memory_bytes(
        &self,
        id: &str,
        patch_meta_json: &str,
        content: &[u8],
    ) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            let content_str = String::from_utf8(content.to_vec()).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid UTF-8 content: {e}"))
            })?;
            let mut patch: MemoryPatch = from_str(patch_meta_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            patch.content = Some(content_str);
            match self.inner.update_memory(uuid, &patch) {
                Ok(memory) => serde_json::to_string(&memory).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    /// Permanently delete a memory.
    fn delete_memory(&self, id: &str) -> PyResult<()> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            self.inner.delete_memory(uuid).map_err(map_err)
        })
    }

    /// Count memories matching the given filter criteria.
    fn count_memories(&self, filter_json: &str) -> PyResult<usize> {
        catch_panic(|| {
            let filter: MemoryFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let count = self.inner.count_memories(&filter).map_err(map_err)?;
            Ok(count as usize)
        })
    }

    // =======================================================================
    // Agent CRUD
    // =======================================================================

    /// Register a new agent.
    ///
    /// `agent_json` — JSON-encoded [`NewAgent`].
    /// Returns JSON-encoded [`Agent`].
    fn create_agent(&self, agent_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewAgent = from_str(agent_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid agent JSON: {e}")))?;
            let agent = self.inner.create_agent(new).map_err(map_err)?;
            serde_json::to_string(&agent).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Retrieve an agent by its unique identifier.
    fn get_agent(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            match self.inner.get_agent(uuid).map_err(map_err)? {
                Some(agent) => serde_json::to_string(&agent).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                None => Ok(None),
            }
        })
    }

    /// List agents.
    ///
    /// `filter_json` — JSON-encoded [`AgentFilter`].
    /// Returns a JSON array of [`Agent`] objects.
    fn list_agents(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: AgentFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let agents = self.inner.list_agents(&filter).map_err(map_err)?;
            serde_json::to_string(&agents).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Partially update an existing agent.
    fn update_agent(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            let patch: AgentPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_agent(uuid, &patch) {
                Ok(agent) => serde_json::to_string(&agent).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    /// Permanently delete an agent.
    fn delete_agent(&self, id: &str) -> PyResult<bool> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            self.inner.delete_agent(uuid).map_err(map_err)?;
            Ok(true)
        })
    }

    // =======================================================================
    // Skill CRUD
    // =======================================================================

    /// Register a new skill.
    ///
    /// `skill_json` — JSON-encoded [`NewSkill`].
    /// Returns JSON-encoded [`Skill`].
    fn create_skill(&self, skill_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewSkill = from_str(skill_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid skill JSON: {e}")))?;
            let skill = self.inner.create_skill(new).map_err(map_err)?;
            serde_json::to_string(&skill).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Retrieve a skill by its unique identifier.
    fn get_skill(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            match self.inner.get_skill(uuid).map_err(map_err)? {
                Some(skill) => serde_json::to_string(&skill).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                None => Ok(None),
            }
        })
    }

    /// List skills.
    ///
    /// `filter_json` — JSON-encoded [`SkillFilter`].
    /// Returns a JSON array of [`Skill`] objects.
    fn list_skills(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: SkillFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let skills = self.inner.list_skills(&filter).map_err(map_err)?;
            serde_json::to_string(&skills).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Partially update an existing skill.
    fn update_skill(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            let patch: SkillPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_skill(uuid, &patch) {
                Ok(skill) => serde_json::to_string(&skill).map(Some).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    /// Permanently delete a skill.
    fn delete_skill(&self, id: &str) -> PyResult<bool> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            self.inner.delete_skill(uuid).map_err(map_err)?;
            Ok(true)
        })
    }

    // =======================================================================
    // Settings
    // =======================================================================

    /// Persist a setting value.
    fn set_setting(&self, key: &str, value: &str) -> PyResult<()> {
        catch_panic(|| self.inner.set_setting(key, value).map_err(map_err))
    }

    /// Retrieve a setting value by key.
    ///
    /// Returns `None` (Python `None`) when the key does not exist.
    fn get_setting(&self, key: &str) -> PyResult<Option<String>> {
        catch_panic(|| self.inner.get_setting(key).map_err(map_err))
    }

    // =======================================================================
    // Audit log
    // =======================================================================

    /// Append a new entry to the audit log.
    fn log_audit(&self, entry_json: &str) -> PyResult<()> {
        catch_panic(|| {
            let entry: NewAuditEntry = from_str(entry_json).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid audit entry JSON: {e}"))
            })?;
            self.inner.log_audit(entry).map_err(map_err)
        })
    }

    /// Query the audit log with optional filters.
    ///
    /// `filter_json` — JSON-encoded [`AuditFilter`].
    /// Returns a JSON array of [`AuditEntry`] objects.
    fn query_audit(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: AuditFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let entries = self.inner.query_audit(&filter).map_err(map_err)?;
            serde_json::to_string(&entries).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    // =======================================================================
    // Maintenance
    // =======================================================================

    /// Flush any pending writes to durable storage.
    fn flush(&self) -> PyResult<()> {
        catch_panic(|| self.inner.flush().map_err(map_err))
    }

    /// Trigger a checkpoint / compaction and return the current RocksDB
    /// sequence number.
    fn checkpoint(&self) -> PyResult<u64> {
        catch_panic(|| self.inner.checkpoint().map_err(map_err))
    }

    /// Report storage size information.
    ///
    /// Returns a JSON object with `perCf`, `walSize`, and `total` fields.
    fn storage_size(&self) -> PyResult<String> {
        catch_panic(|| {
            let size = self.inner.storage_size().map_err(map_err)?;
            serde_json::to_string(&size).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Snapshot of L1 cache performance counters.
    ///
    /// Returns a JSON object with `hits`, `misses`, `totalOps`, `hitRatio`,
    /// and `entriesByType`.
    fn cache_telemetry(&self) -> PyResult<String> {
        catch_panic(|| {
            let tel = self.inner.cache_telemetry();
            serde_json::to_string(&tel).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }

    /// Clear **all** entries from the L1 cache.
    fn clear_cache(&self) -> PyResult<()> {
        catch_panic(|| {
            self.inner.clear_cache();
            Ok(())
        })
    }

    /// Clear all cached entries for a specific entity type
    /// (e.g. `"session"`, `"memory"`, `"agent"`, `"skill"`).
    fn clear_cache_type(&self, entity_type: &str) -> PyResult<()> {
        catch_panic(|| {
            self.inner.clear_cache_type(entity_type);
            Ok(())
        })
    }

    // =======================================================================
    // Generic raw storage (for testing and low-level access)
    // =======================================================================

    /// Store raw bytes under the given `key` in the named column family.
    fn store(&self, cf_name: &str, key: &str, value: Vec<u8>) -> PyResult<()> {
        catch_panic(|| self.inner.store(cf_name, key, &value).map_err(map_err))
    }

    /// Retrieve raw bytes for the given `key` from the named column family.
    /// Returns `None` when the key does not exist.
    fn get(&self, cf_name: &str, key: &str) -> PyResult<Option<Vec<u8>>> {
        catch_panic(|| self.inner.get(cf_name, key).map_err(map_err))
    }

    // =======================================================================
    // Status / health
    // =======================================================================

    /// Basic status / health check.
    ///
    /// Returns a JSON object with `status`, `version`, and `cacheTelemetry`.
    fn status(&self) -> PyResult<String> {
        catch_panic(|| {
            let tel = self.inner.cache_telemetry();
            let health = serde_json::json!({
                "status": "ok",
                "version": env!("CARGO_PKG_VERSION"),
                "cacheTelemetry": {
                    "hits": tel.hits,
                    "misses": tel.misses,
                    "totalOps": tel.total_ops,
                    "hitRatio": tel.hit_ratio,
                    "entriesByType": tel.entries_by_type,
                },
            });
            serde_json::to_string(&health).map_err(|e: serde_json::Error| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a hyphenated UUID string, returning a [`PyValueError`] on failure.
fn parse_uuid(s: &str, entity: &str) -> PyResult<Uuid> {
    Uuid::parse_str(s)
        .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid {entity} id {s:?}: {e}")))
}

// ---------------------------------------------------------------------------
// PyO3 module initialisation
// ---------------------------------------------------------------------------

/// Contexter native module.
///
/// Registers the [`PyEngine`] class so Python callers can access the storage
/// engine.
#[pymodule]
fn contexter(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyEngine>()?;
    Ok(())
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper — create a temporary [`PyEngine`] for testing.
    fn setup() -> (PyEngine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine = PyEngine::open(dir.path().to_str().unwrap()).expect("open");
        (engine, dir)
    }

    /// Helper — parse a JSON string into a serde_json::Value for assertion.
    // SAFETY: direct serde_json::from_str is acceptable here — this is a test
    // helper that only parses internal test data with bounded nesting.
    fn parse_json(s: &str) -> serde_json::Value {
        serde_json::from_str(s).expect("valid JSON")
    }

    /// Helper — create a minimal new-session JSON payload.
    fn new_session_json(project: &str) -> String {
        serde_json::json!({
            "project": project,
            "agentId": Uuid::now_v7().to_string(),
        })
        .to_string()
    }

    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_engine_open() {
        let (_engine, _dir) = setup();
        // Engine opened without error — sufficient.
    }

    #[test]
    fn test_py_engine_status() {
        let (engine, _dir) = setup();
        let status_json = engine.status().expect("status");
        let status = parse_json(&status_json);
        assert_eq!(status["status"], "ok");
        assert!(status["version"].is_string());
        assert!(status["cacheTelemetry"]["totalOps"].is_number());
    }

    // -----------------------------------------------------------------------
    // Session CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_session_create_get() {
        let (engine, _dir) = setup();

        let json = engine
            .create_session(&new_session_json("test-project"))
            .expect("create session");
        let session = parse_json(&json);
        assert_eq!(session["project"], "test-project");
        assert_eq!(session["turnCount"], 0);

        let id = session["id"].as_str().unwrap();
        let fetched_json = engine
            .get_session(id)
            .expect("get session")
            .expect("session exists");
        let fetched = parse_json(&fetched_json);
        assert_eq!(fetched["id"], id);
    }

    #[test]
    fn test_py_session_get_nonexistent() {
        let (engine, _dir) = setup();
        let result = engine
            .get_session("00000000-0000-0000-0000-000000000000")
            .expect("get nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_py_session_list() {
        let (engine, _dir) = setup();

        let project = "list-test";
        let agent_id = Uuid::now_v7().to_string();

        // Create 3 sessions.
        for i in 0..3 {
            let mut j = serde_json::json!({
                "project": project,
                "agentId": agent_id,
            });
            if i == 0 {
                j["status"] = serde_json::json!("active");
            } else {
                j["status"] = serde_json::json!("completed");
            }
            engine.create_session(&j.to_string()).expect("create");
        }

        let filter = serde_json::json!({
            "project": project,
        });
        let list_json = engine.list_sessions(&filter.to_string()).expect("list");
        // SAFETY: direct serde_json::from_str is acceptable here — parses
        // internal engine JSON with bounded nesting.
        let list: Vec<serde_json::Value> = serde_json::from_str(&list_json).expect("parse list");
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_py_session_update() {
        let (engine, _dir) = setup();
        let json = engine
            .create_session(&new_session_json("update-test"))
            .expect("create");
        let session = parse_json(&json);
        let id = session["id"].as_str().unwrap();

        let patch = serde_json::json!({ "turnCount": 42 });
        let updated = engine
            .update_session(id, &patch.to_string())
            .expect("update")
            .expect("session exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["turnCount"], 42);
    }

    #[test]
    fn test_py_session_update_nonexistent() {
        let (engine, _dir) = setup();
        let patch = serde_json::json!({ "turnCount": 1 });
        let result = engine
            .update_session("00000000-0000-0000-0000-000000000000", &patch.to_string())
            .expect("update nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_py_session_delete() {
        let (engine, _dir) = setup();
        let json = engine
            .create_session(&new_session_json("delete-test"))
            .expect("create");
        let session = parse_json(&json);
        let id = session["id"].as_str().unwrap();

        engine.delete_session(id).expect("delete");
        assert!(engine.get_session(id).expect("get after delete").is_none());
    }

    #[test]
    fn test_py_session_delete_idempotent() {
        let (engine, _dir) = setup();
        // Deleting a non-existent session should not error.
        engine
            .delete_session("00000000-0000-0000-0000-000000000000")
            .expect("delete nonexistent");
    }

    #[test]
    fn test_py_session_count() {
        let (engine, _dir) = setup();
        let project = "count-test";
        let agent = Uuid::now_v7().to_string();

        for _ in 0..3 {
            engine
                .create_session(
                    &serde_json::json!({
                        "project": project,
                        "agentId": agent,
                    })
                    .to_string(),
                )
                .expect("create");
        }

        let filter = serde_json::json!({ "project": project });
        let count = engine.count_sessions(&filter.to_string()).expect("count");
        assert_eq!(count, 3);
    }

    // -----------------------------------------------------------------------
    // Memory CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_memory_crud() {
        let (engine, _dir) = setup();

        let new_memory = serde_json::json!({
            "sessionId": Uuid::now_v7().to_string(),
            "agentId": Uuid::now_v7().to_string(),
            "memoryType": "fact",
            "content": "The quick brown fox",
        });

        let created = engine
            .create_memory(&new_memory.to_string())
            .expect("create memory");
        let created = parse_json(&created);
        assert_eq!(created["content"], "The quick brown fox");
        assert_eq!(created["version"], 1);

        let id = created["id"].as_str().unwrap();

        // Get
        let fetched = engine.get_memory(id).expect("get").expect("memory exists");
        let fetched = parse_json(&fetched);
        assert_eq!(fetched["id"], id);

        // Update
        let patch = serde_json::json!({ "content": "updated content" });
        let updated = engine
            .update_memory(id, &patch.to_string())
            .expect("update")
            .expect("memory exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["content"], "updated content");
        assert_eq!(updated["version"], 2);

        // Search
        let query = serde_json::json!({ "keywords": "updated" });
        let results = engine.search_memories(&query.to_string()).expect("search");
        // SAFETY: direct serde_json::from_str is acceptable here — parses
        // internal engine JSON with bounded nesting.
        let results: Vec<serde_json::Value> =
            serde_json::from_str(&results).expect("parse search results");
        assert!(results.iter().any(|m| m["id"] == id));

        // Count
        let filter = serde_json::json!({ "memoryType": "fact" });
        let count = engine.count_memories(&filter.to_string()).expect("count");
        assert!(count > 0);

        // Delete
        engine.delete_memory(id).expect("delete");
        assert!(engine.get_memory(id).expect("get after delete").is_none());
    }

    #[test]
    fn test_py_create_memory_bytes() {
        let (engine, _dir) = setup();

        // Use create_memory_bytes with metadata as JSON + content as raw bytes.
        let meta = serde_json::json!({
            "sessionId": Uuid::now_v7().to_string(),
            "agentId": Uuid::now_v7().to_string(),
            "memoryType": "fact",
        });
        let content = b"large content via PyBytes path";
        let created = engine
            .create_memory_bytes(&meta.to_string(), content)
            .expect("create memory via bytes");
        let created = parse_json(&created);
        assert_eq!(created["content"], "large content via PyBytes path");
        assert_eq!(created["version"], 1);

        // Verify it persisted and is retrievable via get_memory.
        let id = created["id"].as_str().unwrap();
        let fetched = engine
            .get_memory(id)
            .expect("get")
            .expect("memory should exist");
        let fetched = parse_json(&fetched);
        assert_eq!(fetched["content"], "large content via PyBytes path");
    }

    #[test]
    fn test_py_update_memory_bytes() {
        let (engine, _dir) = setup();

        // Create a memory via the normal JSON path first.
        let new_memory = serde_json::json!({
            "sessionId": Uuid::now_v7().to_string(),
            "agentId": Uuid::now_v7().to_string(),
            "memoryType": "fact",
            "content": "original content",
        });
        let created = engine
            .create_memory(&new_memory.to_string())
            .expect("create memory");
        let created = parse_json(&created);
        let id = created["id"].as_str().unwrap();

        // Update via the bytes path.
        let patch_meta = serde_json::json!({}); // no other fields
        let new_content = b"updated via PyBytes path";
        let updated = engine
            .update_memory_bytes(id, &patch_meta.to_string(), new_content)
            .expect("update via bytes")
            .expect("memory exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["content"], "updated via PyBytes path");
        assert_eq!(updated["version"], 2);
    }

    #[test]
    fn test_py_memory_bytes_invalid_utf8_produces_error() {
        let (engine, _dir) = setup();
        let meta = serde_json::json!({
            "sessionId": Uuid::now_v7().to_string(),
            "agentId": Uuid::now_v7().to_string(),
            "memoryType": "fact",
        });
        // Invalid UTF-8 bytes (0xFF is not valid UTF-8).
        let invalid_bytes = [0xFF, 0xFE, 0x00];
        let result = engine.create_memory_bytes(&meta.to_string(), &invalid_bytes);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("invalid UTF-8") || err.contains("utf8") || err.contains("UTF-8"),
            "error should mention invalid UTF-8: {err}"
        );
    }

    #[test]
    fn test_py_memory_bytes_update_nonexistent() {
        let (engine, _dir) = setup();
        let patch_meta = serde_json::json!({});
        let content = b"new content";
        let result = engine.update_memory_bytes(
            "00000000-0000-0000-0000-000000000000",
            &patch_meta.to_string(),
            content,
        );
        // Should return None (NotFound is mapped to Ok(None)).
        assert!(result.is_ok(), "non-existent update should not error");
        assert!(result.unwrap().is_none(), "should return None");
    }

    // -----------------------------------------------------------------------
    // Agent / Skill CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_agent_skill() {
        let (engine, _dir) = setup();

        // Create agent.
        let new_agent = serde_json::json!({
            "name": "test-agent",
            "type": "chat",
            "description": "A test agent",
        });
        let json = engine
            .create_agent(&new_agent.to_string())
            .expect("create agent");
        let agent = parse_json(&json);
        assert_eq!(agent["name"], "test-agent");
        let agent_id = agent["id"].as_str().unwrap().to_string();

        // Get agent.
        let fetched = engine
            .get_agent(&agent_id)
            .expect("get")
            .expect("agent exists");
        assert!(fetched.contains(&agent_id));

        // List agents.
        let filter = serde_json::json!({});
        let list = engine.list_agents(&filter.to_string()).expect("list");
        // SAFETY: internal engine JSON, bounded nesting
        let list: Vec<serde_json::Value> = serde_json::from_str(&list).expect("parse list");
        assert!(list.iter().any(|a| a["id"] == agent_id));

        // Update agent.
        let patch = serde_json::json!({ "name": "updated-agent" });
        let updated = engine
            .update_agent(&agent_id, &patch.to_string())
            .expect("update")
            .expect("agent exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["name"], "updated-agent");

        // Create skill.
        let new_skill = serde_json::json!({
            "name": "code-review",
            "description": "Review code changes",
            "category": "dev",
        });
        let skill_json = engine
            .create_skill(&new_skill.to_string())
            .expect("create skill");
        let skill = parse_json(&skill_json);
        assert_eq!(skill["name"], "code-review");
        let skill_id = skill["id"].as_str().unwrap().to_string();

        // Get skill.
        let fetched = engine
            .get_skill(&skill_id)
            .expect("get")
            .expect("skill exists");
        assert!(fetched.contains(&skill_id));

        // List skills.
        let filter = serde_json::json!({});
        let list = engine.list_skills(&filter.to_string()).expect("list");
        // SAFETY: internal engine JSON, bounded nesting
        let list: Vec<serde_json::Value> = serde_json::from_str(&list).expect("parse list");
        assert!(list.iter().any(|s| s["id"] == skill_id));

        // Update skill.
        let patch = serde_json::json!({ "name": "super-review" });
        let updated = engine
            .update_skill(&skill_id, &patch.to_string())
            .expect("update")
            .expect("skill exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["name"], "super-review");

        // Delete agent.
        engine.delete_agent(&agent_id).expect("delete agent");
        assert!(engine
            .get_agent(&agent_id)
            .expect("get after delete")
            .is_none());

        // Delete skill.
        engine.delete_skill(&skill_id).expect("delete skill");
        assert!(engine
            .get_skill(&skill_id)
            .expect("get after delete")
            .is_none());
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_settings() {
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

    // -----------------------------------------------------------------------
    // Audit
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_audit() {
        let (engine, _dir) = setup();

        let entry = serde_json::json!({
            "action": "create_session",
            "entityType": "Session",
            "entityId": "abc-123",
        });
        engine.log_audit(&entry.to_string()).expect("log audit");

        let filter = serde_json::json!({});
        let entries = engine.query_audit(&filter.to_string()).expect("query");
        // SAFETY: internal engine JSON, bounded nesting
        let entries: Vec<serde_json::Value> =
            serde_json::from_str(&entries).expect("parse entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["action"], "create_session");
    }

    // -----------------------------------------------------------------------
    // Maintenance
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_maintenance() {
        let (engine, _dir) = setup();

        // Flush on an empty engine should succeed.
        engine.flush().expect("flush");

        // Checkpoint should return a sequence number.
        let seq = engine.checkpoint().expect("checkpoint");
        assert!(seq > 0, "checkpoint seq should be > 0");

        // Storage size should return valid JSON.
        let size_json = engine.storage_size().expect("storage size");
        let size = parse_json(&size_json);
        assert!(size["total"].is_number());

        // Cache telemetry should return valid JSON with zero counters.
        let tel_json = engine.cache_telemetry().expect("cache telemetry");
        let tel = parse_json(&tel_json);
        assert_eq!(tel["totalOps"], 0);
    }

    #[test]
    fn test_py_clear_cache() {
        let (engine, _dir) = setup();

        // Create a session to populate the cache.
        let json = engine
            .create_session(&new_session_json("cache-clear"))
            .expect("create");
        let session = parse_json(&json);
        let id = session["id"].as_str().unwrap().to_string();

        // Warm the cache.
        engine.get_session(&id).expect("get").expect("exists");

        // Clear cache type.
        engine
            .clear_cache_type("session")
            .expect("clear cache type");
        let _tel_result = engine.cache_telemetry().expect("tel");
        // After clearing session, entriesByType may have session: 0 or be absent.

        // Clear all.
        engine.clear_cache().expect("clear cache");
    }

    // -----------------------------------------------------------------------
    // Error handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_invalid_json_returns_error() {
        let (engine, _dir) = setup();

        let result = engine.create_session("not valid json");
        assert!(result.is_err(), "invalid JSON should produce an error");

        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("invalid session JSON") || msg.contains("JSON"),
            "error message should mention JSON: {msg}"
        );
    }

    #[test]
    fn test_py_invalid_uuid_returns_error() {
        let (engine, _dir) = setup();
        let result = engine.get_session("not-a-uuid");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid session id"));
    }

    // -----------------------------------------------------------------------
    // Serialization round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_serialization_roundtrip() {
        let (engine, _dir) = setup();

        // Create a fully populated session via JSON.
        let input = serde_json::json!({
            "project": "roundtrip",
            "agentId": Uuid::now_v7().to_string(),
            "status": "active",
            "metadata": {"env": "test", "version": 1},
        });
        let json = engine.create_session(&input.to_string()).expect("create");
        let output = parse_json(&json);

        // Fields the Engine sets should survive a round-trip through
        // serde_json serialization in the bridge.
        assert_eq!(output["project"], "roundtrip");
        assert_eq!(output["status"], "active");
        assert_eq!(output["metadata"]["env"], "test");
        assert_eq!(output["turnCount"], 0);

        // IDs should be valid UUIDs.
        assert!(Uuid::parse_str(output["id"].as_str().unwrap()).is_ok());

        // Timestamps should be present.
        assert!(output["createdAt"].is_string());
        assert!(output["lastActive"].is_string());
    }

    // -----------------------------------------------------------------------
    // JSON depth checker
    // -----------------------------------------------------------------------

    #[test]
    fn test_json_depth_shallow_ok() {
        assert!(check_json_depth(r#"{"a":1}"#).is_ok());
        assert!(check_json_depth(r#"[1,2,3]"#).is_ok());
        assert!(check_json_depth(r#"{"a":{"b":{"c":1}}}"#).is_ok());
    }

    #[test]
    fn test_json_depth_string_with_braces_ok() {
        // Braces inside strings should not affect depth count.
        assert!(check_json_depth(r#"{"msg":"hello {world}"}"#).is_ok());
        assert!(check_json_depth(r#"{"regex":"a{b}c"}"#).is_ok());
    }

    #[test]
    fn test_json_depth_escaped_quotes_ok() {
        // Escaped quotes inside strings should be handled correctly.
        assert!(check_json_depth(r#"{"text":"he said \"hello\""}"#).is_ok());
    }

    #[test]
    fn test_json_depth_unterminated_fails() {
        // Unterminated opening.
        let result = check_json_depth(r#"{"a":1"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unterminated"));
    }

    #[test]
    fn test_json_depth_unexpected_close_fails() {
        // Extra closing brace.
        let result = check_json_depth(r#"{"a":1}}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unexpected closing"));
    }

    #[test]
    fn test_json_depth_flat_array_accepted() {
        assert!(check_json_depth("[1,2,3,4,5]").is_ok());
    }

    #[test]
    fn test_json_depth_exceeds_limit() {
        let input = "{".repeat(65) + &"}".repeat(65);
        assert!(check_json_depth(&input).is_err(), "depth 65 should exceed MAX_JSON_DEPTH");
        let input_ok = "{".repeat(64) + &"}".repeat(64);
        assert!(check_json_depth(&input_ok).is_ok(), "depth 64 should be at the limit");
    }

    // -----------------------------------------------------------------------
    // Send + Sync
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_engine_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<PyEngine>();
        assert_sync::<PyEngine>();
    }
}
