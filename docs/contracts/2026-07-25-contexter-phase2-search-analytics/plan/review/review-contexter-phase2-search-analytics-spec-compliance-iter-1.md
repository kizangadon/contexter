# SPEC Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Hybrid search (L3 HNSW + L4 Tantivy) + L5 DuckDB analytics engine wiring, 10 bug fixes

**Verdict:** FAIL (class: HARD)

2026-07-25 · 65/66 requirements matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-VEC-001** | ✅ MATCHED | `src/vector/mod.rs:21-55` | `VectorIndex` trait with `insert()`, `search()`, `remove()`, `save_snapshot()`, `load_snapshot()` methods |
| **REQ-VEC-002** | ✅ MATCHED | `src/vector/hnsw.rs:83-99` | `HnswVectorIndex::new()` takes M, ef_construction, ef_search params; defaults 16, 200, 50 |
| **REQ-VEC-003** | ✅ MATCHED | `src/vector/distance.rs:11-19`, `src/vector/hnsw.rs:39` | `cosine_similarity()` default; `euclidean_distance()` and `dot_product()` also implemented |
| **REQ-VEC-004** | ✅ MATCHED | `src/vector/snapshot.rs:32-47,138-223` | Binary snapshot with magic `0x484E5357` ("HNSW"), version 1, dimension, element_count, embeddings, removed set |
| **REQ-VEC-005** | ✅ MATCHED | `src/vector/hnsw.rs:146-157`, `src/engine/mod.rs:389-412` | `check_auto_snapshot()` triggers every 1000 mutations; `Engine::shutdown()` saves on shutdown |
| **REQ-VEC-006** | ⚠️ PARTIAL | `src/engine/mod.rs:293-299` | Loads snapshot from `snapshot_path` on startup, but **no L2 memory count mismatch check/rebuild** as described |
| **REQ-VEC-007** | ✅ MATCHED | `src/engine/mod.rs:159` | `vector_dimension: u32`, default 384 in `EngineConfig::default()` |
| **REQ-VEC-008** | ✅ MATCHED | `src/engine/mod.rs:286-303` | Without `snapshot_path`, index runs in-memory only |
| **REQ-VEC-009** | ✅ MATCHED | `src/vector/hnsw.rs:377-383` | `remove(&self, id)` — logical deletion via removed set |
| **REQ-FTS-001** | ✅ MATCHED | `src/fts/mod.rs:39-51`, `src/fts/tantivy.rs:120-228` | `FullTextSearch` trait with `index/search/delete/flush`; `TantivyIndex` implements all |
| **REQ-FTS-002** | ✅ MATCHED | `src/fts/schema.rs:86-91` | `schema_for_entity("memory")` returns full schema; default for others |
| **REQ-FTS-003** | ✅ MATCHED | `src/fts/tantivy.rs:166-179` | Field boosts: content=1.0, title=2.0, tags=1.5 |
| **REQ-FTS-004** | ✅ MATCHED | `src/fts/tantivy.rs:186-188`, test `test_phrase_search:324-345` | Tantivy `QueryParser` handles phrase, fuzzy, boolean operators |
| **REQ-FTS-005** | ✅ MATCHED | `src/engine/memory.rs:43-70` | `create_memory()` indexes into FTS on write |
| **REQ-FTS-006** | ✅ MATCHED | `src/engine/mod.rs:179` | `tantivy_path: Option<PathBuf>` in `EngineConfig` |
| **REQ-FTS-007** | ✅ MATCHED | Tantivy native | Tantivy handles auto segment merging internally |
| **REQ-ANA-001** | ✅ MATCHED | `src/analytics/mod.rs:57-72` | `AnalyticsEngine` trait with `query/sync/sync_all/set_storage_backend` |
| **REQ-ANA-002** | ✅ MATCHED | `src/analytics/duckdb.rs:108-151` | DuckDB in-memory engine; `sync_from_backend()` iterates RocksDB CFs |
| **REQ-ANA-003** | ✅ MATCHED | `src/analytics/queries.rs` | `SESSION_COUNT_BY_RANGE`, `MEMORY_COUNT_BY_TYPE`, `TELEMETRY_AGGREGATION`, `EFFICIENCY_SCORES`, `METRIC_CORRELATION` |
| **REQ-ANA-004** | ✅ MATCHED | `src/analytics/duckdb.rs:449-477` | `query()` auto-syncs tables when TTL expired |
| **REQ-ANA-005** | ✅ MATCHED | `src/engine/mod.rs:182,202` | `analytics_cache_ttl_secs: u64`, default 300 |
| **REQ-HYB-001** | ✅ MATCHED | `src/engine/search.rs:111-252` | `hybrid_search()` merges L3 + L4 results |
| **REQ-HYB-002** | ✅ MATCHED | `src/engine/search.rs:33-35,49-63` | `vector_weight: f32` default 0.5 ⇒ text_weight = 1.0 - vector_weight |
| **REQ-HYB-003** | ✅ MATCHED | `src/engine/search.rs:70` | `RRF_K: f32 = 60.0` |
| **REQ-HYB-004** | ✅ MATCHED | `src/engine/search.rs:148-252` | HashMap dedup + RRF composite score + sort_by + truncate |
| **REQ-HYB-005** | ✅ MATCHED | `src/engine/search.rs:217-245` | In-memory filtering: memory_type, tags, session_id, agent_id |
| **REQ-EFF-001** | ✅ MATCHED | `src/analytics/queries.rs:41-55` | EFFICIENCY_SCORES SQL: useful_memories / total_memories per session |
| **REQ-EFF-002** | ✅ MATCHED | `src/analytics/queries.rs:58-82` | METRIC_CORRELATION SQL: Pearson r between duration_ms and memory_count |
| **REQ-EFF-003** | ✅ MATCHED | `src/storage/column_families.rs` (CF_EFFICIENCY_MAP) | `efficiency_map` column family defined |
| **REQ-EFF-004** | ✅ MATCHED | `src/analytics/duckdb.rs:599-691` | `efficiency_cache: HashMap<String, EfficiencyEntry>` with per-session granularity + TTL |
| **REQ-ENG-001** | ✅ MATCHED | `src/engine/mod.rs:226-240` | `Engine` struct with `vector_index: Option<Arc<dyn VectorIndex>>`, `fts_index`, `analytics_engine` |
| **REQ-ENG-002** | ✅ MATCHED | `src/engine/memory.rs:36-70` | L3 `vx.insert()` + L4 `fts.index()` in `create_memory()` |
| **REQ-ENG-003** | ✅ MATCHED | `src/engine/analytics.rs:52-96` | `Engine::run_analytics()` returns `AnalyticsReport` |
| **REQ-ENG-004** | ✅ MATCHED | `src/engine/mod.rs:189-206` | All defaults: `enable_vector_index: false`, `enable_fulltext_search: false`, `enable_analytics: false` |
| **REQ-ENG-005** | ✅ MATCHED | `src/engine/mod.rs:326` | `engine.set_storage_backend(Box::new(storage.clone()))` |
| **REQ-FIX-001 (Bug-Validation)** | ✅ MATCHED | `src/engine/mod.rs:272-276` | Guard: `config.vector_dimension == 0` → `Err(EngineError::InvalidConfig(...))` |
| **REQ-FIX-002 (Bug-Validation)** | ✅ MATCHED | `src/error/mod.rs:44-46` | `InvalidConfig(String)` variant with Display + sanitized |
| **REQ-FIX-001 (Bug-Search-Validation)** | ✅ MATCHED | `src/engine/search.rs:129` | `vector_weight.clamp(0.0, 1.0)` |
| **REQ-FIX-002 (Bug-Search-Validation)** | ✅ MATCHED | `src/engine/search.rs:130-134` | `if query.limit == 0 { return Ok(Vec::new()) }` else `query.limit.min(1000)` |
| **REQ-FIX-003 (Bug-Search-Validation)** | ✅ MATCHED | `src/engine/search.rs:137-141` | Empty/whitespace-only sort_field treated as no sort |
| **REQ-FIX-004 (Bug-Search-Validation)** | ✅ MATCHED | `src/engine/search.rs:1043-1305` | 7 tests: `test_hybrid_search_weight_clamped_low`, `_high`, `_limit_zero`, `_limit_capped`, `_sort_field_empty`, `_sort_field_whitespace`, `_sort_field_none` |
| **REQ-FIX-001 (Bug-HNSW-Config)** | ✅ MATCHED | `src/engine/mod.rs:168-176,196-198` | `hnsw_m`, `hnsw_ef_construction`, `hnsw_ef_search` in `EngineConfig` |
| **REQ-FIX-002 (Bug-HNSW-Config)** | ✅ MATCHED | `src/engine/mod.rs:287-292` | `HnswVectorIndex::new(dim, config.hnsw_m, config.hnsw_ef_construction, config.hnsw_ef_search)` |
| **REQ-FIX-003 (Bug-HNSW-Config)** | ✅ MATCHED | `src/vector/hnsw.rs:83-99,132-134` | `Builder::default().ef_construction(ef_construction).ef_search(ef_search)` |
| **REQ-FIX-001 (Bug-DB-Analytics)** | ✅ MATCHED | `src/analytics/duckdb.rs:484-493` | `value_to_duckdb()` converts `Value` → duckdb types; `stmt.query(&param_refs[..])` |
| **REQ-FIX-002 (Bug-DB-Analytics)** | ✅ MATCHED | `src/engine/mod.rs:326` | `engine.set_storage_backend(Box::new(storage.clone()))` |
| **REQ-FIX-003 (Bug-DB-Analytics)** | ✅ MATCHED | `src/analytics/duckdb.rs:217-365` | `sync_from_backend()` iterates RocksDB CF: sessions, memories, telemetry |
| **REQ-FIX-001 (Bug-FTS)** | ✅ MATCHED | `src/fts/tantivy.rs:120-228` | `FullTextSearch` fully implemented: `index/search/delete/flush` |
| **REQ-FIX-002 (Bug-FTS)** | ✅ MATCHED | `src/fts/schema.rs:35-36` | `title_field` and `tags_field` added to memory schema |
| **REQ-FIX-003 (Bug-FTS)** | ✅ MATCHED | `src/models/memory.rs:53-62` | `impl TextContent for Memory` — concatenates content + tags |
| **REQ-FIX-004 (Bug-FTS)** | ✅ MATCHED | `src/engine/mod.rs:309` | `TantivyIndex::open(path, "memory")` receives `tantivy_path` from config |
| **REQ-FIX-005 (Bug-FTS)** | ✅ MATCHED | `src/fts/tantivy.rs:84-117` | `add_alias/ list_aliases/ switch_index` implemented, 5 tests in file |
| **REQ-FIX-001 (Bug-Poison)** | ✅ MATCHED | `src/analytics/duckdb.rs:155,165,218,...` | `.unwrap_or_else(|e| e.into_inner())` on all `Mutex` accesses in DuckDbEngine |
| **REQ-FIX-002 (Bug-Poison)** | ✅ MATCHED | `src/engine/mod.rs:84,132,137,...`, `src/engine/search.rs:84,91`, `src/engine/memory.rs:29,89,121,167` | `unwrap_or_else(|e| e.into_inner())` on all RwLock/Mutex in Engine |
| **REQ-FIX-001 (Bug-Errors)** | ✅ MATCHED | All `src/engine/*.rs` | No bare `unwrap()` in non-test engine code; all use `?` or `unwrap_or_else(|e| e.into_inner())` |
| **REQ-FIX-002 (Bug-Errors)** | ✅ MATCHED | `src/error/mod.rs:52-54` | `UnsupportedOperation(String)` variant with Display + sanitized |
| **REQ-FIX-003 (Bug-Errors)** | ✅ MATCHED | See Bug-Poison entries above | Mutex/RwLock poison recovery throughout engine |
| **REQ-FIX-004 (Bug-Errors)** | ✅ MATCHED | `src/analytics/duckdb.rs:44-68` | `TempDirGuard` struct with `Drop` impl cleans up temp directory |
| **REQ-FIX-001 (Bug-File-Security)** | ❌ UNMATCHED | `src/analytics/duckdb.rs:50-53` | `TempDirGuard::new()` uses `create_dir_all` with default permissions; **no 0o700** on analytics temp directory |
| **REQ-FIX-002 (Bug-File-Security)** | ✅ MATCHED | `src/vector/hnsw.rs:397-411` | `load_snapshot()` does `std::fs::metadata(path)` check → directory/empty checks before open (TOCTOU mitigation) |
| **REQ-FIX-001 (Bug-Efficiency)** | ✅ MATCHED | `src/analytics/duckdb.rs:27` | `pub const EFFICIENCY_CF: &str = "efficiency_map";` |
| **REQ-FIX-002 (Bug-Efficiency)** | ✅ MATCHED | `src/analytics/duckdb.rs:546-556` | `sync()` checks `cf_name == EFFICIENCY_CF` → calls `sync_efficiency_cache_from_backend()` |
| **REQ-FIX-003 (Bug-Efficiency)** | ✅ MATCHED | `src/analytics/duckdb.rs:92,473-477,599-643` | `efficiency_cache` field; `query()` checks cache before DuckDB; `populate_efficiency_cache()` |
| **REQ-FIX-004 (Bug-Efficiency)** | ✅ MATCHED | `src/analytics/duckdb.rs:607-609` | `now.duration_since(entry.cached_at).as_secs() > self.cache_ttl_secs` TTL eviction |
| **REQ-FIX-001 (Bug-Snapshot)** | ✅ MATCHED | `src/vector/hnsw.rs:168-201` | `pub fn save(&self, path: &Path)` with atomic write (tmp + rename) |
| **REQ-FIX-002 (Bug-Snapshot)** | ✅ MATCHED | `src/vector/hnsw.rs:254-271`, `src/engine/mod.rs:334-366` | `periodic_snapshot()` thread; wired in `Engine::with_config()` with `snapshot_interval_secs` |
| **REQ-FIX-003 (Bug-Snapshot)** | ✅ MATCHED | `src/engine/mod.rs:389-412` | `Engine::shutdown()` calls `save_snapshot()`, signals cancel, joins thread |

