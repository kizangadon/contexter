---
title: Phase 2 — Search & Analytics Acceptance Criteria
version: 1.0
date_created: 2026-07-25
---

# Acceptance Criteria — Phase 2: Search & Analytics Engine

## Feature Context

Contexter Phase 2 adds three storage tiers (L3 HNSW vector index, L4 Tantivy full-text search, L5 DuckDB analytics) and hybrid search merging L3+L4 results. All tiers are optional and disabled by default.

---

## Happy Path

### L3: HNSW Vector Index

**AC-VEC-H1:** Insert embeddings into an empty index, search for nearest neighbours.
- Given an empty VectorIndex with dimension=384
- When I insert 100 embeddings with IDs and random unit vectors
- Then `search()` with a query vector returns top-K results with similarity scores in descending order
- And `len()` equals 100

**AC-VEC-H2:** Snapshot round-trip persists and restores index exactly.
- Given a VectorIndex with 100 embeddings inserted
- When I call `save_snapshot("/tmp/test_idx.bin")`
- Then `load_snapshot("/tmp/test_idx.bin")` on a new index restores all 100 embeddings
- And `search()` on the restored index returns the same top-5 as before the save

**AC-VEC-H3:** Auto-snapshot triggers after 1,000 mutations.
- Given a VectorIndex configured with auto_snapshot_path
- When I insert 1,000 embeddings
- Then the snapshot file exists on disk

### L4: Tantivy Full-Text Search

**AC-FTS-H1:** Index a document and search by keyword.
- Given a FullTextSearch index at a temp directory
- When I index a document with title="rust memory" and content="Contexter stores memories in RocksDB"
- Then `search("rust")` returns the document with score > 0.0
- And `search("python")` returns empty results

**AC-FTS-H2:** Index persistence across restart.
- Given a FullTextSearch index with 10 documents indexed
- When I drop the index and create a new FullTextSearch pointing to the same directory
- Then `search("keyword")` returns the same top result

**AC-FTS-H3:** Phrase query support.
- Given an index with documents containing known phrases
- When I search with a phrase query in quotes
- Then only documents containing the exact phrase appear in results

### L5: DuckDB Analytics

**AC-ANA-H1:** Sync data and run aggregate query.
- Given an AnalyticsEngine connected to the RocksDB store
- When I call `sync("telemetry")` to materialize telemetry records
- Then `query("SELECT COUNT(*) FROM telemetry")` returns the correct row count
- And `query("SELECT AVG(event_count) FROM telemetry")` returns a valid float

**AC-ANA-H2:** Multiple queries with different filters.
- Given synced analytics data covering 7 days
- When I query with time range filter
- Then results reflect only records within that range

### Hybrid Search

**AC-HYB-H1:** Merged L3+L4 results with RRF.
- Given an Engine with L3 and L4 enabled, and memories that match both vector-similar and keyword queries
- When I call `hybrid_search(query_text, query_vector, k=10)`
- Then results include entries from both tiers
- And results are deduplicated (same memory ID appears once)
- And results are sorted by combined score

**AC-HYB-H2:** Pure L3 mode.
- Given hybrid search configured with weight=[1.0, 0.0]
- When I call `hybrid_search(...)`
- Then results contain only L3 matches (zero L4 contribution)

**AC-HYB-H3:** Pure L4 mode.
- Given hybrid search configured with weight=[0.0, 1.0]
- When I call `hybrid_search(...)`
- Then results contain only L4 matches (zero L3 contribution)

### Efficiency & Correlation

**AC-EFF-H1:** Session efficiency score.
- Given analytics with session telemetry data
- When I query efficiency for a session with 10 total and 8 useful memories
- Then the efficiency score is 0.8

**AC-EFF-H2:** Metric correlation.
- Given session telemetry with duration and memory count pairs
- When I compute correlation
- Then the correlation coefficient is between -1.0 and 1.0 inclusive
- And the count of pairs matches the number of sessions

### Engine Integration

**AC-ENG-H1:** Backward compatible — all tiers disabled by default.
- Given the default EngineConfig
- When I call `Engine::new(config, backend)`
- Then no L3/L4/L5 paths are loaded
- And existing `search_memories()`, `store()`, `get_memory()` all work without L3/L4/L5

**AC-ENG-H2:** L3 enabled via config.
- Given EngineConfig with `vector_index = true`
- When I call `Engine::new(config, backend)`
- Then HNSW index is initialised (empty)
- And inserting a memory also updates the vector index

**AC-ENG-H3:** Build and test pass.
- Given the contexter workspace
- When I run `cargo build --workspace`
- Then it compiles without errors
- And `cargo test --workspace` passes all tests

---

## Edge Cases & Error States

**AC-VEC-E1:** Search on empty index.
- Given a VectorIndex with no embeddings
- When I call `search(query, 10)`
- Then an empty Vec is returned (no error)

**AC-VEC-E2:** Remove nonexistent ID.
- Given a VectorIndex with 10 embeddings
- When I call `remove("nonexistent")`
- Then the call succeeds (no-op)
- And `len()` remains 10

**AC-VEC-E3:** Insert with wrong dimension.
- Given a VectorIndex dimension=384
- When I call `insert("id", &vec![0.0; 128])`
- Then an error is returned with dimension mismatch message

**AC-VEC-E4:** Load corrupt snapshot.
- Given a truncated/empty snapshot file
- When I call `load_snapshot(path)`
- Then an error is returned (not a panic)

**AC-VEC-E5:** Search with k=0.
- Given a VectorIndex with embeddings
- When I call `search(query, 0)`
- Then an empty Vec is returned or error handled gracefully (no panic)

**AC-FTS-E1:** Search on empty index.
- Given a FullTextSearch with no documents
- When I call `search("anything", 10)`
- Then an empty Vec is returned

**AC-FTS-E2:** Delete nonexistent document.
- Given a FullTextSearch index
- When I call `delete("nonexistent")`
- Then the call succeeds (no-op)

**AC-FTS-E3:** Index directory creation fails.
- Given a FullTextSearch configured with a path in a read-only directory
- When I initialise the index
- Then an error is returned (not a panic)

**AC-ANA-E1:** Query on unsynced table.
- Given an AnalyticsEngine with no data synced
- When I call `query("SELECT * FROM telemetry")`
- Then an error is returned indicating the table does not exist

**AC-ANA-E2:** Sync on nonexistent column family.
- Given an AnalyticsEngine
- When I call `sync("nonexistent")`
- Then an error is returned

**AC-ENG-E1:** Engine config with invalid dimension.
- Given EngineConfig with vector_index=true and embedding_dim=0
- When I call `Engine::new(config, backend)`
- Then an error is returned: dimension must be positive
