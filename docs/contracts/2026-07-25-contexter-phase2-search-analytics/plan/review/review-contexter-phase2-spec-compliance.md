# SPEC Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> L3 (HNSW vector index), L4 (Tantivy full-text search), L5 (DuckDB analytics), hybrid search, and efficiency/correlation analytics — all optional storage tiers for the Contexter Rust engine.

**Verdict:** FAIL (class: INCOMPLETE)

2026-07-25 · 23/35 requirements matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

| REQ | Description | Status |
|-----|-------------|--------|
| REQ-VEC-001 | VectorIndex trait with insert/search/remove/save_snapshot/load_snapshot | ✅ MATCHED |
| REQ-VEC-002 | HNSW with configurable M=16, ef_construction=200, ef_search=50 | ⚠️ PARTIAL |
| REQ-VEC-003 | Cosine similarity default; Euclidean/Dot product alternatives | ⚠️ PARTIAL |
| REQ-VEC-004 | Binary snapshot with magic, version, dim/count, adjacency, embeddings | ⚠️ PARTIAL |
| REQ-VEC-005 | Auto-snapshot every 1,000 mutations and on graceful shutdown | ⚠️ PARTIAL |
| REQ-VEC-006 | Load snapshot on startup; rebuild if L2 count != index count | ⚠️ PARTIAL |
| REQ-VEC-007 | Configurable embedding dimensions (default 384) | ✅ MATCHED |
| REQ-VEC-008 | In-memory and persisted modes (snapshot optional) | ✅ MATCHED |
| REQ-VEC-009 | ID-based removal from graph | ✅ MATCHED |
| REQ-FTS-001 | FullTextSearch trait with index/search/delete/flush/load | ⚠️ PARTIAL |
| REQ-FTS-002 | Per-entity-type schema (memory, session, agent, skill) | ⚠️ PARTIAL |
| REQ-FTS-003 | Field-level boosting: content=1.0, title=2.0, tags=1.5 | ✅ MATCHED |
| REQ-FTS-004 | Query parsing with phrase, fuzzy, boolean operators | ✅ MATCHED |
| REQ-FTS-005 | Incremental indexing on write | ✅ MATCHED |
| REQ-FTS-006 | Index directory at ~/.contexter/tantivy_index/ | ⚠️ PARTIAL |
| REQ-FTS-007 | Automatic segment merging (Tantivy default) | ✅ MATCHED |
| REQ-ANA-001 | AnalyticsEngine trait with query/sync/sync_all | ✅ MATCHED |
| REQ-ANA-002 | DuckDB in-memory tables populated from RocksDB iterators | ⚠️ PARTIAL |
| REQ-ANA-003 | Predefined SQL queries (session count, memory count, telemetry agg) | ✅ MATCHED |
| REQ-ANA-004 | On-demand sync when analytics request arrives | ✅ MATCHED |
| REQ-ANA-005 | Configurable cache TTL (default 300s) | ✅ MATCHED |
| REQ-HYB-001 | hybrid_search() merging L3+L4 results | ✅ MATCHED |
| REQ-HYB-002 | Configurable weighting (default 0.5/0.5) | ✅ MATCHED |
| REQ-HYB-003 | RRF merge strategy with k=60 | ✅ MATCHED |
| REQ-HYB-004 | Deduplicated, reranked results with score annotations | ✅ MATCHED |
| REQ-HYB-005 | Apply filter criteria (memory_type, tags, session_id) | ✅ MATCHED |
| REQ-EFF-001 | Session efficiency score = useful/total memories per session | ✅ MATCHED |
| REQ-EFF-002 | Pearson correlation between duration and memory count | ✅ MATCHED |
| REQ-EFF-003 | Store efficiency scores in `efficiency_map` column family | ❌ UNMATCHED |
| REQ-EFF-004 | Cache efficiency results with per-session granularity | ❌ UNMATCHED |
| REQ-ENG-001 | Engine composes tiers as Option<Arc<>> for graceful degradation | ✅ MATCHED |
| REQ-ENG-002 | L3/L4 updated on memory write (vector insert + FTS index) | ✅ MATCHED |
| REQ-ENG-003 | L5 accessible via Engine::run_analytics() | ✅ MATCHED |
| REQ-ENG-004 | All tiers disabled by default; enabled via config | ✅ MATCHED |
| REQ-ENG-005 | Engine passes StorageBackend to L5 for RocksDB→DuckDB sync | ❌ UNMATCHED |

