//! Session CRUD operations on [`Engine`].

use uuid::Uuid;

use super::{session_cache_key, Engine};
use crate::cache::CachedValue;
use crate::error::{EngineError, EngineResult};
use crate::models::*;
use crate::storage::column_families::{CF_SESSIONS, KEY_PREFIX_SESSION};

use std::sync::atomic::Ordering;

use crate::engine::BATCH_SIZE;

impl Engine {
    /// Create a new session.
    ///
    /// **Policy:** Write-through — persisted first, then cached.
    pub fn create_session(&self, new_session: NewSession) -> EngineResult<Session> {
        self.telemetry.stats.sessions_created.fetch_add(1, Ordering::Relaxed);
        let session = self.storage.write().unwrap_or_else(|e| e.into_inner()).create_session(new_session)?;
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
        match self.storage.read().unwrap_or_else(|e| e.into_inner()).get_session(id)? {
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
            .unwrap_or_else(|e| e.into_inner())
            .scan_cf_keys(CF_SESSIONS, KEY_PREFIX_SESSION)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap_or_else(|e| e.into_inner());
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
        let session = self.storage.write().unwrap_or_else(|e| e.into_inner()).update_session(id, patch)?;
        let key = session_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(session)
    }

    /// Permanently delete a session.
    ///
    /// **Policy:** Invalidate — deleted from L2, then evicted from L1.
    pub fn delete_session(&self, id: Uuid) -> EngineResult<()> {
        self.telemetry.stats.sessions_deleted.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap_or_else(|e| e.into_inner()).delete_session(id)?;
        let key = session_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
    }

    /// Count sessions matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_sessions(&self, filter: &SessionFilter) -> EngineResult<u64> {
        self.storage.read().unwrap_or_else(|e| e.into_inner()).count_sessions(filter)
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

    /// Verify session create + get roundtrip.
    #[test]
    fn test_session_create_and_get() {
        let (engine, _dir) = setup();
        let session = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: Uuid::now_v7(),
                status: None,
                metadata: None,
            })
            .expect("create session");
        assert_eq!(session.project, "test");

        let fetched = engine
            .get_session(session.id)
            .expect("get session")
            .expect("session exists");
        assert_eq!(fetched.id, session.id);
        assert_eq!(fetched.status, SessionStatus::Active);
    }

    /// Verify that getting a non-existent session returns None.
    #[test]
    fn test_get_nonexistent_session() {
        let (engine, _dir) = setup();
        let result = engine.get_session(Uuid::now_v7()).expect("get session");
        assert!(result.is_none());
    }
}