---

## 02 · Implementation Mapping

### Parent Feature (SPEC.md) — Engine & Hybrid Search

| File | Key Implementations |
|------|-------------------|
| `src/engine/mod.rs` | `EngineConfig` (all tier flags + HNSW params), `Engine` struct (L3/L4/L5 Option fields), `with_config()` (validates, constructs, wires tiers), `shutdown()` (save + cancel + join snapshot), poison recovery on all locks |
| `src/engine/search.rs` | `HybridSearchQuery` struct, `RRF_K=60`, `hybrid_search()` (L3+L4 merge, RRF scoring, weighted blend, in-memory filters), input validation (clamp/cap/whitespace), 21 tests |
| `src/engine/analytics.rs` | `AnalyticsReport`, `SessionEfficiency`, `MetricCorrelation` — `run_analytics()`, `get_efficiency_scores()`, `get_metric_correlation()`, `get_session_count_by_range()`, `get_memory_count_by_type()` |
| `src/engine/memory.rs` | `create_memory()` → L3 `vx.insert()` + L4 `fts.index()`, `delete_memory()` → L3 remove + L4 delete, `update_memory()` → L4 re-index, all with poison recovery |
| `src/engine/session.rs` | Session CRUD with poison recovery |
| `src/engine/agent.rs` | Agent CRUD with poison recovery |
| `src/engine/skill.rs` | Skill CRUD with poison recovery |
| `src/engine/settings.rs` | Settings CRUD with poison recovery |
| `src/engine/maintenance.rs` | Maintenance operations with poison recovery |

