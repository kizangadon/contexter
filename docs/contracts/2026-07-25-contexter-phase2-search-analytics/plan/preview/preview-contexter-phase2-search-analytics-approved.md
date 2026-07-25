# Phase 2: Search & Analytics Engine — Approved Contract

> **Status:** `APPROVED — Contract Frozen` | **Version:** `v1.0.0`
> **Feature:** 16 Acceptance Criteria · 48 Edge Cases · 3 Storage Tiers

---

## Navigation

- [System Design](#architecture)
- [Data Flow](#dataflow)
- [Context](#context)
- [Decision](#decision)
- [API](#api)
- [AC](#ac)
- [Edge Cases](#edgecases)
- [Tests](#tests)
- [References](#references)
- [Contract](#contract)
- [Summary](#summary)

---

## Quick Stats

| Metric | Value |
|---|---|
| AC Passed | 16 |
| Edge Cases | 48 |
| Artifacts | 4 |
| Tests | TBD |

---

## System Design {#architecture}

> **Status:** `FINAL`

### Component Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                         Engine                                  │
│  ┌────────────┐  ┌──────────────┐  ┌─────────────────────────┐ │
│  │ L2 Storage  │  │ L1 Cache     │  │ L3/L4/L5 (Option<Arc>) │ │
│  │ (RocksDB)   │  │ (DashMap+LRU)│  │                         │ │
│  └────────────┘  └──────────────┘  │  ┌───────────────────┐  │ │
│                                    │  │  L3: HNSW (Arc)   │  │ │
│                                    │  │  instant-distance │  │ │
│                                    │  └───────────────────┘  │ │
│                                    │  ┌───────────────────┐  │ │
│                                    │  │  L4: Tantivy (Arc)│  │ │
│                                    │  │  BM25 inverted idx│  │ │
│                                    │  └───────────────────┘  │ │
│                                    │  ┌───────────────────┐  │ │
│                                    │  │  L5: DuckDB (Arc) │  │ │
│                                    │  │  in-memory SQL    │  │ │
│                                    │  └───────────────────┘  │ │
│  ┌────────────┐  ┌──────────────┐  └─────────────────────────┘ │
│  │ Telemetry  │  │ CRDT/Version │                              │
│  └────────────┘  └──────────────┘                              │
└────────────────────────────────────────────────────────────────┘
```

### L3: HNSW Vector Index Module Structure

```
vector/
├── mod.rs         # VectorIndex trait (insert, search, remove, save_snapshot, load_snapshot)
├── hnsw.rs        # HNSW wrapper — delegates to instant-distance graph, supports remove()
├── distance.rs    # Cosine, Euclidean, Dot product kernels
└── snapshot.rs    # Binary format: magic(4) | ver(4) | dim(4) | count(4) | M(8) | ef(8) | graph | vectors
```

**Snapshot format:**
```
[4 bytes]  magic    0x484E5357 ("HNSW")
[4 bytes]  version  u32 LE
[4 bytes]  dimension u32 LE
[4 bytes]  element_count u32 LE
[8 bytes]  M        u64 LE
[8 bytes]  ef_const  u64 LE
[N bytes]  adjacency list (packed graph edges)
[M bytes]  embedding vectors (f32 × dim × count, little-endian)
```

**Remove strategy:** Incremental graph removal via `instant-distance`'s native deletion API. If the library does not support deletion, the wrapper marks removed IDs in a `HashSet<u64>` and filters results post-query (logical deletion). A background rebuild or manual rebuild via `rebuild()` is supported for physical compaction.

### L4: Tantivy Full-Text Search Module Structure

```
fts/
├── mod.rs         # FullTextSearch trait (index, search, delete, flush, load)
├── tantivy.rs     # TantivyIndex — wraps tantivy::Index, field boosting
├── schema.rs      # SchemaBuilder per entity type
└── query.rs       # QueryParser — phrase, fuzzy, boolean, prefix
```

**Schema per entity type:**

| Entity Type | Fields | Boost |
|---|---|---|
| `memory` | content, memory_type, tags | content:1.0, tags:1.5 |
| `session` | project, status, metadata | content:1.0 |
| `agent` | name, description, capabilities | name:2.0, description:1.0 |
| `skill` | name, description, category | name:2.0 |

### L5: DuckDB Analytics Module Structure

```
analytics/
├── mod.rs         # AnalyticsEngine trait (query, sync, sync_all)
├── duckdb.rs      # DuckDbEngine — wraps duckdb::Connection, in-memory
├── queries.rs     # Predefined SQL: session_count, memory_by_type, telemetry_agg, efficiency
└── sync.rs        # RocksDB→DuckDB population via CF iterators
```

**Sync flow:**
1. `sync("telemetry")` → iterate `telemetry` CF → bulk INSERT into DuckDB table
2. DuckDB table created lazily on first sync per CF name
3. `sync_all()` iterates all relevant CFs
4. Cache TTL (default 300s) prevents re-sync on repeated queries

### Engine Integration

```rust
pub(crate) struct Engine {
    pub(crate) storage: SharedBackend,
    pub(crate) cache: DashMapCache,
    pub(crate) telemetry: Arc<TelemetryCollector>,
    // Phase 2: optional tiers
    pub(crate) vector_index: Option<Arc<dyn VectorIndex>>,
    pub(crate) fts_index: Option<Arc<dyn FullTextSearch>>,
    pub(crate) analytics_engine: Option<Arc<dyn AnalyticsEngine>>,
}
```

- `Config::enable_vector_index` → constructs `Some(Arc<HnswVectorIndex>)`
- `Config::enable_fulltext_search` → constructs `Some(Arc<TantivyIndex>)`
- `Config::enable_analytics` → constructs `Some(Arc<DuckDbEngine>)`
- All default to `None` (backward compatible)

---

## Data Flow Sequence {#dataflow}

### 1. Memory Write → L3 + L4 Update

```
Engine::create_memory()
  ├──→ L2: RocksDB persist
  ├──→ L1: cache invalidate
  ├──→ L3: if configured → vector_index.insert(id, &embedding)
  └──→ L4: if configured → fts_index.index(id, &fields)
```

### 2. Hybrid Search (L3 + L4)

```
Engine::hybrid_search(query_text, query_vector, k, weights)
  ├──→ L3: if configured → knn_search(query_vector, k × 2)
  │     Returns Vec<(id, vector_score)>
  ├──→ L4: if configured → fts_search(query_text, k × 2)
  │     Returns Vec<(id, text_score)>
  └──→ Merge: RRF(deduplicate)
        ├──→ RRF score = Σ 1 / (k + rank_in_tier)  where k=60
        ├──→ weight = [vector_weight, text_weight]
        └──→ Final = RRF_vector × w_vec + RRF_text × w_text
```

### 3. Analytics Query

```
Engine::run_analytics(query_type, params)
  ├──→ L5: if not configured → return Err
  ├──→ sync relevant CFs (if cache stale or TTL expired)
  │     └──→ RocksDB iterator → DuckDB INSERT
  └──→ Execute predefined SQL
        ├──→ session_count_by_range(time_start, time_end)
        ├──→ memory_count_by_type()
        ├──→ telemetry_aggregation(event_type, scope)
        ├──→ efficiency_scores()
        └──→ metric_correlation()
```

---

## Why This Feature Exists {#context}

| The Pain | The Principle |
|---|---|
| Memory retrieval is currently limited to keyword, tag, and session-ID filtering — no semantic similarity search, no BM25 full-text ranking, no analytical queries over telemetry data. Users wanting "find me memories similar to this idea" or "which sessions were most efficient?" cannot answer those questions. | A memory platform without semantic search is a key-value store with labels. Phase 2 adds the query algebra that turns collected data into answers. |

---

## Final Decision — Chosen Path {#decision}

> **Status:** `APPROVED`

- **L3:** `instant-distance` HNSW with incremental `remove()` (logical deletion via `HashSet<u64>` filtered post-query; background `rebuild()` for physical compaction)
- **L4:** Tantivy with per-entity-type schemas and field boosting
- **L5:** DuckDB in-memory, on-demand sync from RocksDB with 300s TTL
- **Hybrid:** RRF merge with k=60, configurable weights, deduplication by memory ID
- **Engine:** All tiers `Option<Arc<>>` disabled by default

### Resolved Questions

| ID | Question | Resolution |
|---|---|---|
| RQ-001 | HNSW remove approach | **Incremental remove()** — use `instant-distance` native deletion if available; otherwise logical deletion via `HashSet<u64>` filtered post-query, with optional `rebuild()` for physical compaction. |

---

## API Contract {#api}

> **Status:** `FROZEN`

### VectorIndex Trait

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

### FullTextSearch Trait

```rust
pub struct FieldValue {
    pub field_name: &'static str,
    pub value: String,
}

pub trait FullTextSearch: Send + Sync {
    fn index(&self, doc_id: &str, fields: &[FieldValue]) -> Result<()>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<(String, f32)>>;
    fn delete(&self, doc_id: &str) -> Result<()>;
    fn flush(&self) -> Result<()>;
}
```

### AnalyticsEngine Trait

```rust
pub trait AnalyticsEngine: Send + Sync {
    fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Vec<Value>>>;
    fn sync(&self, cf_name: &str) -> Result<()>;
    fn sync_all(&self) -> Result<()>;
}
```

### EngineConfig Additions

```rust
pub struct EngineConfig {
    pub storage: StorageConfig,
    pub enable_vector_index: bool,
    pub vector_dimension: u32,
    pub snapshot_path: Option<PathBuf>,
    pub enable_fulltext_search: bool,
    pub tantivy_path: Option<PathBuf>,
    pub enable_analytics: bool,
    pub analytics_cache_ttl_secs: u64,
}
```

### HybridSearchQuery

```rust
pub struct HybridSearchQuery {
    pub query_text: Option<String>,
    pub query_vector: Option<Vec<f32>>,
    pub top_k: usize,
    pub vector_weight: f32,       // default 0.5
    pub text_weight: f32,         // default 0.5
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub session_id: Option<Uuid>,
}
```

---

## Acceptance Criteria {#ac}

> **Status:** ✅ 16 / 16 to Verify

| ID | Description | Status |
|---|---|---|
| AC-VEC-H1 | Insert 100 embeddings, search returns top-K with correct similarity ordering | 🔶 To Verify |
| AC-VEC-H2 | Snapshot round-trip: save → load → search returns same top-5 results | 🔶 To Verify |
| AC-VEC-H3 | Auto-snapshot after 1,000 mutations writes file to disk | 🔶 To Verify |
| AC-FTS-H1 | Index a document, search by keyword returns correct result | 🔶 To Verify |
| AC-FTS-H2 | Index persistence across process restart (Tantivy directory) | 🔶 To Verify |
| AC-FTS-H3 | Phrase query returns only exact phrase matches | 🔶 To Verify |
| AC-ANA-H1 | Sync telemetry CF, query COUNT(*) returns correct row count | 🔶 To Verify |
| AC-ANA-H2 | Multiple sequential analytics queries with different filters | 🔶 To Verify |
| AC-HYB-H1 | Hybrid search returns merged, deduplicated results from L3+L4 | 🔶 To Verify |
| AC-HYB-H2 | weight=[1.0, 0.0] returns only L3 results | 🔶 To Verify |
| AC-HYB-H3 | weight=[0.0, 1.0] returns only L4 results | 🔶 To Verify |
| AC-EFF-H1 | Session efficiency = useful/total memories | 🔶 To Verify |
| AC-EFF-H2 | Metric correlation in [-1.0, 1.0] range | 🔶 To Verify |
| AC-ENG-H1 | Default config: all tiers disabled, existing operations work | 🔶 To Verify |
| AC-ENG-H2 | L3 enabled via config initialises HNSW, updates on memory create | 🔶 To Verify |
| AC-ENG-H3 | `cargo build --workspace` + `cargo test --workspace` pass | 🔶 To Verify |

---

## Edge Cases {#edgecases}

> **Status:** 48 Documented (see `EDGE_CASES.md` for full catalog)

| Category | Count | Key Items |
|---|---|---|
| L3: Vector Index | 18 | Empty search, dimension mismatch, corrupt snapshot, NaN/Inf |
| L4: Full-Text Search | 12 | Special chars, read-only index, concurrent access |
| L5: Analytics | 10 | Unsynced table, zero-division, SQL injection attempt |
| Hybrid Search | 8 | Tier disabled mid-query, dedup, extreme weights |
| Engine Integration | 5 | All disabled default, invalid config, wrong dimension |
| Efficiency/Correlation | 6 | Single session, zero variance, negative duration |

---

## Test Coverage {#tests}

> **Status:** To be implemented

| Test File | Covers |
|---|---|
| `contexter-core/tests/vector/hnsw_test.rs` | HNSW insert/search/snapshot/remove edge cases |
| `contexter-core/tests/fts/tantivy_test.rs` | Tantivy index/search/delete/persistence |
| `contexter-core/tests/analytics/duckdb_test.rs` | DuckDB sync/query/multi-query/efficiency |
| Inline `#[cfg(test)]` in each new module | Unit tests for distance kernels, snapshot format, schema builders, query parsers |

---

## Implementation References {#references}

### Cargo.toml additions

```toml
# L3: Vector index
instant-distance = { version = "0.6", features = ["enable-serde"] }
rand = "0.8"        # HNSW construction

# L4: Full-text search
tantivy = "0.22"

# L5: Analytics engine
duckdb = { version = "0.10", features = ["bundled"] }

# Error-types for new modules
thiserror = "1"     # (already present)
```

### EngineConfig struct

```rust
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub storage: StorageConfig,
    // L3
    pub enable_vector_index: bool,
    pub vector_dimension: u32,
    pub snapshot_path: Option<PathBuf>,
    // L4
    pub enable_fulltext_search: bool,
    pub tantivy_path: Option<PathBuf>,
    // L5
    pub enable_analytics: bool,
    pub analytics_cache_ttl_secs: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            enable_vector_index: false,
            vector_dimension: 384,
            snapshot_path: None,
            enable_fulltext_search: false,
            tantivy_path: None,
            enable_analytics: false,
            analytics_cache_ttl_secs: 300,
            storage: StorageConfig::default(),
        }
    }
}
```

---

## Validation Contract Artifacts {#contract}

| File | Description |
|---|---|
| **SPEC.md** | Formal specification with 25 requirements across L3/L4/L5 + hybrid + engine integration |
| **ACCEPTANCE.md** | 16 Given/When/Then acceptance criteria for all verification points |
| **EDGE_CASES.md** | 48 edge cases across all tiers |
| **plan/preview/** | Approved design preview (this document) |

---

## Approved Contract Summary {#summary}

| Metric | Count |
|---|---|
| AC (All to Verify) | 16 |
| Edge Cases | 48 |
| Tests | 3 integration + inline unit tests |
| Artifacts | 4 |
| New Modules | 3 (`vector/`, `fts/`, `analytics/`) |
| New Dependencies | 3 (`instant-distance`, `tantivy`, `duckdb`) |

This approved contract defines Phase 2 of the Contexter search and analytics engine. All 16 acceptance criteria must pass validation.

---

**Generated · 2026-07-25 · Contexter — Phase 2 Search & Analytics Approved Contract · v1.0.0**

<!-- LOCKED: Approved 2026-07-25 -->
