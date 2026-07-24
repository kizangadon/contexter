//! Memory search and query operations on [`Engine`].
//!
//! Provides structured search (keyword, type, tags, session) and
//! memory counting via filter criteria.

use super::Engine;
use crate::models::*;

use std::sync::atomic::Ordering;

impl Engine {
    /// Search memories using structured query criteria.
    ///
    /// Delegates to the storage backend which uses secondary indexes
    /// (via `memory_index` CF) for `memory_type`, `tags`, and `session_id`
    /// filters and applies keyword relevance scoring + `agent_id` filtering.
    pub fn search_memories(
        &self,
        query: &MemorySearchQuery,
    ) -> crate::error::EngineResult<Vec<Memory>> {
        self.telemetry.stats.searches_completed.fetch_add(1, Ordering::Relaxed);

        self.storage.read().unwrap().search_memories(query)
    }

    /// Count memories matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_memories(&self, filter: &MemoryFilter) -> crate::error::EngineResult<u64> {
        self.storage.read().unwrap().count_memories(filter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use uuid::Uuid;

    /// Helper: create a temporary Engine.
    fn setup() -> (Engine, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        (engine, dir)
    }

    /// Search by keyword and tag.
    #[test]
    fn test_search_memories_by_keyword() {
        let (engine, _dir) = setup();

        let memory = engine
            .create_memory(NewMemory {
                session_id: Uuid::now_v7(),
                agent_id: Uuid::now_v7(),
                memory_type: MemoryType::Fact,
                content: "the quick brown fox jumps over the lazy dog".into(),
                tags: Some(vec!["animal".into(), "nature".into()]),
            })
            .expect("create memory");

        assert_eq!(memory.version, 1);
        assert!(memory.tags.contains(&"animal".to_string()));

        // Search by keyword.
        let results = engine
            .search_memories(&MemorySearchQuery {
                keywords: Some("fox".into()),
                ..MemorySearchQuery::default()
            })
            .expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].content,
            "the quick brown fox jumps over the lazy dog"
        );

        // Search by tag.
        let results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["animal".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by tag");
        assert_eq!(results.len(), 1);
    }

