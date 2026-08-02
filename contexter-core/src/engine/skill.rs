//! Skill CRUD operations on [`Engine`].

use uuid::Uuid;

use super::{skill_cache_key, Engine};
use crate::cache::CachedValue;
use crate::error::{EngineError, EngineResult};
use crate::models::*;
use crate::storage::column_families::{CF_SKILLS, KEY_PREFIX_SKILL};

use crate::engine::BATCH_SIZE;

impl Engine {
    /// Validate a skill's `file_path` (if present).
    ///
    /// Rejects empty paths, paths containing `..` path segments (path traversal),
    /// and paths exceeding 4096 bytes to prevent storage abuse and constrain
    /// downstream path handling.
    pub(crate) fn validate_file_path(file_path: &Option<String>) -> EngineResult<()> {
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
        let skill = self.storage.write().unwrap_or_else(|e| e.into_inner()).create_skill(new_skill)?;
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
        match self.storage.read().unwrap_or_else(|e| e.into_inner()).get_skill(id)? {
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
            .unwrap_or_else(|e| e.into_inner())
            .scan_cf_keys(CF_SKILLS, KEY_PREFIX_SKILL)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap_or_else(|e| e.into_inner());
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

    /// Count skills matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_skills(&self, filter: &SkillFilter) -> EngineResult<u64> {
        self.storage
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .count_skills(filter)
    }

    /// Partially update an existing skill.
    ///
    /// **Policy:** Write-around.
    pub fn update_skill(&self, id: Uuid, patch: &SkillPatch) -> EngineResult<Skill> {
        Self::validate_file_path(&patch.file_path)?;
        let skill = self.storage.write().unwrap_or_else(|e| e.into_inner()).update_skill(id, patch)?;
        let key = skill_cache_key(&id);
        self.cache.invalidate(&key);
        Ok(skill)
    }

    /// Permanently delete a skill.
    ///
    /// **Policy:** Invalidate.
    pub fn delete_skill(&self, id: Uuid) -> EngineResult<()> {
        self.storage.write().unwrap_or_else(|e| e.into_inner()).delete_skill(id)?;
        let key = skill_cache_key(&id);
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

    // -----------------------------------------------------------------------
    // validate_file_path unit tests (pure function, no Engine needed)
    // -----------------------------------------------------------------------

    #[test]
    fn test_validate_file_path_none_accepted() {
        assert!(Engine::validate_file_path(&None).is_ok());
    }

    #[test]
    fn test_validate_file_path_empty_rejected() {
        let result = Engine::validate_file_path(&Some(String::new()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("file_path"));
    }

    #[test]
    fn test_validate_file_path_traversal_rejected() {
        let result = Engine::validate_file_path(&Some("../etc/passwd".into()));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("traversal"));
    }

    #[test]
    fn test_validate_file_path_too_long_rejected() {
        let result = Engine::validate_file_path(&Some("a".repeat(4097)));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("4096"));
    }

    #[test]
    fn test_validate_file_path_valid_accepted() {
        assert!(Engine::validate_file_path(&Some("/valid/path.py".into())).is_ok());
    }

    // -----------------------------------------------------------------------
    // Integration: skill create + get
    // -----------------------------------------------------------------------

    #[test]
    fn test_skill_create_and_get() {
        let (engine, _dir) = setup();
        let skill = engine
            .create_skill(NewSkill {
                name: "test-skill".into(),
                description: "a test".into(),
                category: "code".into(),
                file_path: None,
            })
            .expect("create skill");
        assert_eq!(skill.name, "test-skill");

        let fetched = engine
            .get_skill(skill.id)
            .expect("get skill")
            .expect("skill exists");
        assert_eq!(fetched.id, skill.id);
    }
}