---

## 02 · Implementation Mapping

### L3: HNSW Vector Index

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-VEC-001 | `contexter-core/src/vector/mod.rs` | 21–55 | `trait VectorIndex` with `insert()`, `search()`, `remove()`, `save_snapshot()`, `load_snapshot()`, `len()`, `is_empty()` |
| REQ-VEC-002 | `contexter-core/src/vector/hnsw.rs` | 46–76, 149–150, 183 | HNSW struct using `instant_distance`; `m: 16`, `ef_construction: 200` in snapshot header; `Search::default()` for ef_search — values correct but not user-configurable |
| REQ-VEC-003 | `contexter-core/src/vector/distance.rs` | 11–19, 24–30, 33–35 | `cosine_similarity()`, `euclidean_distance()`, `dot_product()` implemented; HNSW uses cosine only (hnsw.rs:34–37) |
| REQ-VEC-004 | `contexter-core/src/vector/snapshot.rs` | 1–23, 138–223 | Binary format: magic `0x484E5357`, version 1, dimension, element_count, m, ef_construction, removed set, embeddings. No adjacency list stored — graph rebuilt on load via `self.rebuild()` (hnsw.rs:233) |
| REQ-VEC-005 | `contexter-core/src/vector/hnsw.rs` | 73, 83–89, 119–130 | `auto_snapshot_threshold: 1000`, `check_auto_snapshot()` on insert/remove. No graceful shutdown handler. |
| REQ-VEC-006 | `contexter-core/src/engine/mod.rs` | 249–262 | Snapshot loaded if `snapshot_path` exists and `enable_vector_index` is true. No L2 count mismatch detection. |
| REQ-VEC-007 | `contexter-core/src/vector/hnsw.rs` | 65 | `HnswVectorIndex::new(dimension: usize)`. Default 384 at `engine/mod.rs:175` |
| REQ-VEC-008 | `contexter-core/src/vector/hnsw.rs` | 58–60, 83–89 | `snapshot_path: RwLock<Option<PathBuf>>` — optional auto-snapshot. Engine only saves/loads if path configured. |
| REQ-VEC-009 | `contexter-core/src/vector/hnsw.rs` | 201–207 | `remove()` adds ID to `removed: HashSet<String>` — logical deletion, filtered in search (hnsw.rs:189) |

### L4: Tantivy Full-Text Search

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-FTS-001 | `contexter-core/src/fts/mod.rs` | 29–41 | `trait FullTextSearch` with `index()`, `search()`, `delete()`, `flush()`. **Missing:** `load()` method. Initialization is via `TantivyIndex::open()` instead. |
| REQ-FTS-002 | `contexter-core/src/fts/schema.rs` | 31–65, 86–91 | `memory_schema()` (with title, tags), `default_schema()` (content only). `schema_for_entity()` maps `"memory"` → memory schema, everything else → default. Session/agent/skill share default. |
| REQ-FTS-003 | `contexter-core/src/fts/tantivy.rs` | 122–135 | Field boosts: content=1.0, title=2.0, tags=1.5 via `query_parser.set_field_boost()` |
| REQ-FTS-004 | `contexter-core/src/fts/query.rs` | 15–22, 33–62 | `parse_query()`, `parse_boosted_query()`. Delegates to Tantivy's `QueryParser` which handles phrase/fuzzy/boolean. |
| REQ-FTS-005 | `contexter-core/src/engine/memory.rs` | 38–48 | On `create_memory()`: `fts.index()` called with memory content. |
| REQ-FTS-006 | `contexter-core/src/engine/mod.rs` | 163–164 | `tantivy_path: Option<PathBuf>` — configurable, not defaulted to `~/.contexter/tantivy_index/`. Engine errors if `enable_fulltext_search=true` but `tantivy_path` is None (engine/mod.rs:271–273). |
| REQ-FTS-007 | `contexter-core/src/fts/tantivy.rs` | 46–48 | Tantivy `IndexWriter` with default auto-merging. |

