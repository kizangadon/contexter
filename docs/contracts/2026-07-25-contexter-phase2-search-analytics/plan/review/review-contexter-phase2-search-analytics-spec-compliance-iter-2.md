# SPEC Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Hybrid search (L3 HNSW + L4 Tantivy) + L5 DuckDB analytics engine wiring, 21 bug fixes

**Verdict:** FAIL (class: HARD)

2026-07-25 · 90/92 requirements matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

### Parent SPEC — L3/L4/L5 Engine

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-VEC-001** | ✅ MATCHED | `contexter-core/src/vector/mod.rs:21-55` | `VectorIndex` trait with `insert()`, `search()`, `remove()`, `save_snapshot()`, `load_snapshot()` methods |
| **REQ-VEC-002** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:83-99` | `HnswVectorIndex::new(dimension, m, ef_construction, ef_search)` accepts params; defaults 16, 200, 50 |
| **REQ-VEC-003** | ✅ MATCHED | `contexter-core/src/vector/distance.rs:11-19,24-32,33-41` | `cosine_similarity()` default; `euclidean_distance()` and `dot_product()` also implemented |
| **REQ-VEC-004** | ✅ MATCHED | `contexter-core/src/vector/snapshot.rs:32-47,155-265` | Binary snapshot format: magic `0x484E5357` ("HNSW"), version 1, dimension, element_count, embeddings, removed set |
| **REQ-VEC-005** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:146-157`, `contexter-core/src/engine/mod.rs:417-439` | `check_auto_snapshot()` triggers every 1000 mutations; `Engine::shutdown()` saves on shutdown |
| **REQ-VEC-006** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:307-325` | **FIXED in Iteration 2.** Startup consistency check: L2 memory count vs HNSW entry count compared; warning logged if mismatch |
| **REQ-VEC-007** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:159,195` | `vector_dimension: u32`, default 384 |
| **REQ-VEC-008** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:286-303` | Without `snapshot_path`, index runs in-memory only |
| **REQ-VEC-009** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:434` | `remove(&self, id)` — logical deletion via removed set |
| **REQ-FTS-001** | ✅ MATCHED | `contexter-core/src/fts/mod.rs:39-51`, `contexter-core/src/fts/tantivy.rs:156-262` | `FullTextSearch` trait with `index/search/delete/flush`; `TantivyIndex` implements all |
| **REQ-FTS-002** | ✅ MATCHED | `contexter-core/src/fts/schema.rs:216-224` | `schema_for_entity("memory"/"session"/"agent"/"skill")` returns per-entity schemas |
| **REQ-FTS-003** | ✅ MATCHED | `contexter-core/src/fts/schema.rs:46` | Field boosts: content=1.0, tags=1.5 (per Bug-API-Conformance, title:2.0 removed) |
| **REQ-FTS-004** | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:225-252` | Tantivy `QueryParser` handles phrase, fuzzy, boolean operators |
| **REQ-FTS-005** | ✅ MATCHED | `contexter-core/src/engine/memory.rs:50-70` | `create_memory()` indexes into FTS on write |
| **REQ-FTS-006** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:180` | `tantivy_path: Option<PathBuf>` in `EngineConfig` |
| **REQ-FTS-007** | ✅ MATCHED | Tantivy native | Tantivy handles auto segment merging internally |
| **REQ-ANA-001** | ✅ MATCHED | `contexter-core/src/analytics/mod.rs:55-66` | `AnalyticsEngine` trait with `query/sync/sync_all` |
| **REQ-ANA-002** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:108-195` | DuckDB in-memory engine; `sync_from_backend()` iterates RocksDB CFs |
| **REQ-ANA-003** | ✅ MATCHED | `contexter-core/src/analytics/queries.rs:10-96` | `SESSION_COUNT_BY_RANGE`, `MEMORY_COUNT_BY_TYPE`, `TELEMETRY_AGGREGATION`, `EFFICIENCY_SCORES`, `METRIC_CORRELATION` |
| **REQ-ANA-004** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:207-209` | `query()` auto-syncs tables when TTL expired |
| **REQ-ANA-005** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:183,203` | `analytics_cache_ttl_secs: u64`, default 300 |
| **REQ-HYB-001** | ✅ MATCHED | `contexter-core/src/engine/search.rs:106-252` | `hybrid_search()` merges L3 + L4 results |
| **REQ-HYB-002** | ✅ MATCHED | `contexter-core/src/engine/search.rs:50-51` | `vector_weight: f32` default 0.5, `text_weight: f32` default 0.5 |
| **REQ-HYB-003** | ✅ MATCHED | `contexter-core/src/engine/search.rs:65` | `RRF_K: f32 = 60.0` |
| **REQ-HYB-004** | ✅ MATCHED | `contexter-core/src/engine/search.rs:215-254` | HashMap dedup + RRF composite score + sort_by + truncate |
| **REQ-HYB-005** | ✅ MATCHED | `contexter-core/src/engine/search.rs:256-267` | In-memory filtering: memory_type, tags, session_id |
| **REQ-EFF-001** | ✅ MATCHED | `contexter-core/src/analytics/queries.rs:41-55` | EFFICIENCY_SCORES SQL: useful_memories / total_memories per session |
| **REQ-EFF-002** | ✅ MATCHED | `contexter-core/src/analytics/queries.rs:58-82` | METRIC_CORRELATION SQL: Pearson r between duration_ms and memory_count |
| **REQ-EFF-003** | ✅ MATCHED | `contexter-core/src/storage/column_families.rs` | `efficiency_map` column family defined |
| **REQ-EFF-004** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:797-826` | `efficiency_cache: Arc<RwLock<HashMap<String, EfficiencyEntry>>>` with per-session granularity + TTL |
| **REQ-ENG-001** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:227-236` | `Engine` struct with `vector_index: Option<Arc<dyn VectorIndex>>`, `fts_index`, `analytics_engine` |
| **REQ-ENG-002** | ✅ MATCHED | `contexter-core/src/engine/memory.rs:38-50` | L3 `vx.insert()` + L4 `fts.index()` in `create_memory()` |
| **REQ-ENG-003** | ✅ MATCHED | `contexter-core/src/engine/analytics.rs` | `Engine::run_analytics()` returns `AnalyticsReport` |
| **REQ-ENG-004** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:194-203` | All defaults: `enable_vector_index: false`, `enable_fulltext_search: false`, `enable_analytics: false` |
| **REQ-ENG-005** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:350` | `engine.set_storage_backend(Box::new(storage.clone()))` |

### Bug-Validation (2 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:273-276` | `if config.vector_dimension == 0 { return Err(EngineError::InvalidConfig(...)) }` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/error/mod.rs:46` | `InvalidConfig(String)` variant with Display + sanitized |

### Bug-Search-Validation (4 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/search.rs:124` | `let vector_weight = query.vector_weight.clamp(0.0, 1.0);` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/engine/search.rs:126-129` | `if query.top_k == 0 { return Ok(Vec::new()) }` else `query.top_k.min(1000)` |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/engine/search.rs:137-141` | Empty/whitespace-only sort_field treated as no sort |
| **REQ-FIX-004** | ✅ MATCHED | `contexter-core/src/engine/search.rs:1077-1223` | 7 tests: weight clamped low/high, limit zero/capped, sort_field empty/whitespace/none |

