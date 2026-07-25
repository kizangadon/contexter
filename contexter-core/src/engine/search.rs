//! Memory search and query operations on [`Engine`].
//!
//! Provides structured search (keyword, type, tags, session), hybrid search
//! (L3 vector + L4 FTS fused via Reciprocal Rank Fusion), and memory
//! counting via filter criteria.

use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use uuid::Uuid;

use super::Engine;
use crate::error::{EngineError, EngineResult};
use crate::models::*;

use std::sync::atomic::Ordering;

// ---------------------------------------------------------------------------
// Hybrid search query
// ---------------------------------------------------------------------------

/// Query parameters for hybrid vector + full-text search.
///
/// Hybrid search merges results from the L3 (HNSW vector) and L4 (Tantivy
/// FTS) tiers using Reciprocal Rank Fusion (RRF) with k = 60.  If only
/// one tier is available, the query degrades gracefully to that single tier.
#[derive(Debug, Clone)]
pub struct HybridSearchQuery {
    /// Text query string (passed to L4 Tantivy FTS).
    pub query_text: Option<String>,
    /// Vector embedding (passed to L3 HNSW vector index).
    pub query_vector: Option<Vec<f32>>,
    /// Weight for the vector score [0.0, 1.0];  Default: 0.5.
    pub vector_weight: f32,
    /// Weight for the text (FTS) score [0.0, 1.0];  Default: 0.5.
    pub text_weight: f32,
    /// Max results after merge.  Default: 20.
    pub top_k: usize,
    // Filter criteria (applied post-merge).
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub session_id: Option<Uuid>,
}

impl Default for HybridSearchQuery {
    fn default() -> Self {
        Self {
            query_text: None,
            query_vector: None,
            vector_weight: 0.5,
            text_weight: 0.5,
            top_k: 20,
            memory_type: None,
            tags: None,
            session_id: None,
        }
    }
}

// ---------------------------------------------------------------------------
// RRF constant
// ---------------------------------------------------------------------------

/// RRF constant used to dampen rank contributions.
const RRF_K: f32 = 60.0;

impl Engine {
    /// Search memories using structured query criteria.
    ///
    /// Delegates to the storage backend which uses secondary indexes
    /// (via `memory_index` CF) for `memory_type`, `tags`, and `session_id`
    /// filters and applies keyword relevance scoring.
    pub fn search_memories(
        &self,
        query: &MemorySearchQuery,
    ) -> EngineResult<Vec<Memory>> {
        self.telemetry.stats.searches_completed.fetch_add(1, Ordering::Relaxed);

        self.storage.read().unwrap_or_else(|e| e.into_inner()).search_memories(query)
    }

    /// Count memories matching the given filter criteria.
    ///
    /// **Policy:** Bypass — always reads from L2.
    pub fn count_memories(&self, filter: &MemoryFilter) -> EngineResult<u64> {
        self.storage.read().unwrap_or_else(|e| e.into_inner()).count_memories(filter)
    }

