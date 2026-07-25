//! Integration tests for the PyO3 bridge.
//!
//! The bridge module is only available when the `python` feature is enabled.
//! These tests verify that the bridge compiles and exposes the expected API.
//!
//! Note: The bridge's full test suite lives inline in `src/bridge.rs` under
//! `#[cfg(test)]` — those tests cover the PyEngine API directly. This file
//! provides feature-gated integration coverage for the compiled library.

// ---------------------------------------------------------------------------
// Compile-time trait bound verification
// ---------------------------------------------------------------------------

/// Ensure the bridge module exists and PyEngine implements Send + Sync.
#[cfg(feature = "python")]
#[test]
fn test_bridge_module_available() {
    use contexter_core::bridge::PyEngine;

    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<PyEngine>();
    assert_sync::<PyEngine>();
}

/// Verify that PyEngine can be constructed and queried for status.
#[cfg(feature = "python")]
#[test]
fn test_py_engine_open_and_status() {
    use contexter_core::bridge::PyEngine;
    use tempfile::TempDir;

    let dir = TempDir::new().expect("temp dir");
    let engine =
        PyEngine::open(dir.path().to_str().unwrap()).expect("PyEngine::open should succeed");
    let status_json = engine.status().expect("status");
    let status: serde_json::Value =
        serde_json::from_str(&status_json).expect("valid JSON status");
    assert_eq!(status["status"], "ok");
    assert!(status["version"].is_string());
}

/// Verify PyEngine session create/get round-trip via JSON.
#[cfg(feature = "python")]
#[test]
fn test_py_session_create_get() {
    use contexter_core::bridge::PyEngine;
    use tempfile::TempDir;
    use uuid::Uuid;

    let dir = TempDir::new().expect("temp dir");
    let engine =
        PyEngine::open(dir.path().to_str().unwrap()).expect("PyEngine::open should succeed");

    let session_json = serde_json::json!({
        "project": "py-bridge-test",
        "agentId": Uuid::now_v7().to_string(),
    });

    let created = engine
        .create_session(&session_json.to_string())
        .expect("create session");
    let created_val: serde_json::Value =
        serde_json::from_str(&created).expect("parse created session");
    assert_eq!(created_val["project"], "py-bridge-test");
    assert_eq!(created_val["turnCount"], 0);

    let id = created_val["id"].as_str().unwrap();
    let fetched = engine
        .get_session(id)
        .expect("get session")
        .expect("session exists");
    let fetched_val: serde_json::Value =
        serde_json::from_str(&fetched).expect("parse fetched session");
    assert_eq!(fetched_val["id"], id);
}
