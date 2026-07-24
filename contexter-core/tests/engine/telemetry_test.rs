use contexter_core::*;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

#[test]
fn test_engine_telemetry_accessible() {
    let (engine, _dir) = common::setup_engine();
    let snapshot = engine.stats();
    assert_eq!(snapshot["sessions_created"], 0);
    assert_eq!(snapshot["sessions_deleted"], 0);
    assert_eq!(snapshot["memories_created"], 0);
    assert_eq!(snapshot["memories_deleted"], 0);
    assert_eq!(snapshot["searches_completed"], 0);
}

#[test]
fn test_engine_telemetry_counters_increment() {
    let (engine, _dir) = common::setup_engine();
    let session = engine
        .create_session(NewSession {
            project: "telemetry".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create session");
    assert_eq!(engine.stats()["sessions_created"], 1);
    engine.delete_session(session.id).expect("delete session");
    assert_eq!(engine.stats()["sessions_deleted"], 1);
}

#[test]
fn test_engine_stats_method_works() {
    let (engine, _dir) = common::setup_engine();
    let snapshot = engine.stats();
    assert_eq!(snapshot["sessions_created"], 0);
    assert_eq!(snapshot["sessions_deleted"], 0);
}
