//! Integration tests for the storage module.
//!
//! Tests column family constants, key prefix definitions, configuration
//! types, raw storage roundtrips, and trait-object safety at the
//! integration level. RocksDB-specific backend tests remain in
//! `rocksdb_test.rs`.

use contexter_core::storage;
use contexter_core::storage::column_families::*;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Column families — constant values
// ---------------------------------------------------------------------------

/// Verify all CF constant names are non-empty.
#[test]
fn test_cf_constants_non_empty() {
    assert!(!CF_MEMORY_ITEMS.is_empty());
    assert!(!CF_SESSIONS.is_empty());
    assert!(!CF_AGENTS.is_empty());
    assert!(!CF_SKILLS.is_empty());
    assert!(!CF_EFFICIENCY_MAP.is_empty());
    assert!(!CF_TELEMETRY.is_empty());
    assert!(!CF_CONFLICTS.is_empty());
    assert!(!CF_INDEX_STATE.is_empty());
    assert!(!CF_MEMORY_INDEX.is_empty());
}

/// Verify each CF constant is unique.
#[test]
fn test_cf_constants_unique() {
    let cfs = vec![
        CF_MEMORY_ITEMS,
        CF_SESSIONS,
        CF_AGENTS,
        CF_SKILLS,
        CF_EFFICIENCY_MAP,
        CF_TELEMETRY,
        CF_CONFLICTS,
        CF_INDEX_STATE,
        CF_MEMORY_INDEX,
    ];
    let mut seen = std::collections::HashSet::new();
    for cf in &cfs {
        assert!(seen.insert(cf), "duplicate CF name: {cf}");
    }
}

// ---------------------------------------------------------------------------
// Key prefix constants
// ---------------------------------------------------------------------------

/// Verify key prefix constants are non-empty.
#[test]
fn test_key_prefixes_non_empty() {
    assert!(!KEY_PREFIX_SESSION.is_empty());
    assert!(!KEY_PREFIX_MEMORY.is_empty());
    assert!(!KEY_PREFIX_AGENT.is_empty());
    assert!(!KEY_PREFIX_SKILL.is_empty());
    assert!(!KEY_PREFIX_SETTING.is_empty());
    assert!(!KEY_PREFIX_AUDIT.is_empty());
}

/// Verify all key prefixes are unique.
#[test]
fn test_key_prefixes_unique() {
    let prefixes = vec![
        KEY_PREFIX_SESSION,
        KEY_PREFIX_MEMORY,
        KEY_PREFIX_AGENT,
        KEY_PREFIX_SKILL,
        KEY_PREFIX_SETTING,
        KEY_PREFIX_AUDIT,
    ];
    let mut seen = std::collections::HashSet::new();
    for p in &prefixes {
        assert!(seen.insert(p), "duplicate prefix: {p}");
    }
}

// ---------------------------------------------------------------------------
// ColumnFamilyMap
// ---------------------------------------------------------------------------

/// Verify that ColumnFamilyMap::new() contains all expected CF names.
#[test]
fn test_column_family_map_via_iter() {
    let cf_map = ColumnFamilyMap::new();
    let names: Vec<&str> = cf_map.iter().collect();

    assert!(names.contains(&CF_MEMORY_ITEMS));
    assert!(names.contains(&CF_SESSIONS));
    assert!(names.contains(&CF_AGENTS));
    assert!(names.contains(&CF_SKILLS));
    assert!(names.contains(&CF_EFFICIENCY_MAP));
    assert!(names.contains(&CF_TELEMETRY));
    assert!(names.contains(&CF_CONFLICTS));
    assert!(names.contains(&CF_INDEX_STATE));
    assert!(names.contains(&CF_MEMORY_INDEX));
    assert!(names.contains(&CF_SETTINGS));
    assert!(names.contains(&CF_AUDIT));
    assert!(names.contains(&CF_SESSION_INDEX));
    assert_eq!(names.len(), 12);
}

// ---------------------------------------------------------------------------
// RocksDbConfig
// ---------------------------------------------------------------------------

#[test]
fn test_rocksdb_config_defaults() {
    let config = storage::types::RocksDbConfig::default();
    assert_eq!(config.path, "contexter.db");
    assert!(config.create_if_missing);
    assert!(config.wal_sync);
}

#[test]
fn test_rocksdb_config_custom() {
    let config = storage::types::RocksDbConfig {
        path: "/tmp/test-rocks".into(),
        create_if_missing: true,
        wal_sync: false,
    };
    assert_eq!(config.path, "/tmp/test-rocks");
    assert!(config.create_if_missing);
    assert!(!config.wal_sync);
}

// ---------------------------------------------------------------------------
// Engine raw storage (store / get)
// ---------------------------------------------------------------------------

/// Verify raw store/get roundtrip on the engine.
#[test]
fn test_raw_store_get_roundtrip() {
    let (engine, _dir) = common::setup_engine();

    // Store under the default CF.
    engine.store("default", "test_key_1", "hello_value").expect("store");
    let got = engine.get("default", "test_key_1").expect("get");
    assert_eq!(got, Some("hello_value".to_string()));
}

/// Verify that a nonexistent key returns None (not an error).
#[test]
fn test_raw_get_missing_key() {
    let (engine, _dir) = common::setup_engine();
    let got = engine.get("default", "nonexistent_key").expect("get");
    assert!(got.is_none());
}

/// Verify that store overwrites existing values.
#[test]
fn test_raw_store_overwrite() {
    let (engine, _dir) = common::setup_engine();

    engine.store("default", "overwrite_key", "v1").expect("store v1");
    engine.store("default", "overwrite_key", "v2").expect("store v2");

    let got = engine.get("default", "overwrite_key").expect("get");
    assert_eq!(got, Some("v2".to_string()));
}

/// Verify that large values can be stored and retrieved.
#[test]
fn test_raw_store_large_value() {
    let (engine, _dir) = common::setup_engine();

    let large_value: String = (0..10_000).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
    engine.store("default", "large_key", &large_value).expect("store large");

    let got = engine.get("default", "large_key").expect("get large");
    assert_eq!(got, Some(large_value));
}

/// Verify cross-CF key isolation.
#[test]
fn test_raw_store_cf_isolation() {
    let (engine, _dir) = common::setup_engine();

    engine.store(CF_SESSIONS, "dup_key", "sessions_val").expect("store sessions");
    engine.store(CF_AGENTS, "dup_key", "agents_val").expect("store agents");

    let sessions_val = engine.get(CF_SESSIONS, "dup_key").expect("get sessions");
    let agents_val = engine.get(CF_AGENTS, "dup_key").expect("get agents");

    assert_eq!(sessions_val, Some("sessions_val".to_string()));
    assert_eq!(agents_val, Some("agents_val".to_string()));
    assert_ne!(sessions_val, agents_val);
}

// ---------------------------------------------------------------------------
// ScoredMemoryId
// ---------------------------------------------------------------------------

#[test]
fn test_scored_memory_id_construction() {
    let id = storage::types::ScoredMemoryId {
        memory_id: uuid::Uuid::now_v7(),
        score: 0.95,
    };
    assert!(!id.memory_id.is_nil());
    assert!((id.score - 0.95).abs() < 1e-10);
}

// ---------------------------------------------------------------------------
// SharedBackend
// ---------------------------------------------------------------------------

#[test]
fn test_shared_backend_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<contexter_core::storage::SharedBackend>();
    assert_sync::<contexter_core::storage::SharedBackend>();
}
