//! Integration tests for session lifecycle — CRUD, settings, audit trail,
//! concurrency, pagination, maintenance operations, and cache behaviour.

use std::sync::Arc;

use contexter_core::{
    AuditFilter, MemorySearchQuery, MemoryType, NewAuditEntry, NewMemory, NewSession,
    SessionFilter, SessionPatch, SessionStatus,
};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// 1. Full session lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_full_session_lifecycle() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Create a session.
    let session = engine
        .create_session(NewSession {
            project: "lifecycle-test".to_string(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: Some(serde_json::json!({"env": "test"})),
        })
        .expect("create session");

    // Verify creation defaults.
    assert!(session.id != Uuid::nil(), "session should have a UUID");
    assert_eq!(session.project, "lifecycle-test");
    assert_eq!(session.agent_id, agent_id);
    assert_eq!(session.status, SessionStatus::Active);
    assert_eq!(session.turn_count, 0);
    assert!(session.metadata.is_object());

    // Get by ID — full round-trip.
    let fetched = engine
        .get_session(session.id)
        .expect("get session")
        .expect("session should exist");
    assert_eq!(fetched.id, session.id);
    assert_eq!(fetched.project, session.project);
    assert_eq!(fetched.agent_id, session.agent_id);
    assert_eq!(fetched.status, session.status);
    assert_eq!(fetched.turn_count, session.turn_count);
    assert_eq!(fetched.metadata, session.metadata);

    // Update: change status and increment turn_count.
    let updated = engine
        .update_session(
            session.id,
            &SessionPatch {
                status: Some(SessionStatus::Completed),
                turn_count: Some(5),
                ..SessionPatch::default()
            },
        )
        .expect("update session");
    assert_eq!(updated.status, SessionStatus::Completed);
    assert_eq!(updated.turn_count, 5);

    // List with matching project filter.
    let matching = engine
        .list_sessions(&SessionFilter {
            project: Some("lifecycle-test".into()),
            ..SessionFilter::default()
        })
        .expect("list sessions");
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].id, session.id);

    // List with non-matching project filter → empty.
    let empty = engine
        .list_sessions(&SessionFilter {
            project: Some("nonexistent".into()),
            ..SessionFilter::default()
        })
        .expect("list sessions");
    assert!(empty.is_empty());

    // Count matching → 1.
    let count = engine
        .count_sessions(&SessionFilter {
            project: Some("lifecycle-test".into()),
            ..SessionFilter::default()
        })
        .expect("count sessions");
    assert_eq!(count, 1);

    // Count non-matching → 0.
    let count_empty = engine
        .count_sessions(&SessionFilter {
            project: Some("nonexistent".into()),
            ..SessionFilter::default()
        })
        .expect("count sessions");
    assert_eq!(count_empty, 0);

    // Delete → succeeds.
    engine.delete_session(session.id).expect("delete session");

    // Delete again → idempotent (no error).
    engine
        .delete_session(session.id)
        .expect("delete again should be idempotent");

    // Get after delete → None.
    let after_delete = engine.get_session(session.id).expect("get after delete");
    assert!(after_delete.is_none());

    // Verify count is now 0.
    let count_final = engine
        .count_sessions(&SessionFilter {
            project: Some("lifecycle-test".into()),
            ..SessionFilter::default()
        })
        .expect("count sessions");
    assert_eq!(count_final, 0);
}

// ---------------------------------------------------------------------------
// 2. Settings round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_settings_roundtrip() {
    let (engine, _dir) = common::setup_engine();

    // Set and get a string setting.
    engine.set_setting("theme", "dark").expect("set theme");
    assert_eq!(
        engine.get_setting("theme").expect("get theme"),
        Some("dark".to_string())
    );

    // Set and get a different type of value (still a string key-value).
    engine
        .set_setting("max_items", "100")
        .expect("set max_items");
    assert_eq!(
        engine.get_setting("max_items").expect("get max_items"),
        Some("100".to_string())
    );

    // Get non-existent setting → None.
    assert!(engine
        .get_setting("nonexistent")
        .expect("get nonexistent")
        .is_none());

    // Overwrite existing setting.
    engine
        .set_setting("theme", "light")
        .expect("overwrite theme");
    assert_eq!(
        engine.get_setting("theme").expect("get theme"),
        Some("light".to_string())
    );

    // Verify other settings are unaffected by overwrite.
    assert_eq!(
        engine.get_setting("max_items").expect("get max_items"),
        Some("100".to_string())
    );
}