### L3: Vector Index

| File | Key Implementations |
|------|-------------------|
| `src/vector/mod.rs` | `VectorIndex` trait (insert/search/remove/save_snapshot/load_snapshot/len/is_empty), `VectorIndexResult` alias |
| `src/vector/hnsw.rs` | `HnswVectorIndex` struct, `new(dim, m, ef_c, ef_s)`, `insert/search/remove/save/load_snapshot/load_or_new/periodic_snapshot`, auto-snapshot every 1000 mutations, poison recovery, SaveData/LoadData bincode |
| `src/vector/snapshot.rs` | Binary snapshot format (magic `0x484E5357`, version 1, dimension, element_count, m, ef_construction), `save_snapshot_data()`, `load_snapshot_data()`, `SnapshotHeader` |
| `src/vector/distance.rs` | `cosine_similarity()`, `euclidean_distance()`, `dot_product()` |
| `src/vector/error.rs` | `VectorError` (DimensionMismatch, InvalidMagic, VersionMismatch, InvalidVector, Io, Bincode, Internal, EmptySnapshot) |

### L4: Full-Text Search

| File | Key Implementations |
|------|-------------------|
| `src/fts/mod.rs` | `FullTextSearch` trait, `TextContent` trait, `FieldValue` struct, `FtsResult` alias |
| `src/fts/tantivy.rs` | `TantivyIndex` struct, `open(path, entity_type)`, `open_in_memory()`, `FullTextSearch` impl (index/search/delete/flush), `add_alias/list_aliases/switch_index`, field boosting (content=1.0, title=2.0, tags=1.5), 16 tests |
| `src/fts/schema.rs` | `EntitySchema` (id_field, content_field, title_field, tags_field, entity_type_field), `schema_for_entity("memory"/"memories")` with title+tags, default without |
| `src/fts/error.rs` | `FtsError` enum |
| `src/fts/query.rs` | Query utilities |

