use contexter_core::*;
use std::collections::HashMap;

#[path = "../common/mod.rs"]
mod common;

#[test]
fn test_engine_open_creates_directories() {
    use tempfile::TempDir;
    let dir = TempDir::new().expect("temp dir");
    let db_path = dir.path().join("contexter.db");
    let engine = Engine::open(&db_path).expect("open engine");
    assert!(db_path.exists(), "RocksDB path should exist after open");
    let tel = engine.cache_telemetry();
    assert_eq!(tel.total_ops, 0, "fresh engine should have zero cache ops");
    let result = engine
        .count_sessions(&SessionFilter::default())
        .expect("count sessions");
    assert_eq!(result, 0);
}

#[test]
fn test_engine_with_config_applies_cache_settings() {
    use tempfile::TempDir;
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
