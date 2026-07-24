//! Settings and audit-log operations on [`Engine`].

use super::{setting_cache_key, Engine};
use crate::cache::CachedValue;
use crate::error::{EngineError, EngineResult};
use crate::models::*;
use crate::storage::column_families::{CF_AUDIT, KEY_PREFIX_AUDIT};

use crate::engine::BATCH_SIZE;

impl Engine {
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
            .scan_cf_keys(CF_AUDIT, KEY_PREFIX_AUDIT)?;

        let mut results = Vec::new();

        for chunk in keys.chunks(BATCH_SIZE) {
            let storage = self.storage.read().unwrap();
            for key_bytes in chunk {
                let key_str = std::str::from_utf8(key_bytes).map_err(|e| {
                    EngineError::Internal(format!("invalid UTF-8 key: {e}"))
                })?;

                let value = match storage.get_raw(CF_AUDIT, key_str)? {
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

    /// Verify setting persistence roundtrip.
    #[test]
    fn test_setting_set_and_get() {
        let (engine, _dir) = setup();
        engine.set_setting("theme", "dark").expect("set setting");
        let value = engine
            .get_setting("theme")
            .expect("get setting")
            .expect("setting exists");
        assert_eq!(value, "dark");
    }

    /// Verify that getting a non-existent setting returns None.
    #[test]
    fn test_get_nonexistent_setting() {
        let (engine, _dir) = setup();
        let result = engine
            .get_setting("nonexistent")
            .expect("get setting");
        assert!(result.is_none());
    }

    /// Verify that empty key is rejected on set.
    #[test]
    fn test_setting_empty_key_rejected() {
        let (engine, _dir) = setup();
        let result = engine.set_setting("", "value");
        assert!(result.is_err());
    }
}
