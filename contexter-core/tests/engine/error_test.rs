//! Integration tests for error paths — non-existent entity lookups and not-found
//! behaviour across all entity types.

use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Not found / error paths
// ---------------------------------------------------------------------------

#[test]
fn test_invalid_session_returns_none() {
    let (engine, _dir) = common::setup_engine();
    let id = Uuid::now_v7();
    let result = engine.get_session(id).expect("get nonexistent");
    assert!(result.is_none(), "non-existent session should return None");
}

#[test]
fn test_not_found_returns_none() {
    let (engine, _dir) = common::setup_engine();
    let random_id = Uuid::now_v7();

    assert!(engine
        .get_session(random_id)
        .expect("get session")
        .is_none());
    assert!(engine.get_memory(random_id).expect("get memory").is_none());
    assert!(engine.get_agent(random_id).expect("get agent").is_none());
    assert!(engine.get_skill(random_id).expect("get skill").is_none());
}