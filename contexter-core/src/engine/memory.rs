//! Memory CRUD operations on [`Engine`].

use uuid::Uuid;

use super::{memory_cache_key, Engine};
use crate::cache::CachedValue;
use crate::error::EngineError;
use crate::models::*;

use std::sync::atomic::Ordering;

impl Engine {
    /// Create a new memory.
    ///
    /// **Policy:** Write-through.
    pub fn create_memory(&self, new_memory: NewMemory) -> crate::error::EngineResult<Memory> {
        self.telemetry.stats.memories_created.fetch_add(1, Ordering::Relaxed);
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
    pub fn get_memory(&self, id: Uuid) -> crate::error::EngineResult<Option<Memory>> {
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

    /// Partially update an existing memory.
    ///
    /// **Policy:** Write-around.
    pub fn update_memory(
        &self,
        id: Uuid,
        patch: &MemoryPatch,
    ) -> crate::error::EngineResult<Memory> {
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
    pub fn delete_memory(&self, id: Uuid) -> crate::error::EngineResult<()> {
        self.telemetry.stats.memories_deleted.fetch_add(1, Ordering::Relaxed);
        self.storage.write().unwrap().delete_memory(id)?;
        let key = memory_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(())
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

    /// Verify memory create + get roundtrip.
    #[test]
    fn test_memory_create_and_get() {
        let (engine, _dir) = setup();
        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "test content".into(),
                tags: None,
            })
            .expect("create memory");
        assert_eq!(memory.content, "test content");

        let fetched = engine
            .get_memory(memory.id)
            .expect("get memory")
            .expect("memory exists");
        assert_eq!(fetched.id, memory.id);
    }

    /// Verify that oversized content is rejected.
    #[test]
    fn test_memory_oversized_content_rejected() {
        let (engine, _dir) = setup();
        let result = engine.create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "x".repeat(1024 * 1024 + 1),
            tags: None,
        });
        assert!(result.is_err());
    }
}