### Bug-HNSW-Config (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:169-177,197-199` | `hnsw_m(16)`, `hnsw_ef_construction(200)`, `hnsw_ef_search(50)` in `EngineConfig` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:289-292` | `HnswVectorIndex::new(dim, config.hnsw_m, config.hnsw_ef_construction, config.hnsw_ef_search)` |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:83-99,133-134` | `Builder::default().ef_construction(ef_construction).ef_search(ef_search)` |

### Bug-DB-Analytics (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:256-262` | `value_to_duckdb()` converts `Value` → duckdb types; `stmt.query(&param_refs[..])` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:350` | `engine.set_storage_backend(Box::new(storage.clone()))` after constructing DuckDbEngine |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:269-430` | `sync_from_backend()` iterates RocksDB CF: sessions, memories, telemetry |

### Bug-FTS (5 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:156-262` | `FullTextSearch` fully implemented: `index/search/delete/flush` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/fts/schema.rs:42-43` | `content_field` and `tags_field` in memory schema; session/agent/skill also have entity-specific fields |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/models/memory.rs:53-62` | `impl TextContent for Memory` — concatenates content + tags |
| **REQ-FIX-004** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:331-332` | `TantivyIndex::open(path, "memory")` receives `tantivy_path` from config |
| **REQ-FIX-005** | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:119-142` | `add_alias/list_aliases/switch_index` implemented, 5 tests in file |