### L5: Analytics

| File | Key Implementations |
|------|-------------------|
| `src/analytics/mod.rs` | `AnalyticsEngine` trait, `Value` enum (Null/Bool/Int/Float/Text), `AnalyticsResult` alias |
| `src/analytics/duckdb.rs` | `DuckDbEngine` struct, `new(ttl)`, `AnalyticsEngine` impl (query/sync/sync_all/set_storage_backend), `EFFICIENCY_CF` constant, `efficiency_cache` with TTL, `sync_from_backend()` (sessions/memories/telemetry from RocksDB), `sync_efficiency_cache_from_backend()`, `get_cached_efficiency_scores()`, `populate_efficiency_cache()`, `TempDirGuard`, `value_to_duckdb()` parameter binding, 16 tests |
| `src/analytics/queries.rs` | 5 predefined SQL queries (session count by range, memory count by type, telemetry agg, efficiency scores, metric correlation) |
| `src/analytics/sync.rs` | `TableSchema` struct, `table_schemas()` (sessions/memories/telemetry with source_cf mappings) |
| `src/analytics/error.rs` | `AnalyticsError` enum |

### Error Handling

| File | Key Implementations |
|------|-------------------|
| `src/error/mod.rs` | `EngineError` (Storage, NotFound, Validation, Serialization, Compression, Cache, Internal, InvalidConfig, Unimplemented, **UnsupportedOperation**), `sanitized()`, `EngineResult` alias, 17 tests |

