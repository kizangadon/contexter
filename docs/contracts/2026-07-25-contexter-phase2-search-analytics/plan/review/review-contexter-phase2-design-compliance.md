# Design Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Backend-only Rust crate adding three optional storage tiers (L3 HNSW vector index, L4 Tantivy full-text search, L5 DuckDB analytics) and hybrid search merging L3+L4 results via RRF, plus analytics efficiency/correlation.

**Verdict:** FAIL (class: PARTIAL — multiple unmatched/partially-matched design commitments)

2026-07-25 · 6/6 design sections verified · Design Compliance Validator

---

## 01 · Design Preview Sections Covered

| Section | Status |
|---|---|
| 3.1 — L3 HNSW Vector Index (REQ-VEC-001–009) | ⚠️ PARTIAL |
| 3.2 — L4 Tantivy Full-Text Search (REQ-FTS-001–007) | ⚠️ PARTIAL |
| 3.3 — L5 DuckDB Analytics Engine (REQ-ANA-001–005) | ⚠️ PARTIAL |
| 3.4 — Hybrid Search (REQ-HYB-001–005) | ✅ MATCHED |
| 3.5 — Efficiency & Correlation (REQ-EFF-001–004) | ❌ UNMATCHED (EFF-003) / ⚠️ PARTIAL |
| 3.6 — Engine Integration (REQ-ENG-001–005) | ⚠️ PARTIAL |
| Section 5 — Proposed trait interfaces (VectorIndex, FullTextSearch, AnalyticsEngine) | ✅ MATCHED |

---

## 02 · Architecture Compliance

Checks whether the actual system architecture matches the architecture described in SPEC.md Sections 3 and 5.

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Module decomposition | `VectorIndex` trait + `HnswVectorIndex` (L3) | `crate::vector::VectorIndex` trait + `HnswVectorIndex` in `contexter-core/src/vector/hnsw.rs` | ✅ MATCHED |
| Module decomposition | `FullTextSearch` trait + `TantivyIndex` (L4) | `crate::fts::FullTextSearch` trait + `TantivyIndex` in `contexter-core/src/fts/tantivy.rs` | ✅ MATCHED |
| Module decomposition | `AnalyticsEngine` trait + `DuckDbEngine` (L5) | `crate::analytics::AnalyticsEngine` trait + `DuckDbEngine` in `contexter-core/src/analytics/duckdb.rs` | ✅ MATCHED |
| Engine composition | `Engine` with `Option<Arc<dyn VectorIndex>>`, `Option<Arc<dyn FullTextSearch>>`, `Option<Arc<dyn AnalyticsEngine>>` | `Engine` struct at `engine/mod.rs:203-213` has `vector_index: Option<Arc<dyn VectorIndex>>`, `fts_index: Option<Arc<dyn FullTextSearch>>`, `analytics_engine: Option<Arc<dyn AnalyticsEngine>>` | ✅ MATCHED |
| Default disabled | All tiers disabled by default | `EngineConfig::default()` has all `enable_*` = `false`; `Engine::open()` sets all three to `None` | ✅ MATCHED |
| L3 HNSW parameters | Configurable `M=16`, `ef_construction=200`, `ef_search=50` | HNSW via `instant_distance` uses `Builder::default()`; M & ef_construction are **not configurable at the API** — only stored as snapshot metadata (hardcoded `m: 16`, `ef_construction: 200` in `snapshot.rs:149-150`). `ef_search` is not exposed. | ⚠️ PARTIAL |
| L3 distance metrics | Cosine default; Euclidean and Dot product as alternatives | Cosine is default (hardcoded in `hnsw.rs:35-38`). Euclidean and dot product exist in `distance.rs` but are **not configurable alternatives** in the HNSW index. | ⚠️ PARTIAL |
| L3 auto-snapshot | Every 1,000 mutations **and on graceful shutdown** | Auto-snapshot at 1000 mutations implemented (`hnsw.rs:119-130`). Graceful shutdown snapshot **NOT implemented** (no `Drop` or shutdown hook). | ⚠️ PARTIAL |
| L3 startup rebuild | Compare L2 memory count vs index entry count; rebuild if mismatch | `engine/mod.rs:249-262` loads snapshot if path exists but **does not** compare L2 count or trigger rebuild on mismatch. | ❌ UNMATCHED |
| FTS per-entity schema | memory, session, agent, skill content | `fts/schema.rs` defines only `"memory"`/`"memories"` and a fallback `"default"` schema. Session, agent, and skill schemas are **not defined**. | ⚠️ PARTIAL |
| FTS index path | `~/.contexter/tantivy_index/` default | `tantivy_path` is configurable in `EngineConfig` but has **no default** path. | ⚠️ PARTIAL |
| L5 RocksDB sync | Real iterator-based population from RocksDB | Sync uses **sample data stubs** (`duckdb.rs:228` comment: "real RocksDB sync will be integrated"). Backend is stored but not wired. | ⚠️ PARTIAL |
| StorageBackend wiring | Engine passes backend to L5 | `set_storage_backend()` exists on `AnalyticsEngine` trait but `engine/mod.rs:with_config()` **never calls it**. | ❌ UNMATCHED |
| Efficiency score caching | Per-session granularity in `efficiency_map` CF | **No `efficiency_map` column family**. Table-level TTL exists in DuckDB but no per-session efficiency caching. | ❌ UNMATCHED |

