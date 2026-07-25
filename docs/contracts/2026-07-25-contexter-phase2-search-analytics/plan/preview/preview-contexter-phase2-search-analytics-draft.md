# Phase 2: Search & Analytics Engine — Design Draft

> **Status:** `DRAFT — Pending Review` · **Version:** `v0.1.0-draft`
> **Feature:** 3 Storage Tiers (L3 HNSW, L4 Tantivy, L5 DuckDB) · 1 Open Question

---

## Navigation

- [Problem](#problem)
- [Options](#options)
- [System Design](#architecture)
- [Data Flow](#dataflow)
- [Questions](#questions)
- [Decisions](#decisions)
- [API](#api)
- [Scope](#scope)
- [AC](#ac)
- [Edge Cases](#edgecases)
- [Summary](#summary)

---

## Why This Feature Exists {#problem}

| The Pain | The Principle |
|---|---|
| Memory retrieval is currently limited to keyword, tag, and session-ID filtering — no semantic similarity search, no BM25 full-text ranking, no analytical queries over telemetry data. Users wanting "find me memories similar to this idea" or "which sessions were most efficient?" cannot answer those questions. L3/L4/L5 stubs exist but return nothing. | A memory platform without semantic search is a key-value store with labels. Phase 2 adds the query algebra that turns collected data into answers: vector similarity ≈ semantic relevance, BM25 ≈ keyword precision, and analytical SQL ≈ structured insight. |

---

## Design Options {#options}

Three independent tiers evaluated. Each can be enabled/disabled independently.

### L3: Vector Index Backend

| Option | Advantages | Disadvantages |
|---|---|---|
| **A — `instant-distance`** (Pure Rust HNSW) | ✅ Zero C deps, fast compile<br>✅ Mature HNSW impl<br>✅ Pure Rust safety | ❌ Smaller community<br>❌ Manual snapshot management |
| **B — `voyager`** (Pure Rust ANN) | ✅ Built-in persistence<br>✅ Configurable distance functions | ❌ Newer crate<br>❌ Fewer downstream users |
| **C — Custom HNSW from scratch** | ✅ Full control over snapshot format<br>✅ Educational value | ❌ Months of debugging<br>❌ Not a real option for a product |

**Recommended: Option A — `instant-distance`** with a thin wrapper for snapshot persistence. It's pure Rust, well-tested, and the snapshot format is straightforward binary.

### L4: Full-Text Search Backend

| Option | Advantages | Disadvantages |
|---|---|---|
| **A — Tantivy** | ✅ Lucene-class BM25<br>✅ Incremental indexing<br>✅ Phrase, fuzzy, boolean queries | ❌ Heavier compile (LLVM deps via `tokenizer-api`) |
| **B — Rust FFI to sqlite FTS5** | ✅ Small compile footprint | ❌ No longer maintained upstream<br>❌ Harder to configure |
| **C — Custom inverted index** | ✅ No deps | ❌ Years of work to approach Tantivy quality |

**Recommended: Option A — Tantivy.** The de facto Rust FTS crate. Schema-per-entity-type and field boosting are first-class features.

### L5: Analytics Engine Backend

| Option | Advantages | Disadvantages |
|---|---|---|
| **A — DuckDB (in-process)** | ✅ True columnar SQL<br>✅ In-process, no server<br>✅ Rich SQL dialect | ❌ Adds ~15MB to binary<br>❌ Transitive C dep via `libduckdb-sys` |
| **B — In-memory aggregator** | ✅ Zero deps<br>✅ Fast for simple counts | ❌ No SQL<br>❌ Must hand-roll every query |
| **C — SQLite in-memory** | ✅ Well-known SQL<br>✅ Already in many systems | ❌ Row-oriented → slow for aggregations<br>❌ No columnar benefits |

**Recommended: Option A — DuckDB.** Columnar engine is materially faster for the aggregation/correlation workload. The `libduckdb-sys` transitive dep is a one-time cost.

---

## System Design {#architecture}

> **Status:** `Draft`

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
├── hnsw.rs        # HNSW wrapper around instant-distance graph
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
1. `sync("telemetry")` → iterate `telemetry` CF → `INSERT INTO telemetry VALUES ...`
2. DuckDB table created lazily on first sync per CF name
3. `sync_all()` iterates all relevant CFs

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

## Data Flow {#dataflow}

### 1. Memory Write → L3 + L4 Update

```
Engine::create_memory()
  │
  ├──→ L2: RocksDB persist
  ├──→ L1: cache invalidate
  ├──→ L3: if configured → vector_index.insert(id, &embedding)
  └──→ L4: if configured → fts_index.index(id, &fields)
```

### 2. Hybrid Search (L3 + L4)

```
Engine::hybrid_search(query_text, query_vector, k, weights)
  │
  ├──→ L3: if configured → knn_search(query_vector, k × 2)
  │     Returns Vec<(id, vector_score)>
  │
  ├──→ L4: if configured → fts_search(query_text, k × 2)
  │     Returns Vec<(id, text_score)>
  │
  └──→ Merge: RRF(deduplicate)
        │
        ├──→ RRF score = Σ 1 / (k + rank_in_tier)
        ├──→ weight = [vector_weight, text_weight]
        └──→ Final = RRF_vector × w_vec + RRF_text × w_text
```

### 3. Analytics Query

```
Engine::run_analytics(query_type, params)
  │
  ├──→ L5: if not configured → return Err("analytics not configured")
  │
  ├──→ sync relevant CFs (if cache stale or TTL expired)
  │     └──→ RocksDB iterator → DuckDB INSERT
  │
  └──→ Execute predefined SQL
        ├──→ session_count_by_range(time_start, time_end)
        ├──→ memory_count_by_type()
        ├──→ telemetry_aggregation(event_type, scope)
        ├──→ efficiency_scores() 
        └──→ metric_correlation()
```

---

## Open Questions {#questions}

| ID | Question | Status |
|---|---|---|
| OQ-001 | Should the `instant-distance` HNSW wrapper support incremental `remove()` or only snapshot-rebuild? HNSW removal is nontrivial (orphan edges). The current design flags removed IDs and ignores them in search results rather than physically removing graph nodes. Is this acceptable? | 🔶 Debating |

---

## Decision Log {#decisions}

| Date | ID | Description | Rationale |
|---|---|---|---|
| 2026-07-25 | CON-L3-001 | `instant-distance` for HNSW (not voyager) | Pure Rust, zero C deps, well-tested HNSW. Thin wrapper for custom snapshot format. |
| 2026-07-25 | CON-L4-001 | Tantivy for FTS (not custom) | De facto Rust FTS crate. BM25, query parser, incremental indexing all built-in. |
| 2026-07-25 | CON-L5-001 | DuckDB for analytics (not SQLite) | Columnar engine is materially faster for aggregation/correlation workload. |
| 2026-07-25 | CON-HYB-001 | RRF merge with k=60 and configurable weights | RRF is stable across score distribution differences between HNSW and BM25. |
| 2026-07-25 | CON-ENG-001 | All tiers Option<Arc<>>, disabled by default | Backward compatible — existing Phase 1 code continues working unchanged. |
| 2026-07-25 | CON-VEC-001 | Snapshot format with magic + version | Enables backward-incompatible changes with clear migration path. |
| 2026-07-25 | CON-ANA-001 | L5 is ephemeral — DuckDB in-memory only | Synchronises from RocksDB on each analytics request. No persistence complexity. |

---

## API Contract {#api}

> ⚠️ **Rust internal trait contracts** — No public API changes at the Python/HTTP level in this phase.

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

### Hybrid Search on Engine

```rust
impl Engine {
    pub fn hybrid_search(
        &self,
        query: &HybridSearchQuery,
    ) -> Result<Vec<(Memory, f32)>>;

    pub fn run_analytics(
        &self,
        query_type: &str,
        params: &[Value],
    ) -> Result<Vec<Vec<Value>>>;
}
```

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

## Out of Scope {#scope}

| # | Item | Rationale |
|---|---|---|
| 01 | L3 GPU acceleration | HNSW is CPU-only. GPU vector search is a future optimisation. |
| 02 | L4 distributed search index | Tantivy is single-node. Distributed search is a separate feature. |
| 03 | L5 persistent DuckDB database | L5 is ephemeral by design. All data lives in RocksDB. Persisting DuckDB would create sync complexity. |
| 04 | Python/HTTP API layer for search/analytics | Phase 3 builds the API layer. Phase 2 is pure Rust. |
| 05 | Real embedding model integration | Phase 2 assumes embeddings are provided by the caller. Model integration is a separate concern. |
| 06 | Multi-tenant separation of search results | Tenant isolation is handled at the Python layer. Rust core returns results unfiltered by tenant. |

---

## Acceptance Criteria {#ac}

> **Status:** Pending review

| ID | Description | Status |
|---|---|---|
| AC-VEC-H1 | Insert 100 embeddings, search returns top-K with correct similarity ordering | 🔶 Pending |
| AC-VEC-H2 | Snapshot round-trip: save → load → search returns same top-5 results | 🔶 Pending |
| AC-VEC-H3 | Auto-snapshot after 1,000 mutations writes file to disk | 🔶 Pending |
| AC-FTS-H1 | Index a document, search by keyword returns correct result | 🔶 Pending |
| AC-FTS-H2 | Index persistence across process restart (Tantivy directory) | 🔶 Pending |
| AC-FTS-H3 | Phrase query returns only exact phrase matches | 🔶 Pending |
| AC-ANA-H1 | Sync telemetry CF, query COUNT(*) returns correct row count | 🔶 Pending |
| AC-ANA-H2 | Multiple sequential analytics queries with different filters | 🔶 Pending |
| AC-HYB-H1 | Hybrid search returns merged, deduplicated results from L3+L4 | 🔶 Pending |
| AC-HYB-H2 | weight=[1.0, 0.0] returns only L3 results | 🔶 Pending |
| AC-HYB-H3 | weight=[0.0, 1.0] returns only L4 results | 🔶 Pending |
| AC-EFF-H1 | Session efficiency = useful/total memories | 🔶 Pending |
| AC-EFF-H2 | Metric correlation in [-1.0, 1.0] range | 🔶 Pending |
| AC-ENG-H1 | Default config: all tiers disabled, existing operations work | 🔶 Pending |
| AC-ENG-H2 | L3 enabled via config initialises HNSW, updates on memory create | 🔶 Pending |
| AC-ENG-H3 | `cargo build --workspace` + `cargo test --workspace` pass | 🔶 Pending |

---

## Edge Cases {#edgecases}

> **Status:** Indentified

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-VEC-01 | Empty index search | Return empty Vec | High |
| EC-VEC-05 | Dimension mismatch on insert | Return Err(DimensionMismatch) | High |
| EC-VEC-08 | Remove nonexistent ID | Succeed silently, len unchanged | Medium |
| EC-VEC-12 | Load corrupt snapshot | Return Err("corrupt snapshot") | High |
| EC-VEC-18 | NaN/Inf in embedding | Return Err("vector contains NaN/Inf") | High |
| EC-FTS-01 | Empty index search (FTS) | Return empty Vec | Medium |
| EC-FTS-09 | Index directory nonexistent | Create directory and continue | Medium |
| EC-ANA-01 | Query on unsynced table | Return Err("table does not exist") | High |
| EC-ANA-10 | Efficiency: zero total memories | Score = 0.0 (zero division guard) | High |
| EC-HYB-01 | L3 disabled during hybrid search | Return only L4 results | Medium |
| EC-HYB-06 | Same ID in both result sets | Deduplicated once | High |
| EC-ENG-01 | All tiers disabled (default) | Engine opens, works without L3/L4/L5 | High |

---

## Design Draft Summary {#summary}

| Metric | Count |
|---|---|
| Acceptance Criteria | 16 |
| Edge Cases | 12 (representative) |
| Design Options | 3 backends evaluated per tier |
| Open Questions | 1 |
| Decision Log Entries | 7 |
| New Dependencies | 3 (`instant-distance`, `tantivy`, `duckdb`/`duckdb-rs`) |
| New Modules | 3 (`vector/`, `fts/`, `analytics/`) |

This draft covers Phase 2 of the Contexter implementation plan. All 16 acceptance criteria must pass before this phase is considered complete.

---

**Generated · 2026-07-25 · Contexter — Phase 2 Search & Analytics Design Draft · v0.1.0-draft**