### Models

| File | Key Implementations |
|------|-------------------|
| `src/models/memory.rs` | `Memory` struct with `impl TextContent`, `NewMemory`, `MemoryPatch`, `MemorySearchQuery`, `MemoryFilter`, `MemoryType` enum, TextContent tests |

### Storage

| File | Key Implementations |
|------|-------------------|
| `src/storage/rocksdb.rs` | `RocksDbBackend::open_with_config()` with **0o700 permissions** on data directory |
| `src/storage/mod.rs` | `StorageBackend` trait, `SharedBackend` type alias |

---

## 03 · Unmatched Requirements

### REQ-FIX-001 (Bug-File-Security) — Set restrictive permissions on analytics temp files

**Severity:** MEDIUM

**SPEC text:** "When creating temp directories for analytics, set umask or explicitly chmod to 0o700 to prevent other users from reading temp data."

**Evidence of gap:** `TempDirGuard::new()` at `src/analytics/duckdb.rs:50-53` calls `std::fs::create_dir_all(&dir)?` without any `set_permissions` or `umask` call. The directory is created with the process's default umask permissions (typically 0o755 or 0o775).

Note: 0o700 permissions ARE applied to the RocksDB data directory at `src/storage/rocksdb.rs:186`, but the bug SPEC specifically targets the **analytics temp directory** created by `TempDirGuard`. The analytics temp directory remains unprotected.

**Fix:** Add `std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))` after `create_dir_all` in `TempDirGuard::new()`.

---

## 04 · Partially Matched Requirements

### REQ-VEC-006 — Startup snapshot loading with L2 mismatch rebuild

**Severity:** LOW

**SPEC text:** "On startup, load snapshot from ~/.contexter/vector_index.bin; rebuild if memory count in L2 doesn't match index entry count"

**What's implemented:** `Engine::with_config()` at `src/engine/mod.rs:293-299` loads the snapshot from `config.snapshot_path` if the file exists and ignores `NotFound` errors.

**What's missing:** There is no comparison of the "memory count in L2" (RocksDB stored memories) against the "index entry count" (HNSW snapshot element count). If these differ, the SPEC says the index should be rebuilt. The current implementation never checks this and never triggers a rebuild at startup.

**Impact:** Low — the snapshot path is configurable and the snapshot is loaded correctly. The L2 mismatch check is an additional safety net documented in the SPEC that is not implemented.

---

## 05 · Constraint Violations

| Constraint | Status | Notes |
|-----------|--------|-------|
| **CON-001**: No external processes | ✅ Compliant | L3/L4/L5 all in-process |
| **CON-002**: L3 snapshot backward-compatible | ✅ Compliant | Version field in header; validation on load |
| **CON-003**: L5 ephemeral (never persisted) | ✅ Compliant | DuckDB is in-memory only; data synced from RocksDB |
| **CON-004**: Hybrid search must not degrade non-hybrid search | ✅ Compliant | `search_memories()` bypasses hybrid path; no cross-impact |
| **CON-005**: Tantivy index directory created if absent | ✅ Compliant | `TantivyIndex::open()` creates parent directories |

