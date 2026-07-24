//! Integration tests for column family verification via the Engine API.
//!
//! Verifies that:
//! - `storage_size()` reports the expected number of column families.
//! - Engine-level settings roundtrip correctly through the storage layer.

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Column family existence
// ---------------------------------------------------------------------------

/// Verify that `storage_size()` reports at least 12 column families.
///
/// The `ColumnFamilyMap` defines 12 canonical CFs (see
/// `contexter_core::storage::column_families`). This test ensures the engine
/// reports them all on a fresh database.
#[test]
fn test_column_families_exist() {
    let (engine, _dir) = common::setup_engine();
    let size = engine.storage_size().expect("storage size");

    // Verify at minimum we have CF count info
    assert!(
        size.per_cf.len() >= 12,
        "expected at least 12 CFs, got {}",
        size.per_cf.len()
    );
}

// ---------------------------------------------------------------------------
// Storage roundtrip
// ---------------------------------------------------------------------------

/// Verify a full setting roundtrip through the Engine API.
#[test]
fn test_storage_roundtrip() {
    let (engine, _dir) = common::setup_engine();

    // Store and retrieve data via Engine API
    engine.set_setting("test", "value").expect("set setting");

    let value = engine
        .get_setting("test")
        .expect("get setting");
    assert_eq!(value, Some("value".into()));
}
