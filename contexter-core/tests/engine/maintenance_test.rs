//! Integration tests for engine maintenance operations — flush, checkpoint,
//! storage size reporting, and cache telemetry / cache invalidation.

use contexter_core::{MemoryType, NewMemory, NewSession};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Flush and checkpoint
// ---------------------------------------------------------------------------

#[test]
fn test_flush_and_checkpoint() {
    let (engine, _dir) = common::setup_engine();

    // Write something so there's data to flush.
    engine
        .create_session(NewSession {
            project: "flush-test".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create");

    engine.flush().expect("flush");
    let seq = engine.checkpoint().expect("checkpoint");
    assert!(seq > 0, "checkpoint sequence number should be > 0");
}

// ---------------------------------------------------------------------------
// Storage size
// ---------------------------------------------------------------------------

#[test]
fn test_storage_size_non_zero() {
    let (engine, _dir) = common::setup_engine();

    // Initially the size may be small (RocksDB metadata), but the call
    // should succeed and return at least the WAL size.
    let size = engine.storage_size().expect("storage size");
    // Verify the structure is well-formed.
    assert!(!size.per_cf.is_empty() || size.wal_size > 0);

    // Write a reasonable amount of data.
    for i in 0..10 {
        engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: format!(
                    "large content item number {i} with some padding data to make the size more significant"
                ),
                tags: None,
            })
            .expect("create memory");
    }

    let _size_after = engine.storage_size().expect("storage size after writes");
    engine.flush().expect("flush");
    let size_flushed = engine.storage_size().expect("storage size after flush");
    // After flush, SST files should exist.
    assert!(
        size_flushed.total > 0 || size_flushed.wal_size > 0,
        "after flush, total or WAL size should be > 0"
    );
}

// ---------------------------------------------------------------------------
// Cache telemetry
// ---------------------------------------------------------------------------

#[test]
fn test_cache_telemetry_tracking() {
    let (engine, _dir) = common::setup_engine();

    let tel = engine.cache_telemetry();
    assert_eq!(tel.total_ops, 0);

    // Create a session (write-through stores in cache, no read ops).
    let session = engine
        .create_session(NewSession {
            project: "tel-test".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create");

    // Get session (cache hit from write-through).
    let tel_before = engine.cache_telemetry();
    let _ = engine.get_session(session.id).expect("get");
    let tel_after = engine.cache_telemetry();

    assert_eq!(
        tel_after.total_ops - tel_before.total_ops,
        1,
        "a single get should be exactly 1 cache op"
    );
    assert_eq!(tel_after.hits - tel_before.hits, 1, "should be a hit");
}

// ---------------------------------------------------------------------------
// Cache clear
// ---------------------------------------------------------------------------

#[test]
fn test_cache_clear_and_clear_type() {
    let (engine, _dir) = common::setup_engine();

    let session = engine
        .create_session(NewSession {
            project: "cache-clear".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create");

    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "cache-clear content".into(),
            tags: None,
        })
        .expect("create memory");

    // Sessions are cached via write-through; memories use invalidate-on-create.
    // Pre-load the memory into cache via get (cache-aside) so we can test
    // type-scoped cache clearing.
    engine.get_memory(memory.id).expect("get memory - prime cache");

    engine.clear_cache_type("session");
    // Session should miss now, memory should still hit.
    let tel_before = engine.cache_telemetry();
    let _session_get = engine.get_session(session.id).expect("get session");
    let _memory_get = engine.get_memory(memory.id).expect("get memory");
    let tel_after = engine.cache_telemetry();

    // Session was a miss (cleared), memory was a hit.
    assert_eq!(
        tel_after.misses - tel_before.misses,
        1,
        "session should miss after clear_type"
    );
    assert_eq!(
        tel_after.hits - tel_before.hits,
        1,
        "memory should still hit after clear_type(session)"
    );

    // Clear all.
    engine.clear_cache();
    let tel_before2 = engine.cache_telemetry();
    let _session_get2 = engine.get_session(session.id).expect("get session");
    let _memory_get2 = engine.get_memory(memory.id).expect("get memory");
    let tel_after2 = engine.cache_telemetry();
    // Both should miss now.
    assert_eq!(tel_after2.misses - tel_before2.misses, 2);
}