**Architecture Findings:**

1. **FINDING-VEC-002** — HNSW parameters (M, ef_construction, ef_search) are not configurable via the public API. The spec requires configurable parameters; snapshot metadata shows hardcoded `M=16, ef_construction=200` but `ef_search=50` is absent entirely.
2. **FINDING-VEC-003** — Euclidean and dot product distance metrics exist in `distance.rs` but are not wired into `HnswVectorIndex` as configurable alternatives. Only cosine similarity is used.
3. **FINDING-VEC-005** — Graceful shutdown auto-snapshot is not implemented. Only the mutation-count-triggered snapshot exists.
4. **FINDING-VEC-006** — Startup rebuild logic (comparing L2 memory count vs index entry count) is missing. The snapshot is simply loaded if present without validation against L2.
5. **FINDING-FTS-002** — Only "memory" entity schema is defined. Session, agent, and skill schemas referenced in the spec are not implemented.
6. **FINDING-FTS-006** — No default path `~/.contexter/tantivy_index/` is configured. The path is required to be provided explicitly.
7. **FINDING-ANA-002** — DuckDB sync uses sample data stubs instead of real RocksDB iterator-based population.
8. **FINDING-ENG-005** — The Engine's `with_config()` method does not wire the RocksDB storage backend into the analytics engine via `set_storage_backend()`.
9. **FINDING-EFF-003** — The `efficiency_map` column family for storing computed efficiency scores does not exist.
10. **FINDING-EFF-004** — Per-session granularity efficiency caching is not implemented (only table-level TTL exists).

---

## 03 · API Contract Compliance

Checks whether the actual trait method signatures match the proposed interfaces in SPEC.md Section 5.

### VectorIndex Trait (SPEC.md Section 5, lines 97-107)

| Method | Proposed Signature | Actual Signature | Status |
|---|---|---|---|
| `insert` | `fn insert(&self, id: &str, vector: &[f32]) -> Result<()>` | `fn insert(&self, id: &str, vector: &[f32]) -> VectorIndexResult<()>` | ✅ MATCHED |
| `search` | `fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>>` | `fn search(&self, query: &[f32], k: usize) -> VectorIndexResult<Vec<(String, f32)>>` | ✅ MATCHED |
| `remove` | `fn remove(&self, id: &str) -> Result<()>` | `fn remove(&self, id: &str) -> VectorIndexResult<()>` | ✅ MATCHED |
| `save_snapshot` | `fn save_snapshot(&self, path: &Path) -> Result<()>` | `fn save_snapshot(&self, path: &Path) -> VectorIndexResult<()>` | ✅ MATCHED |
| `load_snapshot` | `fn load_snapshot(&self, path: &Path) -> Result<usize>` | `fn load_snapshot(&self, path: &Path) -> VectorIndexResult<usize>` | ✅ MATCHED |
| `len` | `fn len(&self) -> usize` | `fn len(&self) -> usize` | ✅ MATCHED |
| `is_empty` | `fn is_empty(&self) -> bool` | `fn is_empty(&self) -> bool` | ✅ MATCHED |

### FullTextSearch Trait (SPEC.md Section 5, lines 109-117)