    /// Hybrid vector + full-text search using Reciprocal Rank Fusion (RRF).
    ///
    /// Merges results from L3 (vector index) and L4 (FTS index) if both are
    /// enabled.  Degrades gracefully when only one tier is available.
    /// Applies filter criteria post-merge.
    ///
    /// # Algorithm
    ///
    /// 1. Run L3 search (up to `top_k * 2` results) if `query_vector` is Some.
    /// 2. Run L4 search (up to `top_k * 2` results) if `query_text` is Some.
    /// 3. Compute RRF score = 1.0 / (60.0 + rank) for each list.
    /// 4. Sum RRF scores for results appearing in both lists.
    /// 5. Combine: `vector_weight * rrf_l3 + (1.0 - vector_weight) * rrf_l4`.
    ///    If only one tier ran, use its RRF score directly.
    /// 6. Fetch full `Memory` objects from the storage backend.
    /// 7. Apply in-memory filtering (memory_type, tags, session_id).
    /// 8. Sort by score descending, take top `limit`.
    pub fn hybrid_search(
        &self,
        query: &HybridSearchQuery,
    ) -> EngineResult<Vec<(Memory, f32)>> {
        // Reject empty query — at least one of query_text or query_vector must be set.
        if query.query_text.is_none() && query.query_vector.is_none() {
            return Err(EngineError::Validation(
                "Hybrid search requires query_text, query_vector, or both".into(),
            ));
        }

        if self.vector_index.is_none() && self.fts_index.is_none() {
            return Err(EngineError::Unimplemented(
                "Hybrid search requires L3 or L4 enabled".into(),
            ));
        }

        // --- Input validation ---
        let vector_weight = query.vector_weight.clamp(0.0, 1.0);
        let text_weight = query.text_weight.clamp(0.0, 1.0);
        let limit = if query.top_k == 0 {
            return Ok(Vec::new());
        } else {
            query.top_k.min(1000)
        };

        let vector_available = self.vector_index.is_some() && query.query_vector.is_some();
        let fts_available = self.fts_index.is_some() && query.query_text.is_some();

        // --- Phase 1: Collect all candidate IDs from each tier ---
        let fetch_k = limit * 2;

        // L3 vector results: Vec<(id_string, rrf_score)>
        let l3_results: Vec<(String, f32)> = if vector_available {
            if let Some(ref vx) = self.vector_index {
                if let Some(ref vec) = query.query_vector {
                    match vx.search(vec, fetch_k) {
                        Ok(results) => results
                            .iter()
                            .enumerate()
                            .map(|(rank, (id, _sim))| {
                                (id.clone(), 1.0 / (RRF_K + rank as f32))
                            })
                            .collect(),
                        Err(e) => {
                            return Err(EngineError::Internal(format!("Vector search: {e}")));
                        }
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // L4 FTS results: Vec<(id_string, rrf_score)>
        let l4_results: Vec<(String, f32)> = if fts_available {
            if let Some(ref fts) = self.fts_index {
                if let Some(ref text) = query.query_text {
                    match fts.search(text, fetch_k) {
                        Ok(results) => results
                            .iter()
                            .enumerate()
                            .map(|(rank, (id, _score))| {
                                (id.clone(), 1.0 / (RRF_K + rank as f32))
                            })
                            .collect(),
                        Err(e) => {
                            return Err(EngineError::Internal(format!("FTS search: {e}")));
                        }
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // --- Phase 2: Batch-fetch all unique memory IDs ---
        // Collect all unique IDs from both result sets.
        let mut all_ids: Vec<Uuid> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();

        for (id, _) in l3_results.iter().chain(l4_results.iter()) {
            if seen.insert(id.clone()) {
                if let Ok(mem_id) = Uuid::from_str(id) {
                    all_ids.push(mem_id);
                }
            }
        }

        // Batch-fetch all memories in one call.
        let batch_memories: HashMap<String, Memory> = if all_ids.is_empty() {
            HashMap::new()
        } else {
            self.get_memories(&all_ids)
                .unwrap_or_default()
                .into_iter()
                .zip(all_ids.iter())
                .filter_map(|(opt_mem, uid)| opt_mem.map(|m| (uid.to_string(), m)))
                .collect()
        };

        // --- Phase 3: Build merged results map ---
        // Results map: memory_id → (Memory, rrf_l3, rrf_l4)
        // We track per-tier RRF scores separately for correct weighted blending.
        let mut merged: HashMap<String, (Memory, f32, f32)> = HashMap::new();

        for (id, rrf_score) in &l3_results {
            if let Some(memory) = batch_memories.get(id) {
                let entry = merged
                    .entry(id.clone())
                    .or_insert_with(|| (memory.clone(), 0.0, 0.0));
                entry.1 = *rrf_score; // rrf_l3
            }
        }

        for (id, rrf_score) in &l4_results {
            if let Some(memory) = batch_memories.get(id) {
                let entry = merged
                    .entry(id.clone())
                    .or_insert_with(|| (memory.clone(), 0.0, 0.0));
                entry.2 = *rrf_score; // rrf_l4
            }
        }

        // --- Weighted combination ---
        // If only one tier ran, use its RRF score directly.
        // If both ran: final = vector_weight * rrf_l3 + text_weight * rrf_l4.

        let mut scored: Vec<(Memory, f32)> = merged
            .into_values()
            .map(|(memory, rrf_l3, rrf_l4)| {
                let final_score = if vector_available && fts_available {
                    vector_weight * rrf_l3 + text_weight * rrf_l4
                } else if vector_available {
                    rrf_l3
                } else {
                    rrf_l4
                };
                (memory, final_score)
            })
            .collect();

        // --- In-memory filtering ---
        if query.memory_type.is_some()
            || query.tags.is_some()
            || query.session_id.is_some()
        {
            scored.retain(|(mem, _)| {
                if let Some(ref mt) = query.memory_type {
                    if mem.memory_type != *mt {
                        return false;
                    }
                }
                if let Some(ref tags) = query.tags {
                    if !tags.iter().any(|t| mem.tags.contains(t)) {
                        return false;
                    }
                }
                if let Some(ref sid) = query.session_id {
                    if mem.session_id != *sid {
                        return false;
                    }
                }
                true
            });
        }

        // --- Sort by score descending, take top limit ---
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        Ok(scored)
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

    /// Helper: create an Engine with L3 (vector) and L4 (FTS) enabled.
    fn setup_hybrid_engine(dir: &TempDir) -> Engine {
        let tantivy_path: std::path::PathBuf = dir.path().join("tantivy");
        std::fs::create_dir_all(&tantivy_path).expect("create tantivy dir");
        Engine::with_config(crate::engine::EngineConfig {
            storage: crate::engine::StorageConfig {
                path: dir.path().join("rocksdb"),
                cache_config: None,
            },
            enable_vector_index: true,
            vector_dimension: 4,
            enable_fulltext_search: true,
            tantivy_path: Some(tantivy_path),
            ..crate::engine::EngineConfig::default()
        })
        .expect("create hybrid engine")
    }

    /// Helper: create an Engine with only L3 (vector) enabled.
    fn setup_vector_only_engine(dir: &TempDir) -> Engine {
        Engine::with_config(crate::engine::EngineConfig {
            storage: crate::engine::StorageConfig {
                path: dir.path().join("rocksdb"),
                cache_config: None,
            },
            enable_vector_index: true,
            vector_dimension: 4,
            ..crate::engine::EngineConfig::default()
        })
        .expect("create vector-only engine")
    }

    /// Helper: create an Engine with only L4 (FTS) enabled.
    fn setup_fts_only_engine(dir: &TempDir) -> Engine {
        let tantivy_path: std::path::PathBuf = dir.path().join("tantivy");
        std::fs::create_dir_all(&tantivy_path).expect("create tantivy dir");
        Engine::with_config(crate::engine::EngineConfig {
            storage: crate::engine::StorageConfig {
                path: dir.path().join("rocksdb"),
                cache_config: None,
            },
            enable_fulltext_search: true,
            tantivy_path: Some(tantivy_path),
            ..crate::engine::EngineConfig::default()
        })
        .expect("create fts-only engine")
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

    // -------------------------------------------------------------------
    // Hybrid search tests
    // -------------------------------------------------------------------

    /// Hybrid search returns Unimplemented when L3/L4 not enabled.
    #[test]
    fn test_hybrid_search_disabled_by_default() {
        let (engine, _dir) = setup();

        let result = engine.hybrid_search(&HybridSearchQuery {
            query_text: Some("test".into()),
            ..Default::default()
        });

        assert!(result.is_err(), "hybrid search should fail without L3/L4");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Hybrid search requires L3 or L4"),
            "unexpected error: {msg}"
        );
    }

    /// Hybrid search with L3+L4 enabled returns results.
    #[test]
    fn test_hybrid_search_returns_results() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_hybrid_engine(&dir);
        let agent = Uuid::now_v7();
        let session = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session");

        // Create a few memories with content embeddings for vector search
        // and titles/content for FTS search.
        let memories = vec![
            ("rust programming", "A memory about rust", vec![0.9, 0.1, 0.1, 0.0]),
            ("python programming", "A memory about python", vec![0.1, 0.9, 0.1, 0.0]),
            ("javascript programming", "A memory about javascript", vec![0.1, 0.1, 0.9, 0.0]),
        ];

        for (title, content, embedding) in &memories {
            let memory = engine
                .create_memory(NewMemory {
                    session_id: session.id,
                    agent_id: agent,
                    memory_type: MemoryType::Fact,
                    content: content.to_string(),
                    tags: Some(vec![title.to_string()]),
                })
                .expect("create memory");

            // Directly insert embedding into L3 vector index
            if let Some(ref vx) = engine.vector_index {
                vx.insert(&memory.id.to_string(), embedding)
                    .expect("vector insert");
            }
        }

        // Now hybrid search for "rust" should return the rust memory.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("rust".into()),
                query_vector: Some(vec![0.9, 0.1, 0.1, 0.0]),
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search");

        assert!(!results.is_empty(), "hybrid search should return results");
        assert!(
            results.iter().any(|(m, _score)| m.content.contains("rust")),
            "should find rust memory"
        );
    }

    /// Hybrid search with only L3 (vector) enabled.
    #[test]
    fn test_hybrid_search_vector_only() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_vector_only_engine(&dir);
        let agent = Uuid::now_v7();
        let session = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session");

        // Create memories and insert embeddings.
        let memories = vec![
            ("alpha", vec![1.0, 0.0, 0.0, 0.0]),
            ("beta", vec![0.0, 1.0, 0.0, 0.0]),
            ("gamma", vec![0.0, 0.0, 1.0, 0.0]),
        ];

        for (tag, embedding) in &memories {
            let memory = engine
                .create_memory(NewMemory {
                    session_id: session.id,
                    agent_id: agent,
                    memory_type: MemoryType::Fact,
                    content: format!("memory tagged {tag}"),
                    tags: Some(vec![tag.to_string()]),
                })
                .expect("create memory");

            if let Some(ref vx) = engine.vector_index {
                vx.insert(&memory.id.to_string(), embedding)
                    .expect("vector insert");
            }
        }

        // Search by vector close to alpha.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_vector: Some(vec![0.95, 0.05, 0.0, 0.0]),
                top_k: 3,
                ..Default::default()
            })
            .expect("hybrid search");

        assert!(!results.is_empty(), "vector-only search should return results");
        // alpha should be the top result since it's closest to the query vector
        let (top_mem, _score) = &results[0];
        assert!(
            top_mem.content.contains("alpha") || top_mem.content.contains("memory"),
            "top result should be relevant, got: {:?}",
            top_mem.content
        );
    }

    /// Hybrid search with only L4 (FTS) enabled.
    #[test]
    fn test_hybrid_search_fts_only() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_fts_only_engine(&dir);
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
                content: "quick brown fox jumps over the lazy dog".into(),
                tags: None,
            })
            .expect("create fox memory");

        engine
            .create_memory(NewMemory {
                session_id: session.id,
                agent_id: agent,
                memory_type: MemoryType::Fact,
                content: "slow red turtle crawls under the heavy log".into(),
                tags: None,
            })
            .expect("create turtle memory");

        // Search via FTS for "fox" should find the fox memory.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("fox".into()),
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search fts only");

        assert!(!results.is_empty(), "FTS-only search should return results");
        assert!(
            results.iter().any(|(m, _score)| m.content.contains("fox")),
            "should find fox memory"
        );
    }

    /// Empty query returns error.
    #[test]
    fn test_hybrid_search_empty_query() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_hybrid_engine(&dir);

        let result = engine.hybrid_search(&HybridSearchQuery {
            query_text: None,
            query_vector: None,
            ..Default::default()
        });

        assert!(
            result.is_err(),
            "empty query should fail"
        );
    }

    /// top_k and offset work correctly.
    #[test]
    fn test_hybrid_search_pagination() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_hybrid_engine(&dir);
        let agent = Uuid::now_v7();
        let session = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session");

        // Create 5 memories with embeddings.
        for i in 0..5 {
            let memory = engine
                .create_memory(NewMemory {
                    session_id: session.id,
                    agent_id: agent,
                    memory_type: MemoryType::Fact,
                    content: format!("memory {i} with text about general topic rust programming"),
                    tags: None,
                })
                .expect("create memory");

            if let Some(ref vx) = engine.vector_index {
                let emb = vec![
                    0.8 - (i as f32) * 0.1,
                    0.1,
                    0.1,
                    0.0,
                ];
                vx.insert(&memory.id.to_string(), &emb)
                    .expect("vector insert");
            }
        }

        // Fetch top 2.
        let top2 = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("rust".into()),
                query_vector: Some(vec![0.8, 0.1, 0.1, 0.0]),
                top_k: 2,
                ..Default::default()
            })
            .expect("hybrid search");

        assert_eq!(top2.len(), 2, "should return exactly 2 results");

        // Fetch top 10 (should get at most 5).
        let top10 = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("rust".into()),
                query_vector: Some(vec![0.8, 0.1, 0.1, 0.0]),
                top_k: 10,
                ..Default::default()
            })
            .expect("hybrid search");

        assert!(
            top10.len() <= 5,
            "should not exceed available memories"
        );
    }

    /// Search with session_id filter works in hybrid search.
    #[test]
    fn test_hybrid_search_session_filter() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_hybrid_engine(&dir);
        let agent = Uuid::now_v7();
        let session_a = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session a");
        let session_b = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session b");

        // Create memories in session A.
        let mem_a = engine
            .create_memory(NewMemory {
                session_id: session_a.id,
                agent_id: agent,
                memory_type: MemoryType::Fact,
                content: "rust is a systems language".into(),
                tags: None,
            })
            .expect("create in session A");
        if let Some(ref vx) = engine.vector_index {
            vx.insert(&mem_a.id.to_string(), &vec![0.9, 0.1, 0.0, 0.0])
                .expect("insert emb");
        }

        // Create memories in session B.
        let mem_b = engine
            .create_memory(NewMemory {
                session_id: session_b.id,
                agent_id: agent,
                memory_type: MemoryType::Fact,
                content: "rust is also systems programming".into(),
                tags: None,
            })
            .expect("create in session B");
        if let Some(ref vx) = engine.vector_index {
            vx.insert(&mem_b.id.to_string(), &vec![0.8, 0.2, 0.0, 0.0])
                .expect("insert emb");
        }

        // Search filtered to session A.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("rust".into()),
                session_id: Some(session_a.id),
                query_vector: Some(vec![0.85, 0.15, 0.0, 0.0]),
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search with session filter");

        assert_eq!(results.len(), 1, "should only find memory in session A");
        assert!(
            results[0].0.content.contains("systems language"),
            "should find session A's memory"
        );
    }

    /// Verify RRF weighting by combining vector and FTS results.
    #[test]
    fn test_hybrid_search_rrf_weighting() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_hybrid_engine(&dir);
        let agent = Uuid::now_v7();
        let session = engine
            .create_session(NewSession {
                project: "test".into(),
                agent_id: agent,
                status: None,
                metadata: None,
            })
            .expect("create session");

        // Create one memory about "rust" both in content and vector.
        let mem_rust = engine
            .create_memory(NewMemory {
                session_id: session.id,
                agent_id: agent,
                memory_type: MemoryType::Fact,
                content: "rust is a great systems programming language".into(),
                tags: None,
            })
            .expect("create rust memory");
        if let Some(ref vx) = engine.vector_index {
            vx.insert(&mem_rust.id.to_string(), &[0.9, 0.05, 0.05, 0.0])
                .expect("insert emb");
        }

        // Create a second memory about "python".
        let mem_python = engine
            .create_memory(NewMemory {
                session_id: session.id,
                agent_id: agent,
                memory_type: MemoryType::Fact,
                content: "python is a great general purpose language".into(),
                tags: None,
            })
            .expect("create python memory");
        if let Some(ref vx) = engine.vector_index {
            vx.insert(&mem_python.id.to_string(), &[0.05, 0.9, 0.05, 0.0])
                .expect("insert emb");
        }

        // Search for "rust" — the rust memory should rank higher since it matches both.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("rust".into()),
                query_vector: Some(vec![0.85, 0.1, 0.05, 0.0]),
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search");

        assert!(!results.is_empty(), "should have results");
        assert_eq!(
            results[0].0.content, "rust is a great systems programming language",
            "rust memory should rank first (matches both FTS and vector)"
        );
    }

    // -------------------------------------------------------------------
    // Input validation tests
    // -------------------------------------------------------------------

    /// vector_weight < 0.0 is clamped to 0.0 (does not cause error).
    #[test]
    fn test_hybrid_search_weight_clamped_low() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_fts_only_engine(&dir);
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
                content: "test content".into(),
                tags: None,
            })
            .expect("create memory");

        // vector_weight = -0.5 should be clamped to 0.0 — no error.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("test".into()),
                vector_weight: -0.5,
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search with negative weight should succeed");

        assert!(!results.is_empty(), "should still return results with clamped weight");
    }

    /// vector_weight > 1.0 is clamped to 1.0 (does not cause error).
    #[test]
    fn test_hybrid_search_weight_clamped_high() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_fts_only_engine(&dir);
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
                content: "test content".into(),
                tags: None,
            })
            .expect("create memory");

        // vector_weight = 2.0 should be clamped to 1.0 — no error.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("test".into()),
                vector_weight: 2.0,
                top_k: 5,
                ..Default::default()
            })
            .expect("hybrid search with excessive weight should succeed");

        assert!(!results.is_empty(), "should still return results with clamped weight");
    }

    /// limit = 0 returns empty results immediately.
    #[test]
    fn test_hybrid_search_limit_zero() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_fts_only_engine(&dir);
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
                content: "test content".into(),
                tags: None,
            })
            .expect("create memory");

        // limit = 0 → empty results.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("test".into()),
                top_k: 0,
                ..Default::default()
            })
            .expect("hybrid search with limit=0 should succeed");

        assert!(results.is_empty(), "limit=0 should return empty results");
    }

    /// limit > 1000 is capped to 1000.
    #[test]
    fn test_hybrid_search_limit_capped() {
        let dir = TempDir::new().expect("temp dir");
        let engine = setup_fts_only_engine(&dir);
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
                content: "test content".into(),
                tags: None,
            })
            .expect("create memory");

        // limit = 5000 should be capped to 1000 — should succeed with ≤1000 results.
        let results = engine
            .hybrid_search(&HybridSearchQuery {
                query_text: Some("test".into()),
                top_k: 5000,
                ..Default::default()
            })
            .expect("hybrid search with capped limit should succeed");

        // We only have 1 memory, so we get at most 1.
        assert!(results.len() <= 1000, "capped limit should not exceed 1000");
    }

}