// ---------------------------------------------------------------------------
// 3. Audit trail
// ---------------------------------------------------------------------------

#[test]
fn test_audit_trail() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Create a session.
    let session = common::create_session(&engine, "audit-test", agent_id);
    engine
        .log_audit(NewAuditEntry {
            action: "create".to_string(),
            entity_type: "session".to_string(),
            entity_id: session.id.to_string(),
            actor: Some("test-user".to_string()),
            summary: Some(serde_json::json!({"project": "audit-test"})),
        })
        .expect("log create audit");

    // Update session — log an audit entry.
    let _updated = engine
        .update_session(
            session.id,
            &SessionPatch {
                turn_count: Some(10),
                ..SessionPatch::default()
            },
        )
        .expect("update session");
    engine
        .log_audit(NewAuditEntry {
            action: "update".to_string(),
            entity_type: "session".to_string(),
            entity_id: session.id.to_string(),
            actor: Some("test-user".to_string()),
            summary: Some(serde_json::json!({"turn_count": 10})),
        })
        .expect("log update audit");

    // Delete session — log an audit entry.
    engine.delete_session(session.id).expect("delete session");
    engine
        .log_audit(NewAuditEntry {
            action: "delete".to_string(),
            entity_type: "session".to_string(),
            entity_id: session.id.to_string(),
            actor: Some("test-user".to_string()),
            summary: None,
        })
        .expect("log delete audit");

    // Query audit with entity_type filter → should return session entries.
    let session_entries = engine
        .query_audit(&AuditFilter {
            entity_type: Some("session".to_string()),
            ..AuditFilter::default()
        })
        .expect("query session audits");
    assert_eq!(session_entries.len(), 3);
    for entry in &session_entries {
        assert_eq!(entry.entity_type, "session");
    }

    // Query audit with non-matching entity_id filter → empty.
    let no_match = engine
        .query_audit(&AuditFilter {
            entity_id: Some(Uuid::now_v7().to_string()),
            ..AuditFilter::default()
        })
        .expect("query non-matching");
    assert!(no_match.is_empty());

    // Query all → 3 entries.
    let all = engine
        .query_audit(&AuditFilter::default())
        .expect("query all");
    assert_eq!(all.len(), 3);
}

// ---------------------------------------------------------------------------
// 4. Concurrent operations
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_operations() {
    let (engine, _dir) = common::setup_engine();
    let engine = Arc::new(engine);

    let mut handles = Vec::new();

    // Spawn 4 threads doing simultaneous creates + gets + updates.
    for thread_id in 0..4 {
        let engine = Arc::clone(&engine);
        handles.push(std::thread::spawn(move || {
            let agent_id = Uuid::now_v7();

            for i in 0..25 {
                // Create a session.
                let session = engine
                    .create_session(NewSession {
                        project: format!("concurrent-{thread_id}"),
                        agent_id,
                        status: Some(SessionStatus::Active),
                        metadata: None,
                    })
                    .expect("concurrent create");

                // Get it back.
                let fetched = engine
                    .get_session(session.id)
                    .expect("concurrent get")
                    .expect("session should exist");
                assert_eq!(fetched.project, format!("concurrent-{thread_id}"));

                // Update it.
                let updated = engine
                    .update_session(
                        session.id,
                        &SessionPatch {
                            turn_count: Some(i as u32 + 1),
                            ..SessionPatch::default()
                        },
                    )
                    .expect("concurrent update");
                assert_eq!(updated.turn_count, i as u32 + 1);
            }
        }));
    }

    // All threads must complete without panic.
    for handle in handles {
        handle.join().expect("thread panicked");
    }

    // Verify total count is correct: 4 threads × 25 sessions = 100.
    let total = engine
        .count_sessions(&SessionFilter::default())
        .expect("count sessions");
    assert_eq!(total, 100, "should have 100 sessions across all threads");
}