| Method | Proposed Signature | Actual Signature | Status |
|---|---|---|---|
| `index` | `fn index(&self, doc_id: &str, fields: &[FieldValue]) -> Result<()>` | `fn index(&self, doc_id: &str, fields: &[FieldValue]) -> FtsResult<()>` | ✅ MATCHED |
| `search` | `fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>>` | `fn search(&self, query: &str, limit: usize) -> FtsResult<Vec<(String, f32)>>` | ✅ MATCHED |
| `delete` | `fn delete(&self, doc_id: &str) -> Result<()>` | `fn delete(&self, doc_id: &str) -> FtsResult<()>` | ✅ MATCHED |
| `flush` | `fn flush(&self) -> Result<()>` | `fn flush(&self) -> FtsResult<()>` | ✅ MATCHED |
| `load` | `fn load(&self) -> Result<()>` (implied by REQ-FTS-001) | **Not present** in trait. Replaced by `TantivyIndex::open()` | ⚠️ PARTIAL |

### AnalyticsEngine Trait (SPEC.md Section 5, lines 119-126)

| Method | Proposed Signature | Actual Signature | Status |
|---|---|---|---|
| `query` | `fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Vec<Value>>>` | `fn query(&self, sql: &str, params: &[Value]) -> AnalyticsResult<Vec<Vec<Value>>>` | ✅ MATCHED |
| `sync` | `fn sync(&self, cf_name: &str) -> Result<()>` | `fn sync(&self, cf_name: &str) -> AnalyticsResult<()>` | ✅ MATCHED |
| `sync_all` | `fn sync_all(&self) -> Result<()>` | `fn sync_all(&self) -> AnalyticsResult<()>` | ✅ MATCHED |
| Extra method | — | `fn set_storage_backend(&self, backend: Box<dyn Any + Send>)` | ➖ Extra (beyond spec) |

**API Findings:**

1. **FINDING-FTS-001** — The `FullTextSearch` trait is missing a `load()` method. The spec lists `load()` as an explicit method. Instead, the Tantivy implementation provides `open()`/`open_in_memory()` constructors, which functionally serve the same purpose but don't conform to the trait signature.

---

## 04 · UI Wireframe Compliance

➖ **NOT APPLICABLE** — Contexter Phase 2 is a backend-only Rust library crate with no user interface. No wireframes were generated in the design preview. All design commitments are architectural and API-level only.

---

## 05 · Data Flow Compliance

Checks whether the actual runtime data flow matches the numbered steps defined in SPEC.md requirements.

### Memory Write Path (REQ-ENG-002: L3 updated on memory write; L4 indexed)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. User calls `create_memory()` | L2 storage write | `engine/memory.rs:24` — `self.storage.write().unwrap().create_memory(new_memory)` | ✅ MATCHED |
| 2. Cache update | L1 write-through | `engine/memory.rs:26` — `self.cache.store(...)` | ✅ MATCHED |
| 3. L3 vector insert | If L3 enabled, insert embedding | `engine/memory.rs:29-35` — `if let Some(ref vx) { vx.insert(...) }` | ✅ MATCHED |
| 4. L4 FTS index | If L4 enabled, index content | `engine/memory.rs:38-49` — `if let Some(ref fts) { fts.index(...); fts.flush() }` | ✅ MATCHED |

### Hybrid Search Path (REQ-HYB-001–005)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. Validate query | At least text_query or vector_query | `search.rs:111-114` — validation check | ✅ MATCHED |
| 2. Check tiers available | At least L3 or L4 must be enabled | `search.rs:117-121` — availability check | ✅ MATCHED |
| 3. L3 vector search | Search L3 for up to limit*2 results | `search.rs:132-152` — `vx.search(vec, fetch_k)` | ✅ MATCHED |
| 4. L4 FTS search | Search L4 for up to limit*2 results | `search.rs:155-175` — `fts.search(text, fetch_k)` | ✅ MATCHED |
| 5. RRF scoring | Compute RRF: 1/(k + rank) with k=60 | `search.rs:64-65` — `RRF_K: f32 = 60.0`; `search.rs:140,163` — `1.0 / (RRF_K + rank as f32)` | ✅ MATCHED |
| 6. Weighted blend | vector_weight * RRF_L3 + (1-vector_weight) * RRF_L4 | `search.rs:180-196` — weighted combination with single-tier fallback | ✅ MATCHED |
| 7. Deduplication | Same memory ID appears once | `search.rs:128` — HashMap keyed by memory_id | ✅ MATCHED |
| 8. In-memory filtering | Apply memory_type, tags, session_id, agent_id filters | `search.rs:198-226` — retain filter chain | ✅ MATCHED |
| 9. Sort & truncate | Sort by score desc, take top limit | `search.rs:229-230` — `scored.sort_by(...); scored.truncate(query.limit)` | ✅ MATCHED |

