//! Integration tests for memory CRUD — lifecycle, cross-entity workflows,
//! and edge cases.

use contexter_core::{
    AgentFilter, AgentStatus, MemoryFilter, MemoryPatch, MemorySearchQuery, MemoryType, NewAgent,
    NewMemory, NewSession, NewSkill, SessionFilter, SessionPatch, SessionStatus, SkillFilter,
};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// 1. Full memory lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_full_memory_lifecycle() {
    let (engine, _dir) = common::setup_engine();
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
// 2. Cross-entity workflow
// ---------------------------------------------------------------------------

#[test]
fn test_cross_entity_workflow() {
    let (engine, _dir) = common::setup_engine();

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
    let session = common::create_session(&engine, "physics-research", agent.id);
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
// 3. Edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_edge_cases() {
    let (engine, _dir) = common::setup_engine();

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
    let session = common::create_session(&engine, "multi-delete", agent_id);
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
// 4. Memory cache behaviour (extracted from inline engine tests)
// ---------------------------------------------------------------------------

#[test]
fn test_memory_get_cached() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "cache-me".into(),
            tags: None,
        })
        .expect("create");

    // First get after invalidate-on-create should be a L1 miss (populates cache).
    let tel_before = engine.cache_telemetry();
    let fetched = engine.get_memory(memory.id).expect("get memory");
    let tel_after = engine.cache_telemetry();
    assert!(fetched.is_some());
    assert_eq!(tel_after.misses - tel_before.misses, 1,
        "first get should miss (invalidate-on-create policy)");
    // Second get should be a hit (cache-aside populated the cache).
    let tel_before2 = engine.cache_telemetry();
    let _fetched2 = engine.get_memory(memory.id).expect("get memory");
    let tel_after2 = engine.cache_telemetry();
    assert_eq!(tel_after2.hits - tel_before2.hits, 1,
        "second get should be a hit");
}

#[test]
fn test_memory_update_version_bump() {
    let (engine, _dir) = common::setup_engine();

    let created = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "v1 content".into(),
            tags: None,
        })
        .expect("create");
    assert_eq!(created.version, 1);

    let updated = engine
        .update_memory(
            created.id,
            &MemoryPatch {
                content: Some("v2 content".into()),
                ..MemoryPatch::default()
            },
        )
        .expect("update");
    assert_eq!(updated.version, 2);

    let updated2 = engine
        .update_memory(
            created.id,
            &MemoryPatch {
                content: Some("v3 content".into()),
                ..MemoryPatch::default()
            },
        )
        .expect("update again");
    assert_eq!(updated2.version, 3);

    // After update, cache should be invalidated. Re-fetch should give v3.
    let fetched = engine.get_memory(created.id).expect("get").expect("exists");
    assert_eq!(fetched.content, "v3 content");
}

#[test]
fn test_memory_delete_invalidates_cache() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Preference,
            content: "to be deleted".into(),
            tags: None,
        })
        .expect("create");

    engine.delete_memory(memory.id).expect("delete");
    assert!(engine
        .get_memory(memory.id)
        .expect("get after delete")
        .is_none());
}

// ---------------------------------------------------------------------------
// Content size limits
// ---------------------------------------------------------------------------

#[test]
fn test_memory_content_exactly_1mb_succeeds() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "x".repeat(1024 * 1024),
            tags: None,
        })
        .expect("1MB memory content should succeed");
    assert_eq!(memory.content.len(), 1024 * 1024);
}

#[test]
fn test_memory_content_exceeds_limit_rejected() {
    let (engine, _dir) = common::setup_engine();
    let result = engine.create_memory(NewMemory {
        session_id: Uuid::now_v7(),
        agent_id: Uuid::now_v7(),
        memory_type: MemoryType::Fact,
        content: "x".repeat(1024 * 1024 + 1),
        tags: None,
    });
    assert!(result.is_err(), "oversized memory should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("1MB"), "error should mention the size limit");
}

#[test]
fn test_update_memory_content_exactly_1mb_succeeds() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "initial".into(),
            tags: None,
        })
        .expect("create");
    let updated = engine
        .update_memory(memory.id, &MemoryPatch {
            content: Some("x".repeat(1024 * 1024)),
            ..MemoryPatch::default()
        })
        .expect("1MB update content should succeed");
    assert_eq!(updated.content.len(), 1024 * 1024);
}

#[test]
fn test_update_memory_content_exceeds_limit_rejected() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "initial".into(),
            tags: None,
        })
        .expect("create");
    let result = engine.update_memory(memory.id, &MemoryPatch {
        content: Some("x".repeat(1024 * 1024 + 1)),
        ..MemoryPatch::default()
    });
    assert!(result.is_err(), "oversized update content should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(err.contains("1MB"), "error should mention the size limit");
}

#[test]
fn test_update_memory_content_none_skips_validation() {
    let (engine, _dir) = common::setup_engine();
    let memory = engine
        .create_memory(NewMemory {
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "initial".into(),
            tags: None,
        })
        .expect("create");
    let updated = engine
        .update_memory(memory.id, &MemoryPatch {
            memory_type: Some(MemoryType::Preference),
            ..MemoryPatch::default()
        })
        .expect("update without content should succeed");
    assert_eq!(updated.memory_type, MemoryType::Preference);
}