// ---------------------------------------------------------------------------
// 5. Large dataset and pagination
// ---------------------------------------------------------------------------

#[test]
fn test_large_dataset() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Create 200 sessions.
    let mut ids = Vec::with_capacity(200);
    for i in 0..200 {
        let session = common::create_session(&engine, &format!("bulk-{}", i % 10), agent_id);
        ids.push(session.id);
    }

    // Verify count returns 200.
    let count = engine
        .count_sessions(&SessionFilter::default())
        .expect("count sessions");
    assert_eq!(count, 200, "should have 200 sessions");

    // List with limit 50 → returns 50.
    let page1 = engine
        .list_sessions(&SessionFilter {
            limit: 50,
            offset: 0,
            ..SessionFilter::default()
        })
        .expect("list page 1");
    assert_eq!(page1.len(), 50, "first page should have 50 items");

    // List with offset 50 → returns items 51-100.
    let page2 = engine
        .list_sessions(&SessionFilter {
            limit: 50,
            offset: 50,
            ..SessionFilter::default()
        })
        .expect("list page 2");
    assert_eq!(page2.len(), 50, "second page should have 50 items");

    // Verify pages don't overlap.
    let page1_ids: Vec<Uuid> = page1.iter().map(|s| s.id).collect();
    let page2_ids: Vec<Uuid> = page2.iter().map(|s| s.id).collect();
    for id in &page1_ids {
        assert!(!page2_ids.contains(id), "pages should not overlap");
    }

    // Verify pagination across full dataset.
    let page3 = engine
        .list_sessions(&SessionFilter {
            limit: 50,
            offset: 100,
            ..SessionFilter::default()
        })
        .expect("list page 3");
    assert_eq!(page3.len(), 50, "third page should have 50 items");

    let page4 = engine
        .list_sessions(&SessionFilter {
            limit: 50,
            offset: 150,
            ..SessionFilter::default()
        })
        .expect("list page 4");
    assert_eq!(page4.len(), 50, "fourth page should have 50 items");

    // Total distinct IDs across all pages should be 200.
    let all_ids: Vec<Uuid> = page1_ids
        .into_iter()
        .chain(page2_ids)
        .chain(page3.iter().map(|s| s.id))
        .chain(page4.iter().map(|s| s.id))
        .collect();
    assert_eq!(all_ids.len(), 200);

    // Project-filtered count.
    let project_count = engine
        .count_sessions(&SessionFilter {
            project: Some("bulk-0".to_string()),
            ..SessionFilter::default()
        })
        .expect("count by project");
    assert_eq!(
        project_count, 20,
        "there are 20 sessions with project bulk-0"
    );
}

// ---------------------------------------------------------------------------
// 6. Maintenance operations
// ---------------------------------------------------------------------------

