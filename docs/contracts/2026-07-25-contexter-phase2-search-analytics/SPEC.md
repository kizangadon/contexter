---
title: Contexter Phase 2 — Search & Analytics Engine
version: 1.0
date_created: 2026-07-25
tags: rust, hnsw, tantivy, duckdb, search, analytics
---

# Contexter Phase 2 — Search & Analytics Engine

## 1. Purpose & Scope

This specification covers Phase 2 of the Contexter platform: building the L3 (HNSW vector index), L4 (Tantivy full-text search), and L5 (DuckDB analytics engine) storage tiers on top of the existing Rust core. It includes hybrid search merging L3+L4 results and analytics computation for session efficiency and metric correlation.

**Audience:** Rust backend engineers
**Project:** contexter-core crate

## 2. Definitions

| Term | Definition |
|---|---|
| L3 | Vector index tier — HNSW approximate nearest neighbour search |
| L4 | Full-text search tier — Tantivy BM25 inverted index |
| L5 | Analytics engine tier — DuckDB columnar SQL queries |
| HNSW | Hierarchical Navigable Small World graph for ANN |
| ANN | Approximate Nearest Neighbour |
| BM25 | Best Matching 25 — Lucene-class ranking function |
| Cosine similarity | dot(a,b) / (|a| * |b|) |
| ef_construction | HNSW build parameter: search width during construction |
| ef_search | HNSW query parameter: search width during querying |

## 3. Requirements

### 3.1 L3: HNSW Vector Index

- **REQ-VEC-001**: `VectorIndex` trait with `insert()`, `search()`, `remove()`, `save_snapshot()`, `load_snapshot()` methods
- **REQ-VEC-002**: HNSW implementation with configurable `M=16`, `ef_construction=200`, `ef_search=50`
- **REQ-VEC-003**: Cosine similarity as default distance metric; Euclidean and Dot product as alternatives
- **REQ-VEC-004**: Binary snapshot persistence with magic number, version, dimension/element counts, adjacency list, and embedding vectors
- **REQ-VEC-005**: Auto-snapshot every 1,000 mutations and on graceful shutdown
- **REQ-VEC-006**: On startup, load snapshot from `~/.contexter/vector_index.bin`; rebuild if memory count in L2 doesn't match index entry count
- **REQ-VEC-007**: Support configurable embedding dimensions (default 384)
- **REQ-VEC-008**: Support both in-memory and persisted modes (snapshot optional)
- **REQ-VEC-009**: Implement ID-based removal from the graph

### 3.2 L4: Tantivy Full-Text Search

- **REQ-FTS-001**: `FullTextSearch` trait with `index()`, `search()`, `delete()`, `flush()`, `load()` methods
- **REQ-FTS-002**: Tantivy backend with per-entity-type schema (memory, session, agent, skill content)
- **REQ-FTS-003**: Field-level boosting: content=1.0, title=2.0, tags=1.5
- **REQ-FTS-004**: Query parsing supporting phrase, fuzzy, boolean operators
- **REQ-FTS-005**: Incremental indexing — new documents added on write
- **REQ-FTS-006**: Index directory at `~/.contexter/tantivy_index/`
- **REQ-FTS-007**: Automatic segment merging (Tantivy default)

### 3.3 L5: DuckDB Analytics Engine

- **REQ-ANA-001**: `AnalyticsEngine` trait with `query()`, `sync()`, `sync_all()` methods
- **REQ-ANA-002**: DuckDB backend with in-memory tables populated on demand from RocksDB iterators
- **REQ-ANA-003**: Predefined SQL queries for: session count by time range, memory count by type, telemetry aggregation, efficiency scores
- **REQ-ANA-004**: On-demand sync — data materialized into DuckDB in-memory tables when analytics request arrives
- **REQ-ANA-005**: Configurable cache TTL for analytics results (default: 300s)

### 3.4 Hybrid Search

- **REQ-HYB-001**: `hybrid_search()` on `Engine` that merges L3 (HNSW) + L4 (Tantivy) results
- **REQ-HYB-002**: Configurable weighting between vector score and BM25 score (default: 0.5 vector / 0.5 text)
- **REQ-HYB-003**: Reciprocal Rank Fusion (RRF) merge strategy with k=60
- **REQ-HYB-004**: Return deduplicated, reranked results with score annotations
- **REQ-HYB-005**: Apply existing filter criteria (memory_type, tags, session_id) to hybrid results