    /// Filter results by memory type.
    #[test]
    fn test_search_by_memory_type() {
        let (engine, _dir) = setup();
        let session_id = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact memory".into(),
                tags: None,
            })
            .expect("create fact");

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Preference,
                content: "preference memory".into(),
                tags: None,
            })
            .expect("create preference");

        // Search by Fact type → should return exactly one.
        let facts = engine
            .search_memories(&MemorySearchQuery {
                memory_type: Some(MemoryType::Fact),
                ..MemorySearchQuery::default()
            })
            .expect("search by type");
        assert_eq!(facts.len(), 1);
        assert_eq!(facts[0].content, "fact memory");

        // Search by non-matching type → should be empty.
        let episodes = engine
            .search_memories(&MemorySearchQuery {
                memory_type: Some(MemoryType::Episode),
                ..MemorySearchQuery::default()
            })
            .expect("search by non-matching type");
        assert!(
            episodes.is_empty(),
            "no memories with Episode type should exist"
        );
    }

    /// Filter results by session ID.
    #[test]
    fn test_search_by_session_id() {
        let (engine, _dir) = setup();
        let session_a = Uuid::now_v7();
        let session_b = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "memory in session A".into(),
                tags: None,
            })
            .expect("create session a");

        engine
            .create_memory(NewMemory {
                session_id: session_b,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "memory in session B".into(),
                tags: None,
            })
            .expect("create session b");

        // Search by session_a → should return exactly one.
        let results_a = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                ..MemorySearchQuery::default()
            })
            .expect("search by session");
        assert_eq!(results_a.len(), 1);
        assert_eq!(results_a[0].content, "memory in session a");

        // Search by session_b → should return exactly one.
        let results_b = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_b),
                ..MemorySearchQuery::default()
            })
            .expect("search by session b");
        assert_eq!(results_b.len(), 1);
        assert_eq!(results_b[0].content, "memory in session b");
    }

    /// Filter results by tags.
    #[test]
    fn test_search_by_tags() {
        let (engine, _dir) = setup();
        let session_id = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "rust programming tips".into(),
                tags: Some(vec!["rust".into(), "programming".into()]),
            })
            .expect("create rust memory");

        engine
            .create_memory(NewMemory {
                session_id,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "python programming tips".into(),
                tags: Some(vec!["python".into(), "programming".into()]),
            })
            .expect("create python memory");

        // Search by tag "rust" → should return exactly one.
        let rust_results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["rust".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by tag");
        assert_eq!(rust_results.len(), 1);
        assert_eq!(rust_results[0].content, "rust programming tips");

        // Search by tag "programming" → should return both.
        let prog_results = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["programming".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by shared tag");
        assert_eq!(prog_results.len(), 2);

        // Search by non-matching tag → empty.
        let no_match = engine
            .search_memories(&MemorySearchQuery {
                tags: Some(vec!["golang".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search by non-matching tag");
        assert!(no_match.is_empty(), "no memories tagged 'golang'");
    }

    /// Combined filters: session + type + tags.
    #[test]
    fn test_search_combined_filters() {
        let (engine, _dir) = setup();
        let session_a = Uuid::now_v7();
        let session_b = Uuid::now_v7();
        let agent_id = Uuid::now_v7();

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact in session A tagged urgent".into(),
                tags: Some(vec!["urgent".into()]),
            })
            .expect("create memory 1");

        engine
            .create_memory(NewMemory {
                session_id: session_a,
                agent_id,
                memory_type: MemoryType::Preference,
                content: "preference in session A tagged urgent".into(),
                tags: Some(vec!["urgent".into()]),
            })
            .expect("create memory 2");

        engine
            .create_memory(NewMemory {
                session_id: session_b,
                agent_id,
                memory_type: MemoryType::Fact,
                content: "fact in session B tagged normal".into(),
                tags: Some(vec!["normal".into()]),
            })
            .expect("create memory 3");

        // Combined: session_a + Fact → 1 result.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                memory_type: Some(MemoryType::Fact),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + fact");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "fact in session a tagged urgent");

        // Combined: session_a + urgent tag → 2 results.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + urgent");
        assert_eq!(results.len(), 2);

        // Combined: session_a + Fact + urgent → 1 result.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_a),
                memory_type: Some(MemoryType::Fact),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_a + fact + urgent");
        assert_eq!(results.len(), 1);

        // Combined: session_b + urgent → 0 results.
        let results = engine
            .search_memories(&MemorySearchQuery {
                session_id: Some(session_b),
                tags: Some(vec!["urgent".into()]),
                ..MemorySearchQuery::default()
            })
            .expect("search session_b + urgent");
        assert!(results.is_empty(), "no urgent memories in session B");
    }

    /// Count memories with filter criteria.
    #[test]
    fn test_count_memories() {
        let (engine, _dir) = setup();
        let session = Uuid::now_v7();

        for i in 0..5 {
            engine
                .create_memory(NewMemory {
                    session_id: session,
                    agent_id: Uuid::now_v7(),
                    memory_type: if i % 2 == 0 {
                        MemoryType::Fact
                    } else {
                        MemoryType::Preference
                    },
                    content: format!("content {i}"),
                    tags: None,
                })
                .expect("create");
        }

        let count = engine
            .count_memories(&MemoryFilter {
                session_id: Some(session),
                ..Default::default()
            })
            .expect("count");
        assert_eq!(count, 5);

        let fact_count = engine
            .count_memories(&MemoryFilter {
                memory_type: Some(MemoryType::Fact),
                ..Default::default()
            })
            .expect("count facts");
        assert_eq!(fact_count, 3);
    }
}