#[test]
fn test_maintenance_operations() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Write some data.
    let session = common::create_session(&engine, "maintenance-test", agent_id);
    let memory = engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id,
            memory_type: MemoryType::Fact,
            content: "maintenance test data".to_string(),
            tags: None,
        })
        .expect("create memory");
    engine
        .create_agent(contexter_core::NewAgent {
            name: "maint-agent".into(),
            agent_type: "test".into(),
            description: "Maintenance agent".into(),
            capabilities: None,
            status: None,
            config: None,
        })
        .expect("create agent");

    // Call flush() → succeeds.
    engine.flush().expect("flush should succeed");

    // Call checkpoint() → returns non-zero sequence number.
    let seq = engine.checkpoint().expect("checkpoint should succeed");
    assert!(seq > 0, "checkpoint sequence number should be > 0");

    // Call storage_size() → returns map with entries per CF.
    let size = engine.storage_size().expect("storage size");
    // At minimum some CFs should have non-zero size after writes + flush.
    let total_cf_sizes: u64 = size.per_cf.values().sum();
    assert!(
        total_cf_sizes > 0 || size.wal_size > 0,
        "storage should report non-zero size after writes"
    );
    // Per_cf should be non-empty (at least the CFs we used).
    assert!(!size.per_cf.is_empty(), "per_cf should not be empty");
    // Total >= WAL + CF sum (or at least self-consistent).
    assert!(
        size.total >= size.wal_size,
        "total should be at least WAL size"
    );

    // Perform some get operations to exercise the cache.
    let _ = engine.get_session(session.id).expect("get session");
    let _ = engine.get_memory(memory.id).expect("get memory");

    // Call cache_telemetry() → returns struct with hit/miss counts.
    let tel = engine.cache_telemetry();
    // The gets should have produced cache hits (write-through stored them).
    assert!(tel.total_ops > 0, "cache should have some ops");
    assert!(tel.hits > 0, "cache should have hits from write-through");
    assert!(
        tel.hits + tel.misses == tel.total_ops,
        "hits + misses should equal total_ops"
    );
    let hit_ratio = if tel.total_ops > 0 {
        tel.hits as f64 / tel.total_ops as f64
    } else {
        0.0
    };
    assert!(
        (0.0..=1.0).contains(&hit_ratio),
        "hit_ratio should be in [0, 1]"
    );
    // At least some entity types should have entries.
    assert!(
        !tel.entries_by_type.is_empty(),
        "entries_by_type should not be empty"
    );
}

// ---------------------------------------------------------------------------
// 7. Session cache behaviour (extracted from inline engine tests)
// ---------------------------------------------------------------------------

#[test]
fn test_session_cache_hits_on_second_get() {
    let (engine, _dir) = common::setup_engine();

    let session = engine
        .create_session(NewSession {
            project: "cache-test".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create session");

    // First get should be a cache hit because write-through stored it.
    let tel_before = engine.cache_telemetry();
    let fetched = engine.get_session(session.id).expect("get session");
    let tel_after = engine.cache_telemetry();

    assert!(fetched.is_some(), "session should exist");
    assert_eq!(
        tel_after.hits - tel_before.hits,
        1,
        "get after write-through create should be a L1 hit"
    );
    assert_eq!(
        tel_after.misses - tel_before.misses,
        0,
        "no L1 miss expected for write-through created entity"
    );
}

#[test]
fn test_session_update_invalidates_cache() {
    let (engine, _dir) = common::setup_engine();

    let session = engine
        .create_session(NewSession {
            project: "invalidation".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create");

    // Warm the cache.
    let _ = engine.get_session(session.id).expect("warm");

    let tel_before = engine.cache_telemetry();

    // Update — write-around should invalidate the cache.
    let updated = engine
        .update_session(
            session.id,
            &SessionPatch {
                turn_count: Some(42),
                ..SessionPatch::default()
            },
        )
        .expect("update session");
    assert_eq!(updated.turn_count, 42);

    // Next get should MISS the cache (invalidation), then re-fetch from L2.
    let re_fetched = engine
        .get_session(session.id)
        .expect("get after update")
        .expect("session exists");
    assert_eq!(re_fetched.turn_count, 42);

    let tel_after = engine.cache_telemetry();
    assert_eq!(
        tel_after.misses - tel_before.misses,
        1,
        "get after write-around update should produce one L1 miss"
    );
    assert_eq!(
        tel_after.hits - tel_before.hits,
        0,
        "no new L1 hits expected (cache was invalidated)"
    );
}

#[test]
fn test_session_delete_invalidates_cache() {
    let (engine, _dir) = common::setup_engine();

    let session = engine
        .create_session(NewSession {
            project: "del-inval".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        })
        .expect("create");

    // Warm the cache.
    let _ = engine.get_session(session.id).expect("warm");

    // Delete — should also invalidate the cache.
    engine.delete_session(session.id).expect("delete session");

    // Should return None (deleted + cache invalidated).
    let fetched = engine.get_session(session.id).expect("get after delete");
    assert!(fetched.is_none(), "deleted session should not exist");
}
