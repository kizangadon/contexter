//! Agent CRUD operations on [`Engine`].

use uuid::Uuid;

use super::{agent_cache_key, Engine};
use crate::cache::CachedValue;
use crate::error::{EngineError, EngineResult};
use crate::models::*;
use crate::storage::column_families::{CF_AGENTS, KEY_PREFIX_AGENT};

use crate::engine::BATCH_SIZE;

impl Engine {
    /// Register a new agent.
    ///
    /// **Policy:** Write-through.
    pub fn create_agent(&self, new_agent: NewAgent) -> EngineResult<Agent> {
        let agent = self.storage.write().unwrap_or_else(|e| e.into_inner()).create_agent(new_agent)?;
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
        match self.storage.read().unwrap_or_else(|e| e.into_inner()).get_agent(id)? {
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
            .unwrap_or_else(|e| e.into_inner())
            .scan_cf_keys(CF_AGENTS, KEY_PREFIX_AGENT)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap_or_else(|e| e.into_inner());
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
        let agent = self.storage.write().unwrap_or_else(|e| e.into_inner()).update_agent(id, patch)?;
        let key = agent_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(agent)
    }

    /// Permanently delete an agent.
    ///
    /// **Policy:** Invalidate.
    pub fn delete_agent(&self, id: Uuid) -> EngineResult<()> {
        self.storage.write().unwrap_or_else(|e| e.into_inner()).delete_agent(id)?;
        let key = agent_cache_key(&id);
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

    /// Verify agent create + get roundtrip.
    #[test]
    fn test_agent_create_and_get() {
        let (engine, _dir) = setup();
        let agent = engine
            .create_agent(NewAgent {
                name: "test-agent".into(),
                agent_type: "chat".into(),
                description: "Test agent".into(),
                status: Some(AgentStatus::Active),
                capabilities: Some(vec![]),
                config: Some(serde_json::Value::Object(Default::default())),
            })
            .expect("create agent");
        assert_eq!(agent.name, "test-agent");

        let fetched = engine
            .get_agent(agent.id)
            .expect("get agent")
            .expect("agent exists");
        assert_eq!(fetched.id, agent.id);
        assert_eq!(fetched.name, "test-agent");
    }

    /// Verify that getting a non-existent agent returns None.
    #[test]
    fn test_get_nonexistent_agent() {
        let (engine, _dir) = setup();
        let result = engine.get_agent(Uuid::now_v7()).expect("get agent");
        assert!(result.is_none());
    }
}