### Bug-Poison (2 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs` 16 occurrences | `.unwrap_or_else(\|e\| e.into_inner())` on all `Mutex` accesses in DuckDbEngine |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/engine/search.rs:2`, `memory.rs:5` | `unwrap_or_else(\|e\| e.into_inner())` on all `RwLock`/`Mutex` in Engine |

### Bug-Errors (4 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | All `contexter-core/src/engine/*.rs` | No bare `unwrap()` in non-test engine code; all use `?` or `unwrap_or_else` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/error/mod.rs:54` | `UnsupportedOperation(String)` variant with Display + sanitized |
| **REQ-FIX-003** | ✅ MATCHED | See Bug-Poison entries above | Mutex/RwLock poison recovery throughout engine |
| **REQ-FIX-004** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:48-83` | `TempDirGuard` struct with `Drop` impl cleans up temp directory |

### Bug-File-Security (2 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:65-66` | **FIXED in Iteration 2.** `std::fs::set_permissions(&dir, Permissions::from_mode(0o700))` after `create_dir_all()` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:455-458` | Open file first, then `metadata()` on opened handle — eliminates TOCTOU window |

### Bug-Efficiency (4 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:30` | `pub const EFFICIENCY_CF: &str = "efficiency_map";` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:729-733` | `sync()` checks `cf_name == EFFICIENCY_CF` → calls `sync_efficiency_cache_from_backend()` |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:123,473-477,797-826` | `efficiency_cache` field; `query()` checks cache before DuckDB; `populate_efficiency_cache()` |
| **REQ-FIX-004** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:809` | `now.duration_since(entry.cached_at).as_secs() > self.cache_ttl_secs` TTL eviction |

### Bug-Snapshot (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:225-256` | `pub fn save(&self, path: &Path)` with atomic write (tmp + rename) |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:311-335`, `contexter-core/src/engine/mod.rs:334-366` | `periodic_snapshot()` thread; wired in `Engine::with_config()` with `snapshot_interval_secs` |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:419-439` | `Engine::shutdown()` calls `save_snapshot()`, signals cancel, joins thread |

### Bug-Test-Flakiness (1 req)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:56-59` | UUID-v4-based temp directory naming instead of PID-based |

### Bug-Snapshot-Robustness (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/vector/snapshot.rs:117-124` | Max-length guard (1024 bytes) on `u32` length prefix before buffer allocation |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/vector/snapshot.rs:131-137` | `String::from_utf8()` with `.map_err()` instead of `from_utf8_lossy` — strict UTF-8 |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:455-458` | Open file first, then `metadata()` check on opened handle |

### Bug-Efficient-Cache (1 req)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:797-820` | Lazy per-entry TTL check: `cache.retain()` with per-entry `expired` check — O(1) instead of O(n) |

### Bug-Permissions-Hardening (4 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:65-66` | `set_permissions(0o700)` on TempDirGuard temp directory |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:58-59` | `set_permissions(0o700)` on Tantivy index directory after creation |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:195-196` (save method) | `set_permissions(0o600)` on snapshot output file |
| **REQ-FIX-004** | ⚠️ PARTIAL | `contexter-core/tests/storage/rocksdb_test.rs` | The test `test_read_only_path_error` does not exist in the codebase. The 0o700 permissions make directories writable, so the original test premise is invalid. No replacement test exists to verify 0o700 behavior. |

### Bug-Analytics-Sync (1 req)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:340-344` | `if created_at.is_empty()` skip + log warning before CAST |

### Bug-API-Conformance (4 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/search.rs:30-38` | Fields: `query_text`, `query_vector`, `text_weight`, `top_k` (no `sort_field`, no `agent_id`) |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/fts/schema.rs:216-224` | Entity-specific schemas for `session`, `agent`, `skill` with appropriate fields and boosts |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/engine/memory.rs:33` | `create_memory` uses `self.cache.invalidate(&key)` — cache-invalidate, not write-through |
| **REQ-FIX-004** | ✅ MATCHED | `contexter-core/src/fts/schema.rs:46` | Memory schema: content=1.0, tags=1.5 — no title:2.0 boost |

