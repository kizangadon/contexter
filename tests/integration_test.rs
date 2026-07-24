//! Integration tests for the full Contexter engine stack.
//!
//! Each test exercises the full path through Engine → DashMapCache →
//! RocksDbBackend, verifying end-to-end correctness of CRUD operations,
//! cache policies, persistence, concurrency, and maintenance operations.
//!
//! These tests live in `tests/` as integration tests — they link against
//! the compiled library crate, NOT the internal modules.

use std::collections::HashMap;
use std::sync::Arc;

use contexter_core::{
    AgentFilter, AgentStatus, AuditFilter, CacheConfig, Engine, MemoryFilter, MemoryPatch,
    MemorySearchQuery, MemoryType, NewAgent, NewAuditEntry, NewMemory, NewSession, NewSkill,
    Session, SessionFilter, SessionPatch, SessionStatus, SkillFilter, StorageConfig,
};
use tempfile::TempDir;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temporary Engine with default configuration.
fn setup_engine() -> (Engine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::open(dir.path()).expect("open engine");
    (engine, dir)
}

/// Create a temporary Engine with a custom cache config.
fn setup_engine_with_config(config: CacheConfig) -> (Engine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::with_config(StorageConfig {
        path: dir.path().to_path_buf(),
        cache_config: Some(config),
    })
    .expect("open with config");
    (engine, dir)
}

/// Helper to create a session with the given project and agent.
fn create_session(engine: &Engine, project: &str, agent_id: Uuid) -> Session {
    engine
        .create_session(NewSession {
            project: project.to_string(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: None,
        })
        .expect("create session")
}

// ---------------------------------------------------------------------------
// 1. Full session lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_full_session_lifecycle() {
    let (engine, _dir) = setup_engine();
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
// 2. Full memory lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_full_memory_lifecycle() {
    let (engine, _dir) = setup_engine();
    let session_id = Uuid::now_v7();
    let agent_id = Uuid::now_v7();

    // Create a memory.
    let memory = engine
        .create_memory(NewMemory {
            session_id,
            agent_id,
            memory_type: MemoryType::Fact,
            content: "einstein was born in ulm, germany.".to_string(),
            tags: Some(vec!["science".into(), "physics".into()]),
        })
        .expect("create memory");

    // Verify creation defaults.
    assert!(memory.id != Uuid::nil());
    assert_eq!(memory.version, 1);
    assert_eq!(memory.content, "einstein was born in ulm, germany.");
    assert_eq!(memory.memory_type, MemoryType::Fact);
    assert!(memory.tags.contains(&"science".to_string()));
    assert!(memory.tags.contains(&"physics".to_string()));
    assert!(memory.created_at <= memory.updated_at);

    // Get by ID — full round-trip.
    let fetched = engine
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory should exist");
    assert_eq!(fetched.content, memory.content);
    assert_eq!(fetched.memory_type, memory.memory_type);
    assert_eq!(fetched.tags, memory.tags);
    assert_eq!(fetched.version, 1);

    // Update content — version should bump.
    let updated = engine
        .update_memory(
            memory.id,
            &MemoryPatch {
                content: Some("einstein was born in ulm, germany (updated).".into()),
                ..MemoryPatch::default()
            },
        )
        .expect("update memory");
    assert_eq!(updated.version, 2);
    assert_eq!(
        updated.content,
        "einstein was born in ulm, germany (updated)."
    );

    // Search by keyword → should find.
    let kw_results = engine
        .search_memories(&MemorySearchQuery {
            keywords: Some("Einstein".into()),
            ..MemorySearchQuery::default()
        })
        .expect("search memories");
    assert_eq!(kw_results.len(), 1);
    assert_eq!(kw_results[0].id, memory.id);

    // Search by memory type + tag combination.
    let combo_results = engine
        .search_memories(&MemorySearchQuery {
            memory_type: Some(MemoryType::Fact),
            tags: Some(vec!["science".into()]),
            ..MemorySearchQuery::default()
        })
        .expect("search by type + tag");
    assert_eq!(combo_results.len(), 1);

    // Search by non-matching keyword → empty.
    let no_match = engine
        .search_memories(&MemorySearchQuery {
            keywords: Some("quantum".into()),
            ..MemorySearchQuery::default()
        })
        .expect("search memories");
    assert!(no_match.is_empty());

    // Count → 1.
    let count = engine
        .count_memories(&MemoryFilter {
            session_id: Some(session_id),
            ..MemoryFilter::default()
        })
        .expect("count memories");
    assert_eq!(count, 1);

    // Delete → succeeds.
    engine.delete_memory(memory.id).expect("delete memory");

    // Delete again → idempotent.
    engine
        .delete_memory(memory.id)
        .expect("delete again should be idempotent");

    // Get after delete → None.
    let after_delete = engine.get_memory(memory.id).expect("get after delete");
    assert!(after_delete.is_none());
}

