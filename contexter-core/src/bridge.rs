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

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use uuid::Uuid;

use crate::engine::Engine;
use crate::error::EngineError;
use crate::models::*;

// ---------------------------------------------------------------------------
// Error conversion
// ---------------------------------------------------------------------------

/// Convert an [`EngineError`] into a Python [`PyRuntimeError`].
fn map_err(e: EngineError) -> PyErr {
    PyErr::new::<PyRuntimeError, _>(e.to_string())
}

/// Wrap a closure with `catch_unwind` so Rust panics never cross the
/// Python FFI boundary.
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

/// Parse a JSON string.
fn from_str<T>(s: &str) -> serde_json::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_str(s)
}

// ---------------------------------------------------------------------------
// PyEngine
// ---------------------------------------------------------------------------

/// Python wrapper around the Contexter storage engine.
#[pyclass(name = "Engine")]
pub struct PyEngine {
    inner: Arc<Engine>,
}

#[pymethods]
impl PyEngine {
    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        catch_panic(|| {
            let engine = Engine::open(path).map_err(map_err)?;
            Ok(Self {
                inner: Arc::new(engine),
            })
        })
    }

    // -------------------------------------------------------------------
    // Session CRUD
    // -------------------------------------------------------------------

    fn create_session(&self, session_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewSession = from_str(session_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid session JSON: {e}")))?;
            let session = self.inner.create_session(new).map_err(map_err)?;
            serde_json::to_string(&session).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn get_session(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            match self.inner.get_session(uuid).map_err(map_err)? {
                Some(session) => serde_json::to_string(&session)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                None => Ok(None),
            }
        })
    }

    fn list_sessions(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: SessionFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let sessions = self.inner.list_sessions(&filter).map_err(map_err)?;
            serde_json::to_string(&sessions).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn update_session(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            let patch: SessionPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_session(uuid, &patch) {
                Ok(session) => serde_json::to_string(&session)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    fn delete_session(&self, id: &str) -> PyResult<()> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "session")?;
            self.inner.delete_session(uuid).map_err(map_err)
        })
    }

    fn count_sessions(&self, filter_json: &str) -> PyResult<usize> {
        catch_panic(|| {
            let filter: SessionFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let count = self.inner.count_sessions(&filter).map_err(map_err)?;
            Ok(count as usize)
        })
    }

    // -------------------------------------------------------------------
    // Memory CRUD
    // -------------------------------------------------------------------

    fn create_memory(&self, memory_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewMemory = from_str(memory_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid memory JSON: {e}")))?;
            let memory = self.inner.create_memory(new).map_err(map_err)?;
            serde_json::to_string(&memory).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn create_memory_bytes(&self, meta_json: &str, content: &[u8]) -> PyResult<String> {
        catch_panic(|| {
            let mut meta: NewMemory = from_str(meta_json).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid memory metadata JSON: {e}"))
            })?;
            meta.content = String::from_utf8(content.to_vec()).map_err(|e| {
                PyErr::new::<PyValueError, _>(format!("invalid UTF-8 content: {e}"))
            })?;
            let memory = self.inner.create_memory(meta).map_err(map_err)?;
            serde_json::to_string(&memory).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn get_memory(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            match self.inner.get_memory(uuid).map_err(map_err)? {
                Some(memory) => serde_json::to_string(&memory)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                None => Ok(None),
            }
        })
    }

    fn search_memories(&self, query_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let query: MemorySearchQuery = from_str(query_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid query JSON: {e}")))?;
            let memories = self.inner.search_memories(&query).map_err(map_err)?;
            serde_json::to_string(&memories).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn update_memory(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            let patch: MemoryPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_memory(uuid, &patch) {
                Ok(memory) => serde_json::to_string(&memory)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

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
                Ok(memory) => serde_json::to_string(&memory)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    fn delete_memory(&self, id: &str) -> PyResult<()> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "memory")?;
            self.inner.delete_memory(uuid).map_err(map_err)
        })
    }

    fn count_memories(&self, filter_json: &str) -> PyResult<usize> {
        catch_panic(|| {
            let filter: MemoryFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let count = self.inner.count_memories(&filter).map_err(map_err)?;
            Ok(count as usize)
        })
    }

    // -------------------------------------------------------------------
    // Agent CRUD
    // -------------------------------------------------------------------

    fn create_agent(&self, agent_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewAgent = from_str(agent_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid agent JSON: {e}")))?;
            let agent = self.inner.create_agent(new).map_err(map_err)?;
            serde_json::to_string(&agent).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn get_agent(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            match self.inner.get_agent(uuid).map_err(map_err)? {
                Some(agent) => serde_json::to_string(&agent)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                None => Ok(None),
            }
        })
    }

    fn list_agents(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: AgentFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let agents = self.inner.list_agents(&filter).map_err(map_err)?;
            serde_json::to_string(&agents).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn update_agent(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            let patch: AgentPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_agent(uuid, &patch) {
                Ok(agent) => serde_json::to_string(&agent)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    fn delete_agent(&self, id: &str) -> PyResult<bool> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "agent")?;
            self.inner.delete_agent(uuid).map_err(map_err)?;
            Ok(true)
        })
    }

    // -------------------------------------------------------------------
    // Skill CRUD
    // -------------------------------------------------------------------

    fn create_skill(&self, skill_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let new: NewSkill = from_str(skill_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid skill JSON: {e}")))?;
            let skill = self.inner.create_skill(new).map_err(map_err)?;
            serde_json::to_string(&skill).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn get_skill(&self, id: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            match self.inner.get_skill(uuid).map_err(map_err)? {
                Some(skill) => serde_json::to_string(&skill)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                None => Ok(None),
            }
        })
    }

    fn list_skills(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: SkillFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let skills = self.inner.list_skills(&filter).map_err(map_err)?;
            serde_json::to_string(&skills).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn update_skill(&self, id: &str, patch_json: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            let patch: SkillPatch = from_str(patch_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid patch JSON: {e}")))?;
            match self.inner.update_skill(uuid, &patch) {
                Ok(skill) => serde_json::to_string(&skill)
                    .map(Some)
                    .map_err(|e: serde_json::Error| {
                        PyErr::new::<PyRuntimeError, _>(e.to_string())
                    }),
                Err(EngineError::NotFound { .. }) => Ok(None),
                Err(e) => Err(map_err(e)),
            }
        })
    }

    fn delete_skill(&self, id: &str) -> PyResult<bool> {
        catch_panic(|| {
            let uuid = parse_uuid(id, "skill")?;
            self.inner.delete_skill(uuid).map_err(map_err)?;
            Ok(true)
        })
    }

    // -------------------------------------------------------------------
    // Settings
    // -------------------------------------------------------------------

    fn set_setting(&self, key: &str, value: &str) -> PyResult<()> {
        catch_panic(|| self.inner.set_setting(key, value).map_err(map_err))
    }

    fn get_setting(&self, key: &str) -> PyResult<Option<String>> {
        catch_panic(|| self.inner.get_setting(key).map_err(map_err))
    }

    // -------------------------------------------------------------------
    // Audit log
    // -------------------------------------------------------------------

    fn log_audit(&self, entry_json: &str) -> PyResult<()> {
        catch_panic(|| {
            let entry: NewAuditEntry = from_str(entry_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid audit entry JSON: {e}")))?;
            self.inner.log_audit(entry).map_err(map_err)
        })
    }

    fn query_audit(&self, filter_json: &str) -> PyResult<String> {
        catch_panic(|| {
            let filter: AuditFilter = from_str(filter_json)
                .map_err(|e| PyErr::new::<PyValueError, _>(format!("invalid filter JSON: {e}")))?;
            let entries = self.inner.query_audit(&filter).map_err(map_err)?;
            serde_json::to_string(&entries).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    // -------------------------------------------------------------------
    // Maintenance
    // -------------------------------------------------------------------

    fn flush(&self) -> PyResult<()> {
        catch_panic(|| self.inner.flush().map_err(map_err))
    }

    fn checkpoint(&self) -> PyResult<u64> {
        catch_panic(|| self.inner.checkpoint().map_err(map_err))
    }

    fn storage_size(&self) -> PyResult<String> {
        catch_panic(|| {
            let size = self.inner.storage_size().map_err(map_err)?;
            serde_json::to_string(&size).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn cache_telemetry(&self) -> PyResult<String> {
        catch_panic(|| {
            let tel = self.inner.cache_telemetry();
            serde_json::to_string(&tel).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
        })
    }

    fn clear_cache(&self) -> PyResult<()> {
        catch_panic(|| {
            self.inner.clear_cache();
            Ok(())
        })
    }

    fn clear_cache_type(&self, entity_type: &str) -> PyResult<()> {
        catch_panic(|| {
            self.inner.clear_cache_type(entity_type);
            Ok(())
        })
    }

    // -------------------------------------------------------------------
    // Generic raw storage
    // -------------------------------------------------------------------

    fn store(&self, cf_name: &str, key: &str, value: &str) -> PyResult<()> {
        catch_panic(|| self.inner.store(cf_name, key, value).map_err(map_err))
    }

    fn get(&self, cf_name: &str, key: &str) -> PyResult<Option<String>> {
        catch_panic(|| {
            self.inner.get(cf_name, key).map_err(map_err)
        })
    }

    // -------------------------------------------------------------------
    // Status / health
    // -------------------------------------------------------------------

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
                    "hitRatio": if tel.total_ops > 0 { tel.hits as f64 / tel.total_ops as f64 } else { 0.0 },
                    "entriesByType": tel.entries_by_type,
                },
            });
            serde_json::to_string(&health).map_err(|e: serde_json::Error| {
                PyErr::new::<PyRuntimeError, _>(e.to_string())
            })
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

    fn setup() -> (PyEngine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine = PyEngine::open(dir.path().to_str().unwrap()).expect("open");
        (engine, dir)
    }

    // SAFETY: direct serde_json::from_str is acceptable here — this is a test
    // helper that only parses internal test data with bounded nesting.
    fn parse_json(s: &str) -> serde_json::Value {
        serde_json::from_str(s).expect("valid JSON")
    }

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
        let filter = serde_json::json!({ "project": project });
        let list_json = engine.list_sessions(&filter.to_string()).expect("list");
        let list: Vec<serde_json::Value> =
            serde_json::from_str(&list_json).expect("parse list");
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
        let fetched = engine.get_memory(id).expect("get").expect("memory exists");
        assert!(fetched.contains(id));
        let patch = serde_json::json!({ "content": "updated content" });
        let updated = engine
            .update_memory(id, &patch.to_string())
            .expect("update")
            .expect("memory exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["content"], "updated content");
        let query = serde_json::json!({ "keywords": "updated" });
        let results = engine.search_memories(&query.to_string()).expect("search");
        let results: Vec<serde_json::Value> =
            serde_json::from_str(&results).expect("parse search results");
        assert!(results.iter().any(|m| m["id"] == id));
        let filter = serde_json::json!({ "memoryType": "fact" });
        let count = engine.count_memories(&filter.to_string()).expect("count");
        assert!(count > 0);
        engine.delete_memory(id).expect("delete");
        assert!(engine.get_memory(id).expect("get after delete").is_none());
    }

    #[test]
    fn test_py_create_memory_bytes() {
        let (engine, _dir) = setup();
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
    }

    // -----------------------------------------------------------------------
    // Agent / Skill CRUD
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_agent_skill() {
        let (engine, _dir) = setup();
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
        let fetched = engine
            .get_agent(&agent_id)
            .expect("get")
            .expect("agent exists");
        assert!(fetched.contains(&agent_id));
        let filter = serde_json::json!({});
        let list = engine.list_agents(&filter.to_string()).expect("list");
        let list: Vec<serde_json::Value> = serde_json::from_str(&list).expect("parse list");
        assert!(list.iter().any(|a| a["id"] == agent_id));
        let patch = serde_json::json!({ "name": "updated-agent" });
        let updated = engine
            .update_agent(&agent_id, &patch.to_string())
            .expect("update")
            .expect("agent exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["name"], "updated-agent");
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
        let fetched = engine
            .get_skill(&skill_id)
            .expect("get")
            .expect("skill exists");
        assert!(fetched.contains(&skill_id));
        let filter = serde_json::json!({});
        let list = engine.list_skills(&filter.to_string()).expect("list");
        let list: Vec<serde_json::Value> = serde_json::from_str(&list).expect("parse list");
        assert!(list.iter().any(|s| s["id"] == skill_id));
        let patch = serde_json::json!({ "name": "super-review" });
        let updated = engine
            .update_skill(&skill_id, &patch.to_string())
            .expect("update")
            .expect("skill exists");
        let updated = parse_json(&updated);
        assert_eq!(updated["name"], "super-review");
        engine.delete_agent(&agent_id).expect("delete agent");
        assert!(engine.get_agent(&agent_id).expect("get after delete").is_none());
        engine.delete_skill(&skill_id).expect("delete skill");
        assert!(engine.get_skill(&skill_id).expect("get after delete").is_none());
    }

    // -----------------------------------------------------------------------
    // Settings
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_settings() {
        let (engine, _dir) = setup();
        engine.set_setting("theme", "dark").expect("set setting");
        engine.set_setting("language", "en-US").expect("set setting");
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
    // Maintenance
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_maintenance() {
        let (engine, _dir) = setup();
        engine.flush().expect("flush");
        let seq = engine.checkpoint().expect("checkpoint");
        assert!(seq > 0, "checkpoint seq should be > 0");
        let size_json = engine.storage_size().expect("storage size");
        let size = parse_json(&size_json);
        assert!(size["total"].is_number());
        let tel_json = engine.cache_telemetry().expect("cache telemetry");
        let tel = parse_json(&tel_json);
        assert_eq!(tel["totalOps"], 0);
    }

    #[test]
    fn test_py_clear_cache() {
        let (engine, _dir) = setup();
        let json = engine
            .create_session(&new_session_json("cache-clear"))
            .expect("create");
        let session = parse_json(&json);
        let id = session["id"].as_str().unwrap().to_string();
        engine.get_session(&id).expect("get").expect("exists");
        engine.clear_cache_type("session").expect("clear cache type");
        engine.clear_cache().expect("clear cache");
    }

    // -----------------------------------------------------------------------
    // Error handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_py_invalid_json_returns_error() {
        let (engine, _dir) = setup();
        let result = engine.create_session("not valid json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JSON"));
    }

    #[test]
    fn test_py_invalid_uuid_returns_error() {
        let (engine, _dir) = setup();
        let result = engine.get_session("not-a-uuid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid session id"));
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