### Bug-Perf-Queryparser (1 req)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:26-33,67-68,105-113,225-227` | `QueryParser` built once in constructor via `build_query_parser()`, stored as `query_parser` field, reused across `search()` calls |

### Bug-HNSW-Batch-Insert (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:167-214` | `insert_batch(&self, new_embeddings)` builds graph once for all embeddings |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:471-482` | `load_snapshot` stores all embeddings at once then calls `self.rebuild()` once — avoids O(n²) per-item rebuild |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/vector/hnsw.rs:367` | Single `insert()` API preserved |

### Bug-Startup-Rebuild-Check (1 req)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:307-325` | **FIXED in Iteration 2.** After loading vector index snapshot, L2 memory count compared with HNSW entry count; warning logged on mismatch |

### Bug-DuckDB-Concurrency (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/storage/mod.rs:183`, `contexter-core/src/storage/rocksdb.rs:795`, `contexter-core/src/engine/memory.rs:153`, `contexter-core/src/engine/search.rs:207` | `fn get_memories(&self, ids: &[Uuid])` added to `StorageBackend`; hybrid search uses batch fetch instead of individual `get_memory` calls |
| **REQ-FIX-002** | ❌ UNMATCHED | `contexter-core/src/analytics/duckdb.rs:111-130` | Single `conn: Mutex<Connection>` — no read-write connection split. Doc comments describe split but struct only has one Mutex |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:9-10,127-129,290-430` | Incremental sync: `last_sync_timestamp` tracked per table; `INSERT OR REPLACE` (upsert) instead of truncate+re-insert |

### Bug-Engine-Drop (3 reqs)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:451-455` | `impl Drop for Engine { fn drop(&mut self) { let _ = self.shutdown(); } }` |
| **REQ-FIX-002** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:239,419-439` | `snapshot_handle: Option<JoinHandle<>>` + `take()` pattern — idempotent shutdown |
| **REQ-FIX-003** | ✅ MATCHED | `contexter-core/src/engine/mod.rs:427` | `if let Some(handle) = self.snapshot_handle.take() { handle.join()... }` — joins thread before return |

---

## 02 · Implementation Mapping

### L3: Vector Index

| File | Key Implementations |
|------|-------------------|
| `contexter-core/src/vector/mod.rs` | `VectorIndex` trait (insert/search/remove/save_snapshot/load_snapshot/len/is_empty) |
| `contexter-core/src/vector/hnsw.rs` | `HnswVectorIndex` struct, `new(dim, m, ef_c, ef_s)`, `insert/search/remove/save/load_snapshot/load_or_new/periodic_snapshot/insert_batch`, auto-snapshot every 1000 mutations, poison recovery, bincode SaveData/LoadData |
| `contexter-core/src/vector/snapshot.rs` | Binary snapshot format (magic 0x484E5357, version 1), `save_snapshot_data/load_snapshot_data`, max-length guard (1024) on read_string, strict UTF-8 |
| `contexter-core/src/vector/distance.rs` | `cosine_similarity/euclidean_distance/dot_product` |
| `contexter-core/src/vector/error.rs` | `VectorError` (DimensionMismatch, InvalidMagic, VersionMismatch, etc.) |

### L4: Full-Text Search

| File | Key Implementations |
|------|-------------------|
| `contexter-core/src/fts/mod.rs` | `FullTextSearch` trait, `TextContent` trait, `FieldValue` struct |
| `contexter-core/src/fts/tantivy.rs` | `TantivyIndex` struct, cached `QueryParser`, `open(path, entity_type)`, `FullTextSearch` impl, `add_alias/list_aliases/switch_index`, field boosts, 16+ tests |
| `contexter-core/src/fts/schema.rs` | `EntitySchema` with per-entity schemas (memory: content=1.0/tags=1.5; session: content/project; agent: name/description/capabilities; skill: name/description/category) |
| `contexter-core/src/fts/error.rs` | `FtsError` enum |

### L5: Analytics

| File | Key Implementations |
|------|-------------------|
| `contexter-core/src/analytics/mod.rs` | `AnalyticsEngine` trait, `Value` enum, `AnalyticsResult` alias |
| `contexter-core/src/analytics/duckdb.rs` | `DuckDbEngine`, `EFFICIENCY_CF`, `efficiency_cache` with per-entry TTL, `sync_from_backend()` with incremental sync, `TempDirGuard` with UUID naming + 0o700 + Drop cleanup, `value_to_duckdb()` parameter binding, `get_cached_efficiency_scores` (O(1) per-entry TTL), 16+ tests |
| `contexter-core/src/analytics/queries.rs` | 5 predefined SQL queries |
| `contexter-core/src/analytics/sync.rs` | `TableSchema`, `table_schemas()` |
| `contexter-core/src/analytics/error.rs` | `AnalyticsError` enum |

### Engine (Integration)

| File | Key Implementations |
|------|-------------------|
| `contexter-core/src/engine/mod.rs` | `EngineConfig` (all flags + HNSW params), `Engine` struct (L3/L4/L5 Option fields), `with_config()` (validation, tier construction, **startup consistency check**, wiring), `shutdown()` (save + cancel + join), `Drop` impl, poison recovery |
| `contexter-core/src/engine/search.rs` | `HybridSearchQuery` (query_text, query_vector, vector_weight, text_weight, top_k), `RRF_K=60`, `hybrid_search()` (batch memory fetch, RRF merge, weighted blend, in-memory filters), input validation (clamp/cap/empty), 21+ tests |
| `contexter-core/src/engine/analytics.rs` | `run_analytics()`, `get_efficiency_scores()`, `get_metric_correlation()` |
| `contexter-core/src/engine/memory.rs` | `create_memory` (L1 cache-invalidate, L3 insert, L4 index), `delete_memory` (L3 remove, L4 delete), `update_memory`, `get_memories` (batch fetch with cache-first) |
| `contexter-core/src/error/mod.rs` | `EngineError` with `InvalidConfig`, `UnsupportedOperation`, `sanitized()` |

### Storage

| File | Key Implementations |
|------|-------------------|
| `contexter-core/src/storage/rocksdb.rs` | `get_memories()` batch fetch, 0o700 permissions on data directory |
| `contexter-core/src/storage/mod.rs` | `StorageBackend` trait with `get_memories()` |

---

## 03 · Unmatched Requirements

### REQ-FIX-002 (Bug-DuckDB-Concurrency) — Split DuckDB Connection

**Severity:** MEDIUM

**SPEC text:** "Replace the single `Mutex<Connection>` with a read-write split: one read connection (not locked for writes) and one write connection. Reads use the read connection (no contention); sync uses the write connection."

**Current implementation:** `DuckDbEngine` at `contexter-core/src/analytics/duckdb.rs:111-115` has only a single `conn: Mutex<Connection>` field. The struct doc comments (lines 105-110) describe a theoretical read-write split with separate mutexes, but the actual code never implements separate read/write connections. All reads and writes share the same Mutex, meaning sync operations block all queries and vice versa.

**Fix:** Add a second `read_conn: Mutex<Connection>` field (or use `RwLock<Connection>`), open a second read-only DuckDB connection in `new()`, route `AnalyticsEngine::query()` to the read connection and `sync()/sync_all()` to the write connection.

---

## 04 · Partially Matched Requirements

### REQ-FIX-004 (Bug-Permissions-Hardening) — Fix test_read_only_path_error

**Severity:** MEDIUM

**SPEC text:** "Fix `test_read_only_path_error` in `tests/storage/rocksdb_test.rs` to account for the new `0o700` permission behavior (the test expected an error on read-only dir but `0o700` makes it writable first)."

**What's implemented:** The behavioral fix is applied — `RocksDbBackend::open_with_config()` creates directories with 0o700 permissions (noted in Iteration 1 report). The test `test_read_only_path_error` no longer exists in the codebase (confirmed: `contexter-core/tests/storage/rocksdb_test.rs` has no such test).

**What's missing:** There is no replacement test that validates the 0o700 permission behavior. No test asserts that the directory is created with restrictive permissions. The SPEC says "fix" the test, not "delete" it — a replacement test was expected.

**Impact:** Low — the behavioral fix is correct (directories are created with 0o700). But the test contract from the bug SPEC is partially unmet.

---

## 05 · Constraint Violations

| Constraint | Status | Notes |
|-----------|--------|-------|
| **CON-001**: No external processes | ✅ Compliant | L3/L4/L5 all in-process |
| **CON-002**: L3 snapshot backward-compatible | ✅ Compliant | Version field in header; validation on load |
| **CON-003**: L5 ephemeral (never persisted) | ✅ Compliant | DuckDB is in-memory file-backed; data synced from RocksDB |
| **CON-004**: Hybrid search does not degrade non-hybrid | ✅ Compliant | `search_memories()` bypasses hybrid path |
| **CON-005**: Tantivy index directory created if absent | ✅ Compliant | `TantivyIndex::open()` creates parent directories |

No constraint violations.

---

## 06 · Edge Case Verification

| Edge Case Context | Status | Notes |
|-------------------|--------|-------|
| Vector NaN/Inf rejection | ✅ Covered | `validate_vector()` in hnsw.rs, tests for NaN/Inf |
| Empty k=0 search | ✅ Covered | `search()` returns empty Vec for k=0 |
| Dimension mismatch | ✅ Covered | Both insert and search check dimensions |
| Missing snapshot file | ✅ Covered | `load_or_new()` returns empty index if path doesn't exist |
| Corrupt snapshot | ✅ Covered | `load_snapshot()` errors on corrupt data; max-length guard + strict UTF-8 |
| Empty query hybrid search | ✅ Covered | Returns Validation error |
| limit=0 hybrid search | ✅ Covered | Returns empty Vec |
| limit>1000 capped | ✅ Covered | `query.top_k.min(1000)` |
| Empty/whitespace sort_field | ✅ Covered | Treated as no sort |
| vector_weight out of range | ✅ Covered | Clamped to [0.0, 1.0] |
| Engine tiers disabled | ✅ Covered | All options default to false |
| Poisoned mutex recovery | ✅ Covered | `unwrap_or_else(\|e\| e.into_inner())` everywhere |
| Analytics temp file permissions | ✅ Fixed in Iteration 2 | 0o700 on TempDirGuard |
| L2/L3 startup consistency | ✅ Fixed in Iteration 2 | Count comparison + warning |
| Analytics sync missing created_at | ✅ Covered | Skip + log warning |
| Snapshot read_string OOM | ✅ Covered | Max-length guard 1024 bytes |
| Snapshot string corruption | ✅ Covered | Strict UTF-8, not lossy |
| Snapshot TOCTOU | ✅ Covered | Open then metadata on handle |
| Batch insert memory | ✅ Covered | `insert_batch()` builds graph once |
| Cached QueryParser | ✅ Covered | Built once, reused |
| Engine Drop safety | ✅ Covered | `Drop` impl calls shutdown, idempotent |
| Missing created_at in analytics | ✅ Covered | Skip + log warning |

---

## 07 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | YES |
| Zero findings are being silently deferred to a future iteration | YES (existing findings have bug contracts; see below) |

**Note on Iteration 2 findings:**
- **REQ-FIX-002 (Bug-DuckDB-Concurrency)** — DuckDB connection split not implemented. This is a NEW finding discovered in Iteration 2 because Bug-DuckDB-Concurrency was newly added for this iteration. It requires a new bug contract.
- **REQ-FIX-004 (Bug-Permissions-Hardening)** — Test fix not completed. This is a CARRYOVER finding from Iteration 1 that was already noted but not fully resolved.

---

## 08 · Summary

> **SPEC Compliance Assessment**
> 90 of 92 requirements (97.8%) are fully matched with implementation code. One requirement (REQ-FIX-002 from Bug-DuckDB-Concurrency) is unmatched: the DuckDB connection split is not implemented — only a single `Mutex<Connection>` exists. One requirement (REQ-FIX-004 from Bug-Permissions-Hardening) is partially matched: the behavioral 0o700 permissions are applied but the `test_read_only_path_error` test was removed without a replacement.

> **Findings**
> | # | Finding | Severity | Category |
> |---|---------|----------|----------|
> | 1 | `DuckDbEngine` has only a single `Mutex<Connection>` — read-write connection split not implemented. Reads blocked during sync writes. | MEDIUM | UNMATCHED REQ-FIX-002 (Bug-DuckDB-Concurrency) |
> | 2 | `test_read_only_path_error` does not exist in the codebase — no replacement test validates 0o700 permission behavior. | LOW | PARTIAL REQ-FIX-004 (Bug-Permissions-Hardening) |

---

## 09 · Final Verdict

| Criterion | Result |
|---|---|
| All REQ-XXX matched with implementation code | ❌ (1 unmatched, 1 partial) |
| All CON-XXX constraints respected | ✅ |
| All EDGE_CASES covered by implementation or tests | ⚠️ (permissions test gap; DuckDB concurrency gap) |
| Carryover declaration clean | ✅ |
| **Overall** | **❌ FAIL** |

---

## Appendix A: Requirements Count

| Scope | Total | ✅ MATCHED | ⚠️ PARTIAL | ❌ UNMATCHED |
|-------|-------|-----------|------------|------------|
| Parent SPEC (L3/L4/L5/Hybrid/Efficiency/Engine) | 35 | 35 | 0 | 0 |
| Bug-Validation | 2 | 2 | 0 | 0 |
| Bug-Search-Validation | 4 | 4 | 0 | 0 |
| Bug-HNSW-Config | 3 | 3 | 0 | 0 |
| Bug-DB-Analytics | 3 | 3 | 0 | 0 |
| Bug-FTS | 5 | 5 | 0 | 0 |
| Bug-Poison | 2 | 2 | 0 | 0 |
| Bug-Errors | 4 | 4 | 0 | 0 |
| Bug-File-Security | 2 | 2 | 0 | 0 |
| Bug-Efficiency | 4 | 4 | 0 | 0 |
| Bug-Snapshot | 3 | 3 | 0 | 0 |
| Bug-Test-Flakiness | 1 | 1 | 0 | 0 |
| Bug-Snapshot-Robustness | 3 | 3 | 0 | 0 |
| Bug-Efficient-Cache | 1 | 1 | 0 | 0 |
| Bug-Permissions-Hardening | 4 | 3 | 1 | 0 |
| Bug-Analytics-Sync | 1 | 1 | 0 | 0 |
| Bug-API-Conformance | 4 | 4 | 0 | 0 |
| Bug-Perf-Queryparser | 1 | 1 | 0 | 0 |
| Bug-HNSW-Batch-Insert | 3 | 3 | 0 | 0 |
| Bug-Startup-Rebuild-Check | 1 | 1 | 0 | 0 |
| Bug-DuckDB-Concurrency | 3 | 2 | 0 | 1 |
| Bug-Engine-Drop | 3 | 3 | 0 | 0 |
| **Total** | **92** | **90** | **1** | **1** |

---

### Iteration 2 Changes from Iteration 1

| Finding from Iteration 1 | Status in Iteration 2 | Notes |
|--------------------------|----------------------|-------|
| REQ-VEC-006: Startup consistency check missing | ✅ RESOLVED | `contexter-core/src/engine/mod.rs:307-325` — L2 vs HNSW count comparison |
| REQ-FIX-001 (Bug-File-Security): TempDirGuard 0o700 | ✅ RESOLVED | `contexter-core/src/analytics/duckdb.rs:65-66` — `set_permissions(0o700)` |
| REQ-FIX-004 (Bug-Permissions-Hardening): test_read_only_path_error | ⚠️ STILL PARTIAL | Test removed, no replacement |

### New Findings in Iteration 2

| Finding | Bug Contract | Notes |
|---------|-------------|-------|
| REQ-FIX-002 (Bug-DuckDB-Concurrency) — connection split not implemented | Needs new bug contract | Single `Mutex<Connection>` blocks reads during sync |
| REQ-FIX-004 (Bug-Permissions-Hardening) — missing replacement test | Bug-Permissions-Hardening exists, needs update | `test_read_only_path_error` removed without replacement |

---

_Generated by SPEC Compliance Validator · 2026-07-25 · Validation Contract: contexter-phase2-search-analytics_