### L5: DuckDB Analytics Engine

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-ANA-001 | `contexter-core/src/analytics/mod.rs` | 57–72 | `trait AnalyticsEngine` with `query()`, `sync()`, `sync_all()`, `set_storage_backend()` |
| REQ-ANA-002 | `contexter-core/src/analytics/duckdb.rs` | 34–43, 55–81, 212–309 | In-memory DuckDB (`Connection::open_in_memory`). Tables created from `table_schemas()`. `sync()` inserts hardcoded sample data (not RocksDB iterators). Comment at lines 223–228 acknowledges missing RocksDB integration. |
| REQ-ANA-003 | `contexter-core/src/analytics/queries.rs` | 10–16, 19–24, 27–36, 41–55, 58–82 | `SESSION_COUNT_BY_RANGE`, `MEMORY_COUNT_BY_TYPE`, `TELEMETRY_AGGREGATION`, `EFFICIENCY_SCORES`, `METRIC_CORRELATION` |
| REQ-ANA-004 | `contexter-core/src/analytics/duckdb.rs` | 84–91, 135–155 | `needs_sync()` checks TTL. Auto-sync in `query()`: before executing predefined queries, checks and syncs relevant CFs. |
| REQ-ANA-005 | `contexter-core/src/analytics/duckdb.rs` | 40, 55 | `cache_ttl_secs: u64` field, constructor parameter. Default 300 at `engine/mod.rs:180`. |

### Hybrid Search

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-HYB-001 | `contexter-core/src/engine/search.rs` | 106–233 | `pub fn hybrid_search()` runs L3 + L4 search, merges results via RRF. |
| REQ-HYB-002 | `contexter-core/src/engine/search.rs` | 31–35, 50 | `vector_weight: f32` field, default `0.5`. Weighted combination at lines 180–194. |
| REQ-HYB-003 | `contexter-core/src/engine/search.rs` | 65, 140, 163 | `const RRF_K: f32 = 60.0`. RRF score = `1.0 / (60.0 + rank)`. |
| REQ-HYB-004 | `contexter-core/src/engine/search.rs` | 128, 183–195, 228–230 | `HashMap<String, (Memory, f32, f32)>` deduplicates results. Scores computed as weighted RRF. Sorted descending at line 229. |
| REQ-HYB-005 | `contexter-core/src/engine/search.rs` | 197–226 | In-memory filter: `memory_type`, `tags`, `session_id`, `agent_id` applied via `retain()`. |

### Efficiency & Correlation

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-EFF-001 | `contexter-core/src/analytics/queries.rs` | 41–55 | `EFFICIENCY_SCORES` SQL: `CASE WHEN COUNT(m.id) > 0 THEN CAST(useful AS DOUBLE) / CAST(total AS DOUBLE) ELSE 0.0 END`. Processed at `engine/analytics.rs:169–216`. |
| REQ-EFF-002 | `contexter-core/src/analytics/queries.rs` | 58–82 | `METRIC_CORRELATION` SQL: Pearson formula with covariance/std. Processed at `engine/analytics.rs:218–251`. |
| REQ-EFF-003 | — | — | **NOT IMPLEMENTED.** No `efficiency_map` column family. Scores computed in-memory only. |
| REQ-EFF-004 | — | — | **NOT IMPLEMENTED.** No per-session efficiency cache. DuckDB TTL cache applies at table level, not per-session granularity. |

### Engine Integration

