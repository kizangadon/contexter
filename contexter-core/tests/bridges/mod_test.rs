//! Integration tests for the bridge module's JSON boundary.
//!
//! The bridge module (`src/bridge.rs`) serializes and deserializes domain
//! entities as JSON strings for the Python FFI boundary. These tests verify
//! that the JSON handling works correctly at the integration level via the
//! Engine API (which the bridge delegates to internally).
//!
//! When the `python` feature is enabled, additional tests exercise
//! `PyEngine` directly.

use contexter_core::*;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// JSON round-trip tests (Engine API — always available)
// ---------------------------------------------------------------------------

/// Verify that a Session round-trips through JSON serialization correctly.
#[test]
fn test_session_json_roundtrip() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    let session = engine
        .create_session(NewSession {
            project: "json-roundtrip".into(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: Some(serde_json::json!({"env": "test", "version": 1})),
        })
        .expect("create session");

    // Verify all fields survived serialization.
    assert_eq!(session.project, "json-roundtrip");
    assert_eq!(session.agent_id, agent_id);
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.turn_count, 0);
    assert_eq!(session.metadata["env"], "test");
    assert_eq!(session.metadata["version"], 1);

    // Round-trip via get.
    let fetched = engine
        .get_session(session.id)
        .expect("get session")
        .expect("session exists");
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.project, session.project);
    assert_eq!(fetched.agent_id, session.agent_id);
}

/// Verify that a Memory with non-ASCII content round-trips correctly.
#[test]
fn test_memory_json_special_chars() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id,
            memory_type: MemoryType::Fact,
            content: "こんにちは世界 •emoji: 🚀 ∞ ∑ ∫".into(),
            tags: Some(vec!["unicode".into(), "emoji".into()]),
        })
        .expect("create memory with special chars");

    let fetched = engine
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory exists");
    assert_eq!(fetched.content, "こんにちは世界 •emoji: 🚀 ∞ ∑ ∫");
    assert!(fetched.tags.contains(&"unicode".into()));
    assert!(fetched.tags.contains(&"emoji".into()));
}

/// Verify that null metadata is handled correctly (converted to empty object).
#[test]
fn test_session_null_metadata() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    let session = engine
        .create_session(NewSession {
            project: "null-meta".into(),
            agent_id,
            status: None,
            metadata: None,
        })
        .expect("create session with null metadata");

    // Metadata should default to empty object, not null.
    assert!(session.metadata.is_object());
    assert_eq!(session.metadata.as_object().map(|m| m.len()), Some(0));
}

/// Verify that deeply nested JSON in metadata round-trips correctly.
#[test]
fn test_deeply_nested_metadata() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Build a deeply nested JSON value (depth ~20).
    let mut nested = serde_json::json!({"level": 0});
    for i in 1..20 {
        nested = serde_json::json!({"level": i, "child": nested});
    }

    let session = engine
        .create_session(NewSession {
            project: "deep-nest".into(),
            agent_id,
            status: None,
            metadata: Some(nested.clone()),
        })
        .expect("create session with deeply nested metadata");

    // Verify the nesting is preserved.
    let mut current = &session.metadata;
    for i in (0..20).rev() {
        assert_eq!(current["level"], i);
        if i > 0 {
            current = &current["child"];
        }
    }
}

/// Verify that empty string content is valid.
#[test]
fn test_memory_empty_content() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id,
            memory_type: MemoryType::Fact,
            content: String::new(),
            tags: None,
        })
        .expect("create memory with empty content");

    assert!(memory.content.is_empty());
    let fetched = engine
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory exists");
    assert!(fetched.content.is_empty());
}

/// Verify that engine rejects invalid data via validation
/// (e.g., session with empty project name is stored as-is, no validation).
/// This is a boundary test: empty project is stored but retrievable.
#[test]
fn test_session_empty_project() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    let session = engine
        .create_session(NewSession {
            project: String::new(),
            agent_id,
            status: None,
            metadata: None,
        })
        .expect("create session with empty project");
    assert!(session.project.is_empty());
}

// ---------------------------------------------------------------------------
// Feature-gated PyEngine tests (requires `python` feature)
// ---------------------------------------------------------------------------

#[cfg(feature = "python")]
mod python_bridge {
    use contexter_core::bridge::PyEngine;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn setup() -> (PyEngine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine =
            PyEngine::open(dir.path().to_str().unwrap()).expect("PyEngine::open");
        (engine, dir)
    }

    /// from_str equivalent: valid JSON → correct type.
    #[test]
    fn test_py_from_str_valid_json() {
        let (engine, _dir) = setup();
        let agent_id = Uuid::now_v7();
        let session_json = serde_json::json!({
            "project": "py-bridge-test",
            "agentId": agent_id.to_string(),
        });

        let created = engine
            .create_session(&session_json.to_string())
            .expect("create session via bridge");
        let created_val: serde_json::Value =
            serde_json::from_str(&created).expect("valid JSON response");
        assert_eq!(created_val["project"], "py-bridge-test");
        assert_eq!(created_val["turnCount"], 0);
    }

    /// from_str equivalent: invalid JSON → error.
    #[test]
    fn test_py_from_str_invalid_json() {
        let (engine, _dir) = setup();
        let result = engine.create_session("not valid json at all");
        assert!(result.is_err(), "invalid JSON should produce error");
        let err = result.unwrap_err().to_string();
        // Should mention JSON or parse error.
        assert!(
            err.contains("JSON") || err.contains("parse") || err.contains("invalid"),
            "error should reference invalid input: {err}"
        );
    }

    /// to_json equivalent: output is valid JSON string.
    #[test]
    fn test_py_to_json_valid_string() {
        let (engine, _dir) = setup();
        let status = engine.status().expect("status");
        let parsed: serde_json::Value =
            serde_json::from_str(&status).expect("status should be valid JSON");
        assert_eq!(parsed["status"], "ok");
    }

    /// Edge case: empty string input returns error.
    #[test]
    fn test_py_empty_string_input() {
        let (engine, _dir) = setup();
        let result = engine.create_session("");
        assert!(
            result.is_err(),
            "empty string should produce an error"
        );
    }

    /// Edge case: null/empty metadata in session JSON.
    #[test]
    fn test_py_null_metadata() {
        let (engine, _dir) = setup();
        let agent_id = Uuid::now_v7();
        let session_json = serde_json::json!({
            "project": "null-meta",
            "agentId": agent_id.to_string(),
        });

        let created = engine
            .create_session(&session_json.to_string())
            .expect("create session");
        let val: serde_json::Value =
            serde_json::from_str(&created).expect("valid JSON");
        assert_eq!(val["project"], "null-meta");
    }

    /// Edge case: deeply nested JSON in session metadata.
    #[test]
    fn test_py_deeply_nested_json() {
        let (engine, _dir) = setup();
        let agent_id = Uuid::now_v7();

        // Build depth ~40 — well within the 64 limit.
        let mut nested = serde_json::json!({"level": 0});
        for i in 1..40 {
            nested = serde_json::json!({"level": i, "child": nested});
        }

        let session_json = serde_json::json!({
            "project": "deep-nest-py",
            "agentId": agent_id.to_string(),
            "metadata": nested,
        });

        let created = engine
            .create_session(&session_json.to_string())
            .expect("create session with deep nesting");
        let val: serde_json::Value =
            serde_json::from_str(&created).expect("valid JSON");
        assert_eq!(val["project"], "deep-nest-py");
    }
}
