//! Integration tests for settings CRUD — persistence, validation, cache
//! behaviour, and audit logging.

use contexter_core::{AuditFilter, NewAuditEntry};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Settings persistence
// ---------------------------------------------------------------------------

#[test]
fn test_settings_persist() {
    let (engine, _dir) = common::setup_engine();

    engine.set_setting("theme", "dark").expect("set setting");
    engine
        .set_setting("language", "en-US")
        .expect("set setting");

    assert_eq!(
        engine.get_setting("theme").expect("get theme"),
        Some("dark".into())
    );
    assert_eq!(
        engine.get_setting("language").expect("get language"),
        Some("en-US".into())
    );
    assert_eq!(
        engine.get_setting("nonexistent").expect("get missing"),
        None
    );
}

// ---------------------------------------------------------------------------
// Setting validation
// ---------------------------------------------------------------------------

#[test]
fn test_setting_empty_key_rejected() {
    let (engine, _dir) = common::setup_engine();
    let result = engine.set_setting("", "value");
    assert!(result.is_err(), "empty key should be rejected");
}

#[test]
fn test_setting_key_too_long_rejected() {
    let (engine, _dir) = common::setup_engine();
    let long_key = "a".repeat(257);
    let result = engine.set_setting(&long_key, "value");
    assert!(result.is_err(), "overlong key should be rejected");
}

#[test]
fn test_setting_valid_key_accepted() {
    let (engine, _dir) = common::setup_engine();
    engine
        .set_setting("valid-key", "value")
        .expect("valid key should succeed");
    assert_eq!(
        engine.get_setting("valid-key").expect("get"),
        Some("value".into())
    );
}

#[test]
fn test_setting_cache_aside() {
    let (engine, _dir) = common::setup_engine();

    // Write-through stores in cache.
    engine.set_setting("test-key", "test-value").expect("set");

    // After write-through, get should be a L1 hit.
    let tel_before = engine.cache_telemetry();
    let val = engine.get_setting("test-key").expect("get");
    let tel_after = engine.cache_telemetry();
    assert_eq!(val, Some("test-value".into()));
    assert_eq!(tel_after.hits - tel_before.hits, 1);
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

#[test]
fn test_audit_logging() {
    let (engine, _dir) = common::setup_engine();

    engine
        .log_audit(NewAuditEntry {
            action: "create_session".into(),
            entity_type: "Session".into(),
            entity_id: "abc-123".into(),
            actor: Some("user-1".into()),
            summary: Some(serde_json::json!({"status": "active"})),
        })
        .expect("log audit");

    engine
        .log_audit(NewAuditEntry {
            action: "create_memory".into(),
            entity_type: "Memory".into(),
            entity_id: "def-456".into(),
            actor: Some("user-1".into()),
            summary: None,
        })
        .expect("log audit");

    // Query all.
    let all = engine
        .query_audit(&AuditFilter::default())
        .expect("query audit");
    assert_eq!(all.len(), 2, "should have 2 audit entries");

    // Filter by entity type.
    let sessions = engine
        .query_audit(&AuditFilter {
            entity_type: Some("Session".into()),
            ..AuditFilter::default()
        })
        .expect("filter by type");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].action, "create_session");

    // Filter by actor.
    let actor_entries = engine
        .query_audit(&AuditFilter {
            actor: Some("user-1".into()),
            ..AuditFilter::default()
        })
        .expect("filter by actor");
    assert_eq!(actor_entries.len(), 2);
}
// ---------------------------------------------------------------------------
// Key length bounds
// ---------------------------------------------------------------------------

#[test]
fn test_setting_key_256_chars_succeeds() {
    let (engine, _dir) = common::setup_engine();
    let key = "a".repeat(256);
    engine
        .set_setting(&key, "value")
        .expect("256-char key should succeed");
    assert_eq!(engine.get_setting(&key).expect("get"), Some("value".into()));
}
