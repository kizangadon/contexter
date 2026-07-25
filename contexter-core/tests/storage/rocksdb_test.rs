//! Integration tests for RocksDB storage — persistence, read-only error, and
//! generic store/get round-trips.

use std::collections::HashMap;

use contexter_core::{AgentFilter, Engine, MemoryFilter, MemoryPatch, NewAgent, NewMemory};
use tempfile::TempDir;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// 1. Storage persistence — engine restart retains data
// ---------------------------------------------------------------------------

#[test]
fn test_storage_persistence() {
    let dir = TempDir::new().expect("temp dir");
    let agent_id = Uuid::now_v7();

    // Engine 1: create entities.
    let engine1 = Engine::open(dir.path()).expect("open engine1");
    let session = common::create_session(&engine1, "persistence-test", agent_id);
    let memory = engine1
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id,
            memory_type: contexter_core::MemoryType::Fact,
            content: "persistent data".to_string(),
            tags: None,
        })
        .expect("create memory");
    let agent = engine1
        .create_agent(NewAgent {
            name: "persist-agent".into(),
            agent_type: "test".into(),
            description: "Persistence test agent".into(),
            capabilities: None,
            status: None,
            config: None,
        })
        .expect("create agent");

    // Flush to ensure all writes are durable.
    engine1.flush().expect("flush");

    // Drop engine1 (dir stays alive).
    drop(engine1);

    // Engine 2: open the same path.
    let engine2 = Engine::open(dir.path()).expect("open engine2");

    // Verify all entities still exist.
    let fetched_session = engine2
        .get_session(session.id)
        .expect("get session")
        .expect("session should persist");
    assert_eq!(fetched_session.id, session.id);
    assert_eq!(fetched_session.project, "persistence-test");

    let fetched_memory = engine2
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory should persist");
    assert_eq!(fetched_memory.content, "persistent data");

    let fetched_agent = engine2
        .get_agent(agent.id)
        .expect("get agent")
        .expect("agent should persist");
    assert_eq!(fetched_agent.name, "persist-agent");

    // Delete one entity.
    engine2.delete_session(session.id).expect("delete session");

    // Verify remaining entities are intact.
    assert!(engine2
        .get_session(session.id)
        .expect("get deleted session")
        .is_none());

    let remaining_memory = engine2
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory should remain");
    assert_eq!(remaining_memory.id, memory.id);

    let remaining_agent = engine2
        .get_agent(agent.id)
        .expect("get agent")
        .expect("agent should remain");
    assert_eq!(remaining_agent.id, agent.id);

    // Drop engine2.
    drop(engine2);

    // Engine 3: re-open and verify the delete persisted.
    let engine3 = Engine::open(dir.path()).expect("open engine3");
    assert!(engine3
        .get_session(session.id)
        .expect("get session")
        .is_none());
    assert!(engine3.get_memory(memory.id).expect("get memory").is_some());
    assert!(engine3.get_agent(agent.id).expect("get agent").is_some());
}

// ---------------------------------------------------------------------------
// 2. Writable path succeeds
// ---------------------------------------------------------------------------

#[test]
fn test_writable_path_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    let dir = TempDir::new()?;
    let sub_path = dir.path().join("sub");
    std::fs::create_dir(&sub_path)?;

    let result = Engine::open(sub_path.to_str().unwrap());
    assert!(result.is_ok(), "Engine::open should succeed on a writable path");

    Ok(())
}

// ---------------------------------------------------------------------------
// 3. Storage directory has 0o700 permissions (Unix)
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn test_engine_dir_has_0700_permissions() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;
    let dir = TempDir::new()?;
    let engine = Engine::open(dir.path())?;
    drop(engine); // flush + close engine

    let meta = std::fs::metadata(dir.path())?;
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o700,
        "Engine storage directory must have 0o700 permissions, got {:#o}",
        mode
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// 4. Generic store/get roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_generic_store_get_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let (engine, _dir) = common::setup_engine();

    engine.store("sessions", "cfg:test_key", "test_value")?;
    let result = engine.get("sessions", "cfg:test_key")?;
    assert_eq!(result, Some("test_value".to_string()));

    // Non-existent key
    let result = engine.get("sessions", "cfg:nonexistent")?;
    assert_eq!(result, None);

    Ok(())
}