// ---------------------------------------------------------------------------
// 3. Cross-entity workflow
// ---------------------------------------------------------------------------

#[test]
fn test_cross_entity_workflow() {
    let (engine, _dir) = setup_engine();

    // Create an agent.
    let agent = engine
        .create_agent(NewAgent {
            name: "albert-einstein".into(),
            agent_type: "physicist".into(),
            description: "Theoretical physicist".into(),
            capabilities: Some(vec!["relativity".into(), "quantum".into()]),
            status: Some(AgentStatus::Active),
            config: Some(serde_json::json!({"field": "physics"})),
        })
        .expect("create agent");
    assert_eq!(agent.name, "albert-einstein");

    // Get agent — verify.
    let fetched_agent = engine
        .get_agent(agent.id)
        .expect("get agent")
        .expect("agent exists");
    assert_eq!(fetched_agent.id, agent.id);

    // Create a skill.
    let skill = engine
        .create_skill(NewSkill {
            name: "relativity-theory".into(),
            description: "Understands general and special relativity".into(),
            category: "physics".into(),
            file_path: None,
        })
        .expect("create skill");
    assert_eq!(skill.name, "relativity-theory");

    // Get skill — verify.
    let fetched_skill = engine
        .get_skill(skill.id)
        .expect("get skill")
        .expect("skill exists");
    assert_eq!(fetched_skill.id, skill.id);

    // Create a session referencing the agent_id.
    let session = create_session(&engine, "physics-research", agent.id);
    assert_eq!(session.agent_id, agent.id);

    // Create a memory referencing the session_id and agent_id.
    let memory = engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id: agent.id,
            memory_type: MemoryType::Fact,
            content: "e=mc^2 is the mass-energy equivalence formula.".to_string(),
            tags: Some(vec!["relativity".into()]),
        })
        .expect("create memory");
    assert_eq!(memory.session_id, session.id);
    assert_eq!(memory.agent_id, agent.id);

    // List sessions → includes the session.
    let sessions = engine
        .list_sessions(&SessionFilter::default())
        .expect("list sessions");
    assert!(sessions.iter().any(|s| s.id == session.id));

    // List agents → includes the agent.
    let agents = engine
        .list_agents(&AgentFilter::default())
        .expect("list agents");
    assert!(agents.iter().any(|a| a.id == agent.id));

    // List skills → includes the skill.
    let skills = engine
        .list_skills(&SkillFilter::default())
        .expect("list skills");
    assert!(skills.iter().any(|s| s.id == skill.id));

    // Search memories by agent_id → includes the memory.
    let agent_memories = engine
        .search_memories(&MemorySearchQuery {
            agent_id: Some(agent.id),
            ..MemorySearchQuery::default()
        })
        .expect("search by agent");
    assert!(agent_memories.iter().any(|m| m.id == memory.id));

    // Delete session → succeeds.
    engine.delete_session(session.id).expect("delete session");

    // Memory still exists after session delete (no cascade).
    let memory_after = engine
        .get_memory(memory.id)
        .expect("get memory after session delete")
        .expect("memory should still exist");
    assert_eq!(memory_after.content, memory.content);
}

// ---------------------------------------------------------------------------
// 4. Cache behavior
// ---------------------------------------------------------------------------