| REQ | File | Lines | Evidence |
|-----|------|-------|----------|
| REQ-ENG-001 | `contexter-core/src/engine/mod.rs` | 203–213 | `vector_index: Option<Arc<dyn VectorIndex>>`, `fts_index: Option<Arc<dyn FullTextSearch>>`, `analytics_engine: Option<Arc<dyn AnalyticsEngine>>`. |
| REQ-ENG-002 | `contexter-core/src/engine/memory.rs` | 28–49 | On `create_memory()`: L3 insert (lines 29–35), L4 index + flush (lines 38–48). On `delete_memory()`: L3 remove (108–110), L4 delete + flush (113–116). |
| REQ-ENG-003 | `contexter-core/src/engine/analytics.rs` | 52–96 | `pub fn run_analytics()` syncs all tables, computes efficiency, correlation, session/memory counts. Returns `AnalyticsReport`. |
| REQ-ENG-004 | `contexter-core/src/engine/mod.rs` | 170–183 | `Default` for `EngineConfig`: all enable flags `false`. `Engine::open()` (lines 222–234) creates engine with all tiers `None`. |
| REQ-ENG-005 | — | — | **NOT IMPLEMENTED.** `DuckDbEngine::set_storage_backend()` exists (duckdb.rs:318–320) but is never called from `Engine::with_config()` (engine/mod.rs:280–286). No `StorageBackend` reference is passed. |

---

## 03 · Unmatched Requirements

### REQ-EFF-003: Store efficiency scores in `efficiency_map` column family

**Severity:** HIGH — Data persistence gap

The SPEC requires computed efficiency scores to be stored in an `efficiency_map` column family in RocksDB. The current implementation computes efficiency scores via SQL queries in DuckDB and returns them in-memory only. There is no `efficiency_map` column family, no RocksDB write path for efficiency data, and no persistence of computed results.

**Evidence:** No code writes to an `efficiency_map` column family anywhere in `contexter-core/src/`.

---

### REQ-EFF-004: Cache efficiency results with per-session granularity

**Severity:** HIGH — Performance/design gap

The SPEC requires efficiency results to be cached with per-session granularity. The current implementation has no caching for efficiency results. The DuckDB engine maintains TTL-based table staleness but this operates at the whole-table level, not at per-session granularity. Every call to `get_efficiency_scores()` recomputes from scratch.

**Evidence:** No per-session cache key or cache store exists for efficiency scores.

---

### REQ-ENG-005: Engine passes correct `StorageBackend` reference to L5 for RocksDB→DuckDB sync

**Severity:** HIGH — Integration gap

The SPEC requires `Engine` to pass a `StorageBackend` reference to the L5 analytics engine so that `sync()` can iterate RocksDB column families. While `DuckDbEngine::set_storage_backend()` is defined (analytics/duckdb.rs:318–320), it is never called from `Engine::with_config()`. As a result, the `sync()` method falls back to inserting hardcoded sample data rather than reading actual RocksDB content.

**Evidence:** `Engine::with_config()` in engine/mod.rs lines 280–286 creates the analytics engine but never calls `set_storage_backend()`. The analytics sync at duckdb.rs:223–228 has a comment: "The real RocksDB sync will be integrated when L5 is wired into the Engine construction path."

---

## 04 · Partially Matched Requirements

### REQ-VEC-002: HNSW with configurable M=16, ef_construction=200, ef_search=50

**Gap:** M and ef_construction are set to the specified values (snapshot.rs:149–150) but as hardcoded constants in the snapshot header, not as configurable parameters exposed through `HnswVectorIndex::new()` or `EngineConfig`. ef_search defaults to `Search::default()` from `instant_distance` (hnsw.rs:183) — not explicitly set to 50.

**Fix boundary:** Single Worker — `HnswVectorIndex` constructor or `EngineConfig` needs M, ef_construction, ef_search fields.

---

### REQ-VEC-003: Cosine similarity as default; Euclidean and Dot product as alternatives

**Gap:** Three distance functions exist in `distance.rs`: `cosine_similarity()`, `euclidean_distance()`, `dot_product()`. However, `HnswVectorIndex` hardcodes cosine similarity usage (`Embedding::distance()` at hnsw.rs:34–37). Euclidean and dot product are never wired in as alternatives.

**Fix boundary:** Single Worker — add a `DistanceMetric` enum to `HnswVectorIndex` and route `distance()` accordingly.

---

### REQ-VEC-004: Binary snapshot with magic number, version, dimension/element counts, adjacency list, and embedding vectors

**Gap:** The binary snapshot format stores embeddings and metadata (magic, version, dimension, element count, m, ef_construction, removed set) but does NOT store the HNSW adjacency list. The graph is rebuilt from embeddings on load via `self.rebuild()` (hnsw.rs:233). The SPEC explicitly lists "adjacency list" as part of the snapshot format.