No constraint violations found.

---

## 06 · Edge Case Verification

| Edge Case File | Status | Notes |
|---------------|--------|-------|
| Parent `EDGE_CASES.md` | N/A (not read — verifying SPEC compliance only) | SPEC Compliance Validator checks REQ-XXX, not EDGE_CASES |
| Bug `EDGE_CASES.md` per bug | N/A | Verifying REQ-XXX mapping to code |
| Vector NaN/Inf rejection | ✅ Covered | `validate_vector()` in hnsw.rs, tests for NaN/Inf |
| Empty k=0 search | ✅ Covered | `search()` returns empty Vec for k=0 |
| Dimension mismatch | ✅ Covered | Both insert and search check dimensions |
| Missing snapshot file | ✅ Covered | `load_or_new()` returns empty index if path doesn't exist |
| Corrupt snapshot | ✅ Covered | `load_snapshot()` errors on corrupt data |
| Empty query hybrid search | ✅ Covered | Returns Validation error |
| limit=0 hybrid search | ✅ Covered | Returns empty Vec |
| limit>1000 capped | ✅ Covered | `query.limit.min(1000)` |
| Whitespace sort_field | ✅ Covered | Treated as no sort |
| Limit analysis temp file permissions | ❌ Not covered | See Unmatched Requirements |
| Engine tiers disabled | ✅ Covered | All options default to false |
| Poisoned mutex recovery | ✅ Covered | `unwrap_or_else(|e| e.into_inner())` everywhere |

---

## 07 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | YES |
| Zero findings are being silently deferred to a future iteration | YES |

---

## 08 · Summary

> **SPEC Compliance Assessment**
> 65 of 66 requirements (98.5%) are fully matched with implementation code. One requirement (REQ-FIX-001 from Bug-File-Security) is unmatched: the analytics temp directory is created without 0o700 permissions. One requirement (REQ-VEC-006) is partially matched: the L2 memory count mismatch check is not implemented.

> **Findings**
> | # | Finding | Severity | Category |
> |---|---------|----------|----------|
> | 1 | `TempDirGuard::new()` does not set 0o700 permissions on analytics temp directory | MEDIUM | Unmatched REQ-FIX-001 (Bug-File-Security) |
> | 2 | `Engine::with_config()` does not verify L2 memory count against HNSW entry count on startup | LOW | Partially matched REQ-VEC-006 |

---

## 09 · Final Verdict

| Criterion | Result |
|---|---|
| All REQ-XXX matched with implementation code | ❌ (1 unmatched, 1 partial) |
| All CON-XXX constraints respected | ✅ |
| All EDGE_CASES covered by implementation or tests | ⚠️ (permissions gap) |
| Carryover declaration clean | ✅ |
| **Overall** | **❌ FAIL** |

---

## Appendix A: Requirements Count

| Scope | Total | ✅ MATCHED | ⚠️ PARTIAL | ❌ UNMATCHED |
|-------|-------|-----------|------------|------------|
| Parent SPEC (REQ-VEC-001..009, REQ-FTS-001..007, REQ-ANA-001..005, REQ-HYB-001..005, REQ-EFF-001..004, REQ-ENG-001..005) | 35 | 34 | 1 (REQ-VEC-006) | 0 |
| Bug-Validation (2) | 2 | 2 | 0 | 0 |
| Bug-Search-Validation (4) | 4 | 4 | 0 | 0 |
| Bug-HNSW-Config (3) | 3 | 3 | 0 | 0 |
| Bug-DB-Analytics (3) | 3 | 3 | 0 | 0 |
| Bug-FTS (5) | 5 | 5 | 0 | 0 |
| Bug-Poison (2) | 2 | 2 | 0 | 0 |
| Bug-Errors (4) | 4 | 4 | 0 | 0 |
| Bug-File-Security (2) | 2 | 1 | 0 | 1 |
| Bug-Efficiency (4) | 4 | 4 | 0 | 0 |
| Bug-Snapshot (3) | 3 | 3 | 0 | 0 |
| **Total** | **66** | **64** | **1** | **1** |

---

_Generated by SPEC Compliance Validator · 2026-07-25 · Validation Contract: contexter-phase2-search-analytics_