#[test]
fn test_cache_behavior() {
    // Configure a small per-type capacity (5 for sessions).
    let mut per_type = HashMap::new();
    per_type.insert("session".to_string(), 5usize);
    let config = CacheConfig {
        default_capacity: 100,
        per_type_capacity: per_type,
        max_ttl: None,
    };
    let (engine, _dir) = setup_engine_with_config(config);
    let agent_id = Uuid::now_v7();

    // Create one session — write-through caches it.
    let s1 = create_session(&engine, "cache-test", agent_id);

    // Get should be a cache hit (write-through).
    let tel = engine.cache_telemetry();
    let _ = engine
        .get_session(s1.id)
        .expect("get session")
        .expect("exists");
    let tel2 = engine.cache_telemetry();
    assert_eq!(
        tel2.hits - tel.hits,
        1,
        "get after write-through should be a hit"
    );

    // Fill sessions beyond capacity (5). Creating 5 more = 6 total.
    // The LRU should evict the oldest (s1).
    for _ in 0..5 {
        create_session(&engine, "cache-test", Uuid::now_v7());
    }

    // Clear telemetry counters to get a clean reading.
    let tel_before = engine.cache_telemetry();

    // s1 should have been evicted from the cache → miss on get.
    let _ = engine.get_session(s1.id).expect("get session");

    let tel_mid = engine.cache_telemetry();
    assert!(
        tel_mid.misses > tel_before.misses,
        "evicted session should cause a cache miss"
    );

    // But the session still exists in storage.
    let from_storage = engine
        .get_session(s1.id)
        .expect("get session")
        .expect("session should exist in storage");
    assert_eq!(from_storage.id, s1.id);

    // Verify hit ratio tracking is non-zero (at least some ops happened).
    let tel_final = engine.cache_telemetry();
    assert!(
        tel_final.total_ops > 0,
        "cache should have tracked some ops"
    );
    assert!(
        tel_final.hits + tel_final.misses == tel_final.total_ops,
        "hits + misses should equal total_ops"
    );

    // Update session — write-around should invalidate the cache.
    // First warm the cache again.
    let _ = engine.get_session(s1.id).expect("get session");
    let tel_before_update = engine.cache_telemetry();

    let _updated = engine
        .update_session(
            s1.id,
            &SessionPatch {
                turn_count: Some(99),
                ..SessionPatch::default()
            },
        )
        .expect("update session");

    // After update (invalidation), get should be a miss.
    let _ = engine.get_session(s1.id).expect("get session");
    let tel_after_update = engine.cache_telemetry();
    assert!(
        tel_after_update.misses > tel_before_update.misses,
        "update should invalidate cache, causing a miss"
    );

    // Delete — verify cache invalidation.
    engine.delete_session(s1.id).expect("delete session");
    // After delete + invalidation, get returns None.
    let deleted = engine.get_session(s1.id).expect("get after delete");
    assert!(deleted.is_none());
}

// ---------------------------------------------------------------------------
// 5. Storage persistence
// ---------------------------------------------------------------------------