**Fix boundary:** Single Worker — either store the adjacency list in the snapshot or update the SPEC to document the rebuild-on-load approach.

---

### REQ-VEC-005: Auto-snapshot every 1,000 mutations and on graceful shutdown

**Gap:** Auto-snapshot at 1,000 mutations is implemented (`check_auto_snapshot()` at hnsw.rs:119–130, threshold at hnsw.rs:73). However, "on graceful shutdown" is NOT implemented — there is no `Drop` impl or shutdown hook that triggers `save_snapshot()`.

**Fix boundary:** Single Worker — implement `Drop` for `HnswVectorIndex` to trigger auto-snapshot, or wire a shutdown handler in `Engine`.

---

### REQ-VEC-006: On startup, load snapshot; rebuild if memory count in L2 doesn't match index entry count

**Gap:** The startup load is implemented (engine/mod.rs:249–262). However, "rebuild if memory count in L2 doesn't match index entry count" is NOT implemented — the engine loads whatever snapshot exists without cross-checking against L2.

**Fix boundary:** Single Worker — add L2 memory count query and compare with index entry count on startup.

---

### REQ-FTS-001: FullTextSearch trait with index/search/delete/flush/load methods

**Gap:** The trait (fts/mod.rs:29–41) defines `index()`, `search()`, `delete()`, `flush()`. The SPEC lists `load()` as a required method. The trait does NOT have a `load()` method — initialization is done via `TantivyIndex::open()` as a constructor.

**Fix boundary:** Single Worker — either add `load()` to `FullTextSearch` or update the SPEC.

---

### REQ-FTS-002: Per-entity-type schema (memory, session, agent, skill content)

**Gap:** The schema system (fts/schema.rs) defines only two schemas: `memory_schema()` (with title, tags) and `default_schema()` (content only). All non-memory entity types (session, agent, skill) map to the generic default schema via `schema_for_entity()` (schema.rs:86–91). The SPEC requires four distinct schemas.

**Fix boundary:** Single Worker — add dedicated schemas for session, agent, and skill entities.

---

### REQ-FTS-006: Index directory at `~/.contexter/tantivy_index/`

**Gap:** The Tantivy index path is configurable via `EngineConfig::tantivy_path` (engine/mod.rs:163–164), which is `Option<PathBuf>`. There is no default path set to `~/.contexter/tantivy_index/`. If the path is not provided and FTS is enabled, the engine returns an error (engine/mod.rs:271–273).

**Fix boundary:** Single Worker — set a default path of `~/.contexter/tantivy_index/` when `enable_fulltext_search=true` and `tantivy_path` is None.

---

### REQ-ANA-002: DuckDB backend with in-memory tables populated on demand from RocksDB iterators

**Gap:** The DuckDB backend is correctly implemented with in-memory tables. However, the `sync()` method (duckdb.rs:212–309) inserts hardcoded sample data rather than reading from RocksDB iterators. The `storage_backend` field exists but is never populated from `Engine` (see REQ-ENG-005). A code comment at duckdb.rs:223–228 explicitly acknowledges this gap.

**Fix boundary:** Single Worker — integrate the RocksDB iterator logic into `sync()` once the `StorageBackend` reference is wired (which depends on REQ-ENG-005).

---

## 05 · Constraint Violations

| CON | Description | Status | Notes |
|-----|-------------|--------|-------|
| CON-001 | No external processes — all tiers run in-process | ✅ OK | HNSW (instant_distance), Tantivy, DuckDB all in-process |
| CON-002 | L3 snapshot backward-compatible with version field | ✅ OK | Version field in snapshot header; validation at snapshot.rs:54–68 |
| CON-003 | L5 is ephemeral — data never persisted in DuckDB | ✅ OK | `Connection::open_in_memory()` used exclusively |
| CON-004 | Hybrid search must not degrade `search_memories()` | ✅ OK | `search_memories()` is a separate method that never touches L3/L4 (search.rs:73–80) |
| CON-005 | Tantivy index directory created if absent | ✅ OK | `create_dir_all(parent)` at tantivy.rs:38–41 |

---

## 06 · Edge Case Verification