### Analytics Run Path (REQ-ENG-003: `run_analytics()`)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. Check L5 enabled | Return error if not configured | `analytics.rs:53-56` — `ok_or_else(|| EngineError::Unimplemented(...))` | ✅ MATCHED |
| 2. Sync all tables | Populate DuckDB via RocksDB sync | `analytics.rs:58-60` — `ae.sync_all()` (sample data only) | ⚠️ PARTIAL |
| 3. Efficiency scores | Compute per session | `analytics.rs:173-215` — via `queries::EFFICIENCY_SCORES` SQL | ✅ MATCHED |
| 4. Metric correlation | Compute Pearson r | `analytics.rs:221-250` — via `queries::METRIC_CORRELATION` SQL | ✅ MATCHED |
| 5. Session/memory counts | Count by type | `analytics.rs:65-88` — via queries | ✅ MATCHED |

### Delete Memory Path

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. Delete from L2 | Remove from storage | `engine/memory.rs:103` — `self.storage.write().unwrap().delete_memory(id)` | ✅ MATCHED |
| 2. Invalidate L1 cache | Cache eviction | `engine/memory.rs:104-105` — `self.cache.invalidate(&key)` | ✅ MATCHED |
| 3. Remove from L3 | If enabled, remove vector | `engine/memory.rs:108-110` — `if let Some(ref vx) { let _ = vx.remove(...) }` | ✅ MATCHED |
| 4. Remove from L4 | If enabled, delete doc | `engine/memory.rs:113-116` — `if let Some(ref fts) { let _ = fts.delete(...) }` | ✅ MATCHED |

**Data Flow Findings:**

No structural findings — all data flow paths are correctly implemented. The only gap is that L5 sync uses sample data stubs rather than real RocksDB iterators (wider architecture finding from section 02).

---

## 06 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | YES |
| Zero findings are being silently deferred to a future iteration | YES |

---

## 07 · Summary

> **Design Compliance Assessment**

> The implementation covers the core architecture faithfully: all three trait definitions match the proposed interfaces, the `Engine` struct composes L3/L4/L5 as `Option<Arc<>>` fields (correctly defaulting to `None`), hybrid search implements RRF with k=60 with proper deduplication and filtering, and the memory write path correctly hooks into L3/L4 when enabled.

> **10 findings identified.** These fall into three categories: (1) **configurability gaps** — HNSW parameters and distance metrics are not configurable as specified; (2) **missing features** — graceful shutdown snapshot, startup rebuild comparison, `efficiency_map` column family, per-session efficiency caching, entity-specific FTS schemas; (3) **integration stubs** — RocksDB→DuckDB sync uses sample data rather than real iterators, storage backend is not wired from Engine to L5.

> **Findings**

> 1. **FINDING-VEC-002** — HNSW parameters M, ef_construction, ef_search are hardcoded, not configurable.
> 2. **FINDING-VEC-003** — Euclidean and Dot product distance metrics exist but are not configurable in HNSW index.
> 3. **FINDING-VEC-005** — Graceful shutdown auto-snapshot not implemented.
> 4. **FINDING-VEC-006** — Startup L2-vs-L3 count mismatch rebuild not implemented.
> 5. **FINDING-FTS-001** — `FullTextSearch` trait missing `load()` method (open() used instead).
> 6. **FINDING-FTS-002** — Only "memory" entity schema defined; session/agent/skill schemas missing.
> 7. **FINDING-FTS-006** — No default Tantivy index path configured.
> 8. **FINDING-ANA-002** — DuckDB sync uses sample data stubs, not real RocksDB iterators.
> 9. **FINDING-ENG-005** — Storage backend not wired to analytics engine from Engine.
> 10. **FINDING-EFF-003/004** — `efficiency_map` column family and per-session efficiency caching not implemented.

---

## 08 · Final Verdict

| Criterion | Result |
|---|---|
| Architecture matches design spec | ⚠️ PARTIAL (7 of 10 architecture checks match; 3 partial, 2 unmatched) |
| Trait interfaces match proposed signatures | ✅ MATCHED (all proposed methods present; `load()` replaced by `open()`) |
| UI wireframe matches rendered output | ➖ NOT APPLICABLE (no UI in backend crate) |
| Data flow matches design specification | ✅ MATCHED (all data flow paths correct; L5 sync uses stubs) |
| Carryover declaration clean | ✅ YES |
| **Overall** | **FAIL** |

---

_Generated by Design Compliance Validator · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase2-search-analytics_