#[test]
fn test_storage_persistence() {
    let dir = TempDir::new().expect("temp dir");
    let agent_id = Uuid::now_v7();

    // Engine 1: create entities.
    let engine1 = Engine::open(dir.path()).expect("open engine1");
    let session = create_session(&engine1, "persistence-test", agent_id);
    let memory = engine1
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id,
            memory_type: MemoryType::Fact,
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
// 6. Settings round-trip
// ---------------------------------------------------------------------------

#[test]
fn test_settings_roundtrip() {
    let (engine, _dir) = setup_engine();

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
// 7. Audit trail
// ---------------------------------------------------------------------------

#[test]
fn test_audit_trail() {
    let (engine, _dir) = setup_engine();
    let agent_id = Uuid::now_v7();

    // Create a session — this should log audit entries via explicit logging.
    // The Engine.create_session does NOT auto-log; we explicitly log.
    let session = create_session(&engine, "audit-test", agent_id);
    engine
        .log_audit(NewAuditEntry {
            action: "create".to_string(),
            entity_type: "session".to_string(),
            entity_id: session.id.to_string(),
            actor: Some("test-user".to_string()),
            changes: Some(serde_json::json!({"project": "audit-test"})),
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
            changes: Some(serde_json::json!({"turn_count": 10})),
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
            changes: None,
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
// 8. Concurrent operations
// ---------------------------------------------------------------------------

#[test]
fn test_concurrent_operations() {
    let (engine, _dir) = setup_engine();
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
// 9. Large dataset and pagination
// ---------------------------------------------------------------------------

#[test]
fn test_large_dataset() {
    let (engine, _dir) = setup_engine();
    let agent_id = Uuid::now_v7();

    // Create 200 sessions.
    let mut ids = Vec::with_capacity(200);
    for i in 0..200 {
        let session = create_session(&engine, &format!("bulk-{}", i % 10), agent_id);
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
// 10. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_edge_cases() {
    let (engine, _dir) = setup_engine();

    // Empty session list → empty vec.
    let empty_sessions = engine
        .list_sessions(&SessionFilter::default())
        .expect("list sessions");
    assert!(empty_sessions.is_empty());

    // Search memory with empty query → all memories (none).
    let empty_memories = engine
        .search_memories(&MemorySearchQuery::default())
        .expect("search memories");
    assert!(empty_memories.is_empty());

    // Get non-existent agent → None.
    let no_agent = engine
        .get_agent(Uuid::now_v7())
        .expect("get non-existent agent");
    assert!(no_agent.is_none());

    // Delete non-existent skill → idempotent (no error).
    engine
        .delete_skill(Uuid::now_v7())
        .expect("delete non-existent skill should not error");

    // Update session that doesn't exist → NotFound error.
    let update_result = engine.update_session(
        Uuid::now_v7(),
        &SessionPatch {
            status: Some(SessionStatus::Completed),
            ..SessionPatch::default()
        },
    );
    assert!(
        update_result.is_err(),
        "updating non-existent session should error"
    );

    // Multiple deletes of same entity — first succeeds, second idempotent.
    let agent_id = Uuid::now_v7();
    let session = create_session(&engine, "multi-delete", agent_id);
    engine
        .delete_session(session.id)
        .expect("first delete should succeed");
    engine
        .delete_session(session.id)
        .expect("second delete should be idempotent");

    // Create memory with empty content → succeeds (valid).
    let empty_memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "".to_string(),
            tags: None,
        })
        .expect("create memory with empty content");
    assert!(empty_memory.content.is_empty());
    assert_eq!(empty_memory.version, 1);

    // List with non-matching filter → empty.
    let no_match = engine
        .list_sessions(&SessionFilter {
            project: Some("impossible-project-name".to_string()),
            ..SessionFilter::default()
        })
        .expect("list non-matching");
    assert!(no_match.is_empty());

    // Count with non-matching filter → 0.
    let zero_count = engine
        .count_sessions(&SessionFilter {
            project: Some("impossible-project-name".to_string()),
            ..SessionFilter::default()
        })
        .expect("count non-matching");
    assert_eq!(zero_count, 0);

    // Get non-existent settings → None.
    assert!(engine
        .get_setting("key-does-not-exist")
        .expect("get non-existent setting")
        .is_none());

    // Create and delete a memory with empty tags.
    let mem_no_tags = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Procedure,
            content: "no tags".to_string(),
            tags: None,
        })
        .expect("create memory without tags");
    assert!(mem_no_tags.tags.is_empty());
    engine.delete_memory(mem_no_tags.id).expect("delete memory");

    // Verify agent list after all operations.
    let agents = engine
        .list_agents(&AgentFilter::default())
        .expect("list agents");
    assert!(agents.is_empty(), "no agents created, should be empty");
}

// ---------------------------------------------------------------------------
// 11. Maintenance operations
// ---------------------------------------------------------------------------

#[test]
fn test_maintenance_operations() {
    let (engine, _dir) = setup_engine();
    let agent_id = Uuid::now_v7();

    // Write some data.
    let session = create_session(&engine, "maintenance-test", agent_id);
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
        .create_agent(NewAgent {
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
    assert!(
        (0.0..=1.0).contains(&tel.hit_ratio),
        "hit_ratio should be in [0, 1]"
    );
    // At least some entity types should have entries.
    assert!(
        !tel.entries_by_type.is_empty(),
        "entries_by_type should not be empty"
    );
}

// ---------------------------------------------------------------------------
// 12. Read-only path error
// ---------------------------------------------------------------------------

#[test]
fn test_read_only_path_error() -> Result<(), Box<dyn std::error::Error>> {
    // Test that a read-only directory returns an error on Engine::open
    let dir = TempDir::new()?;
    let ro_path = dir.path().join("ro");
    std::fs::create_dir(&ro_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ro_path, std::fs::Permissions::from_mode(0o444))?;
    }

    let result = Engine::open(ro_path.to_str().unwrap());
    // Should fail with storage error (can't write to read-only dir)
    assert!(result.is_err());

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&ro_path, std::fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// 13. Generic store/get roundtrip
// ---------------------------------------------------------------------------

#[test]
fn test_generic_store_get_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    let (engine, _dir) = setup_engine();

    engine.store("sessions", "cfg:test_key", b"test_value")?;
    let result = engine.get("sessions", "cfg:test_key")?;
    assert_eq!(result, Some(b"test_value".to_vec()));

    // Non-existent key
    let result = engine.get("sessions", "cfg:nonexistent")?;
    assert_eq!(result, None);

    Ok(())
}