| Edge Case | Status | Notes |
|-----------|--------|-------|
| EC-VEC-01: Empty index search | ✅ Covered | hnsw.rs:172 — returns empty if `is_empty()`, hnsw.rs:310–313 test |
| EC-VEC-02: Single-element index | ✅ Covered | hnsw.rs:397–400 — k larger than index bound test |
| EC-VEC-03: k larger than index | ✅ Covered | hnsw.rs:185 — `actual_k = k.min(self.len())` |
| EC-VEC-04: k=0 search | ✅ Covered | hnsw.rs:172 — returns empty if k==0, test at hnsw.rs:403–408 |
| EC-VEC-05: Dimension mismatch on insert | ✅ Covered | hnsw.rs:136–140, test at hnsw.rs:351–363 |
| EC-VEC-06: Dimension mismatch on search | ✅ Covered | hnsw.rs:168–171, test at hnsw.rs:365–374 |
| EC-VEC-07: Remove existing ID | ✅ Covered | hnsw.rs:201–207, test at hnsw.rs:316–332 |
| EC-VEC-08: Remove nonexistent ID | ✅ Covered | hnsw.rs:204 — insert to set (no-op if dup), test at hnsw.rs:410–416 |
| EC-VEC-09: Remove from empty index | ✅ Covered | hnsw.rs:201–207 — no error check, test covers (hnsw.rs:410–416) |
| EC-VEC-10: Save snapshot to readonly path | ✅ Covered | `File::create` returns io::Error → mapped via `From` in error.rs:19–23 |
| EC-VEC-11: Load from nonexistent path | ✅ Covered | `File::open` returns io::Error → mapped via `From` |
| EC-VEC-12: Load corrupt snapshot | ✅ Covered | snapshot.rs:186 `header.validate()`, test at hnsw.rs:503–514 |
| EC-VEC-13: Wrong magic number | ✅ Covered | snapshot.rs:55–59, test at snapshot.rs:256–268 |
| EC-VEC-14: Version mismatch | ✅ Covered | snapshot.rs:61–66, test at snapshot.rs:270–283 |
| EC-VEC-15: Auto-snapshot at 1,000 | ✅ Covered | hnsw.rs:119–130, threshold 1000 at line 73 |
| EC-VEC-16: Multiple insert same ID (update) | ✅ Covered | hnsw.rs:151–156 — replaces if exists, test at hnsw.rs:335–345 |
| EC-VEC-17: All-zero query vector | ✅ Covered | No special case needed — cosine handles zero norm (distance.rs:15–17) |
| EC-VEC-18: NaN/Inf rejected | ✅ Covered | hnsw.rs:98–103, tests at hnsw.rs:376–390 |
| EC-FTS-01: Empty index search | ✅ Covered | tantivy.rs:225–241 test — empty index returns empty |
| EC-FTS-02: No match search | ✅ Covered | tantivy.rs:225–241 test — nonexistent query returns empty |
| EC-FTS-03: Special characters in query | ⚠️ Partial | Tantivy QueryParser handles special characters; no explicit test for all special chars |
| EC-FTS-04: Delete nonexistent doc | ⚠️ Partial | tantivy.rs:170–174 implements delete; no explicit test for nonexistent ID |
| EC-FTS-05: Delete already-deleted doc | ⚠️ Partial | No explicit test |
| EC-FTS-06: Index with empty content | ✅ Covered | tantivy.rs:77–104 handles empty content |
| EC-FTS-09: Index directory nonexistent | ✅ Covered | tantivy.rs:38–41 — creates directory |
| EC-FTS-12: Flush on idle index | ✅ Covered | tantivy.rs:177–183 — commit on empty writer succeeds |
| EC-ANA-01: Query on unsynced table | ✅ Covered | duckdb.rs:397–406 test — empty result (table exists but no data) |
| EC-ANA-02: Sync empty CF | ⚠️ Partial | Not explicitly tested |
| EC-ANA-03: Sync nonexistent CF | ✅ Covered | duckdb.rs:216–217 — returns ColumnFamilyNotFound error |
| EC-ANA-04: Invalid SQL query | ✅ Covered | duckdb.rs:159–165 — prepare returns QueryError |
| EC-HYB-01: L3 disabled, L4 enabled | ✅ Covered | search.rs:123 logic, test at search.rs:751–800 |
| EC-HYB-02: L3 enabled, L4 disabled | ✅ Covered | search.rs:123 logic, test at search.rs:693–749 |
| EC-HYB-03: Both tiers no matches | ✅ Covered | search.rs:128 — empty merged map returns empty |
| EC-HYB-04: RRF k=0 | ⚠️ Partial | RRF_K is const 60, can't be 0 at runtime |
| EC-HYB-06: Same ID in both result sets | ✅ Covered | search.rs:128 — HashMap deduplicates by ID |
| EC-ENG-01: All tiers disabled (default) | ✅ Covered | engine/mod.rs:170–183 default, test at search.rs:618–633 |
| EC-ENG-05: run_analytics() with L5 disabled | ✅ Covered | engine/analytics.rs:53–56 — returns `Unimplemented` if None |
| EC-EFF-02: Zero division guard | ✅ Covered | queries.rs:46–49 `CASE WHEN COUNT > 0 THEN ... ELSE 0.0` |
| EC-EFF-04: Single session correlation | ✅ Covered | queries.rs:76–78 `CASE WHEN std_dur > 0 AND std_mem > 0 THEN ... ELSE 0.0` |

