//! Integration tests for memory search via the Engine API.
//!
//! Tests keyword search, agent_id filtering, and combined search across
//! multiple dimensions.

use contexter_core::{MemorySearchQuery, MemoryType, NewMemory, NewSession};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Keyword search
// ---------------------------------------------------------------------------

/// Search memories by content keyword.
///
/// Creates two memories with distinct content and verifies that each can be
/// found by its unique keyword, and that a non-matching keyword returns
/// empty results.
#[test]
fn test_search_by_content() {
    let (engine, _dir) = common::setup_engine();
    let agent = Uuid::now_v7();
    let session = engine
        .create_session(NewSession {
            project: "test".into(),
            agent_id: agent,
            status: None,
            metadata: None,
        })
        .expect("create session");

    engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id: agent,
            memory_type: MemoryType::Fact,
            content: "rust is fast".into(),
            tags: None,
        })
        .expect("create memory");

    engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id: agent,
            memory_type: MemoryType::Fact,
            content: "python is fun".into(),
            tags: None,
        })
        .expect("create memory");

    // Search for "rust" keyword → 1 result
    let results = engine
        .search_memories(&MemorySearchQuery {
            keywords: Some("rust".into()),
            ..MemorySearchQuery::default()
        })
        .expect("search memories");
    assert_eq!(results.len(), 1, "expected 1 result for 'rust'");
    assert_eq!(results[0].content, "rust is fast");

    // Search for "python" keyword → 1 result
    let results = engine
        .search_memories(&MemorySearchQuery {
            keywords: Some("python".into()),
            ..MemorySearchQuery::default()
        })
        .expect("search memories");
    assert_eq!(results.len(), 1, "expected 1 result for 'python'");
    assert_eq!(results[0].content, "python is fun");

    // Search with non-matching keyword → empty
    let results = engine
        .search_memories(&MemorySearchQuery {
            keywords: Some("golang".into()),
            ..MemorySearchQuery::default()
        })
        .expect("search memories");
    assert!(
        results.is_empty(),
        "expected no results for non-matching keyword"
    );
}

// ---------------------------------------------------------------------------
// Agent-id filtering
// ---------------------------------------------------------------------------

/// Search memories by agent_id filter.
///
/// Creates two memories with different agent IDs and verifies that each
/// filter returns only the memories created by that agent.
#[test]
fn test_search_by_agent_id() {
    let (engine, _dir) = common::setup_engine();
    let agent_a = Uuid::now_v7();
    let agent_b = Uuid::now_v7();
    let session = engine
        .create_session(NewSession {
            project: "search".into(),
            agent_id: agent_a,
            status: None,
            metadata: None,
        })
        .expect("create session");

    // Create memory with agent_a
    engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id: agent_a,
            memory_type: MemoryType::Fact,
            content: "memory from agent a".into(),
            tags: None,
        })
        .expect("create memory");

    // Create memory with agent_b
    engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id: agent_b,
            memory_type: MemoryType::Fact,
            content: "memory from agent b".into(),
            tags: None,
        })
        .expect("create memory");

    // Search by agent_a → 1 result
    let results = engine
        .search_memories(&MemorySearchQuery {
            agent_id: Some(agent_a),
            ..MemorySearchQuery::default()
        })
        .expect("search by agent_id");
    assert_eq!(results.len(), 1, "expected 1 result for agent_a");
    assert_eq!(results[0].content, "memory from agent a");

    // Search by agent_b → 1 result
    let results = engine
        .search_memories(&MemorySearchQuery {
            agent_id: Some(agent_b),
            ..MemorySearchQuery::default()
        })
        .expect("search by agent_id");
    assert_eq!(results.len(), 1, "expected 1 result for agent_b");
    assert_eq!(results[0].content, "memory from agent b");
}