### 3.5 Efficiency & Correlation

- **REQ-EFF-001**: Compute session efficiency score = (useful memories / total memories) per session
- **REQ-EFF-002**: Compute metric correlation (Pearson) between session duration and memory count
- **REQ-EFF-003**: Store computed efficiency scores in `efficiency_map` column family
- **REQ-EFF-004**: Cache efficiency results with per-session granularity

### 3.6 Engine Integration

- **REQ-ENG-001**: `Engine` struct composes `VectorIndex`, `FullTextSearch`, and `AnalyticsEngine` as `Option<Arc<>>` fields (graceful degradation if tier not configured)
- **REQ-ENG-002`: L3 updated on memory write (insert vector); L4 indexed on memory write
- **REQ-ENG-003**: L5 analytics accessible via `Engine::run_analytics()` method (replaces current Unimplemented stub)
- **REQ-ENG-004**: All tiers are disabled by default (Engine opens without L3/L4/L5); enabled via `EngineConfig` or environment
- **REQ-ENG-005**: Engine passes correct `StorageBackend` reference to L5 for RocksDB→DuckDB sync

## 4. Constraints

- **CON-001**: No external processes — all tiers run in-process
- **CON-002**: L3 snapshot must be backward-compatible with version field
- **CON-003**: L5 is ephemeral — data is never persisted in DuckDB; always synced from RocksDB
- **CON-004**: Hybrid search must not degrade non-hybrid `search_memories()` performance
- **CON-005**: Tantivy index directory must be created if absent

## 5. Interfaces

### VectorIndex trait (proposed)
```rust
pub trait VectorIndex: Send + Sync {
    fn insert(&self, id: &str, vector: &[f32]) -> Result<()>;
    fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>>;
    fn remove(&self, id: &str) -> Result<()>;
    fn save_snapshot(&self, path: &Path) -> Result<()>;
    fn load_snapshot(&self, path: &Path) -> Result<usize>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}
```

### FullTextSearch trait (proposed)
```rust
pub trait FullTextSearch: Send + Sync {
    fn index(&self, doc_id: &str, fields: &[FieldValue]) -> Result<()>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>>;
    fn delete(&self, doc_id: &str) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
```

### AnalyticsEngine trait (proposed)
```rust
pub trait AnalyticsEngine: Send + Sync {
    fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Vec<Value>>>;
    fn sync(&self, cf_name: &str) -> Result<()>;
    fn sync_all(&self) -> Result<()>;
}
```

## 6. Dependencies

| Crate | Purpose | Version Notes |
|---|---|---|
| `instant-distance` or `voyager` | HNSW vector index | Pure Rust, no C deps |
| `tantivy` | Full-text search engine | v0.22+, Lucene-class BM25 |
| `duckdb` / `duckdb-rs` | OLAP analytics engine | via `libduckdb-sys` |

## 7. Acceptance Criteria

- **AC-VEC-001**: Vector index can insert 10,000 embeddings and search returns top-K results within 50ms
- **AC-VEC-002**: Snapshot round-trip: save → load → search returns same results
- **AC-VEC-003**: Auto-rebuild: delete snapshot → restart → index auto-rebuilds from L2
- **AC-FTS-001**: Full-text search returns relevant documents (BM25 scoring) for keyword queries
- **AC-FTS-002**: Index survives process restart (persistent directory)
- **AC-FTS-003**: Fuzzy and phrase queries work on indexed content
- **AC-ANA-001**: Analytics query returns aggregated counts from RocksDB data
- **AC-ANA-002**: Multiple analytics queries can run sequentially with different time ranges
- **AC-HYB-001**: Hybrid search returns merged, deduplicated results from L3+L4
- **AC-HYB-002**: Hybrid search with weight=[1.0, 0.0] returns only L3 results
- **AC-HYB-003**: Hybrid search with weight=[0.0, 1.0] returns only L4 results
- **AC-ENG-001**: Engine opens with all tiers disabled (backward compatible)
- **AC-ENG-002**: Engine opens with L3 enabled via config
- **AC-ENG-003**: `cargo build --workspace` and `cargo test --workspace` pass