---

## 07 · Carryover Check

| Check | Result |
|-------|--------|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | **NO** — Findings are documented in this report but no bug contracts have been created. |
| Zero findings are being silently deferred to a future iteration | **NO** — See unmatched requirements REQ-EFF-003, REQ-EFF-004, REQ-ENG-005 which must be resolved. |

---

## 08 · Summary

> **SPEC Compliance Assessment**
> The Phase 2 implementation covers the majority of requirements (23/35 fully matched, 9 partially matched, 3 unmatched). The core architecture (three-tier composition in Engine, trait definitions, HNSW index, Tantivy backend, DuckDB engine, hybrid search with RRF, efficiency/correlation SQL) is implemented and working. However, three critical gaps exist: the `efficiency_map` column family (REQ-EFF-003), per-session efficiency caching (REQ-EFF-004), and the missing StorageBackend wiring to L5 (REQ-ENG-005). An additional 9 requirements are partially matched, with issues ranging from missing configurability (VEC-002) to sample data in sync (ANA-002).

> **Findings**
> 1. ❌ **REQ-EFF-003**: Efficiency scores not stored in `efficiency_map` column family
> 2. ❌ **REQ-EFF-004**: No per-session granularity cache for efficiency results
> 3. ❌ **REQ-ENG-005**: StorageBackend not wired to L5 DuckDbEngine
> 4. ⚠️ **REQ-VEC-002**: HNSW parameters not user-configurable (hardcoded constants)
> 5. ⚠️ **REQ-VEC-003**: Euclidean/Dot product not wired as alternatives in HNSW
> 6. ⚠️ **REQ-VEC-004**: Snapshot missing adjacency list (graph rebuilt on load)
> 7. ⚠️ **REQ-VEC-005**: No graceful shutdown handler for auto-snapshot
> 8. ⚠️ **REQ-VEC-006**: No L2 vs index count mismatch detection on startup
> 9. ⚠️ **REQ-FTS-001**: Trait missing `load()` method
> 10. ⚠️ **REQ-FTS-002**: Only 2 entity schemas instead of the SPEC's 4
> 11. ⚠️ **REQ-FTS-006**: No default path for Tantivy index directory
> 12. ⚠️ **REQ-ANA-002**: sync() uses sample data, not RocksDB iterators

---

## 09 · Final Verdict

| Criterion | Result |
|-----------|--------|
| All REQ-XXX matched with implementation code | ❌ FAIL — 3 unmatched, 9 partial |
| All CON-XXX constraints respected | ✅ PASS — 5/5 constraints satisfied |
| All EDGE_CASES covered by implementation or tests | ⚠️ Most — 5 edge cases partially covered or untested |
| Carryover declaration clean | ❌ NO — Unmatched requirements exist without bug contracts |
| **Overall** | **❌ FAIL** |

---

_Generated by SPEC Compliance Validator · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase2-search-analytics_
