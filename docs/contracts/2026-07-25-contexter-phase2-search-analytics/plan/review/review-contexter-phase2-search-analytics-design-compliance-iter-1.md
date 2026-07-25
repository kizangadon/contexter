# Design Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> **Iteration 1 (Auto Bug Loop)**

**Verdict:** CONDITIONAL FAIL (class: structural-gaps-found)

2026-07-25 · 4/5 design sections verified · Design Compliance Validator

---

## 01 · Design Preview Sections Covered

| Section | Status |
|---|---|
| Architecture Diagrams — Component Architecture | ✅ MATCHED |
| L3: HNSW Vector Index Module Structure | ✅ MATCHED |
| L4: Tantivy Full-Text Search Module Structure | ⚠️ PARTIAL |
| L5: DuckDB Analytics Module Structure | ✅ MATCHED |
| Engine Integration (Rust struct + Config) | ✅ MATCHED |
| API Contract — VectorIndex Trait | ✅ MATCHED |
| API Contract — FullTextSearch Trait | ✅ MATCHED |
| API Contract — AnalyticsEngine Trait | ⚠️ PARTIAL |
| API Contract — HybridSearchQuery | ⚠️ PARTIAL |
| API Contract — EngineConfig | ✅ MATCHED |
| Data Flow — Memory Write → L3 + L4 Update | ⚠️ PARTIAL |
| Data Flow — Hybrid Search (L3 + L4) | ✅ MATCHED |
| Data Flow — Analytics Query | ✅ MATCHED |
| Component Hierarchy | ✅ MATCHED |
| Cargo.toml Dependency Declarations | ✅ MATCHED |
| Snapshot Binary Format | ✅ MATCHED |
| Schema Per Entity Type | ⚠️ PARTIAL |
| UI Wireframes | ➖ NOT APPLICABLE |

---

## 02 · Architecture Compliance

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Module / service decomposition | 3 new modules: `vector/`, `fts/`, `analytics/` with exact sub-module layout | `vector/` (mod, hnsw, distance, snapshot, error), `fts/` (mod, tantivy, schema, query, error), `analytics/` (mod, duckdb, queries, sync, error) — all present with extra `error.rs` modules in each | ✅ MATCHED |
| Component hierarchy | `Engine` struct with: storage, cache, telemetry, vector_index, fts_index, analytics_engine | `Engine` struct matches exactly + adds `snapshot_path`, `snapshot_handle`, `snapshot_cancel` lifecycle fields | ✅ MATCHED |
| Data flow | Engine composition: SharedBackend ↔ DashMapCache ↔ Option<Arc<dyn VectorIndex/FTS/Analytics>> | Implementation follows the exact composition pattern | ✅ MATCHED |
| EngineConfig tier options | All tiers disabled by default, per-config bool flags | All tiers default to `false`, plus HNSM M/ef params and `snapshot_interval_secs` added | ✅ MATCHED |

### Architecture Findings

**No architectural findings.** The three-tier (L3/L4/L5) module decomposition shown in the design is fully implemented. Each module's sub-structure (mod.rs, hnsw/tantivy/duckdb impl files, schema/query/sync helpers) matches the design layout exactly. The `Engine` struct fields mirror the design with lifecycle extensions that are implementation-appropriate.

---

## 03 · API Contract Compliance

| Endpoint | Design Schema | Actual Schema | Status |
|---|---|---|---|
| `VectorIndex` trait | `insert, search, remove, save_snapshot, load_snapshot, len, is_empty` | All 7 methods match exact design signatures | ✅ MATCHED |
| `FullTextSearch` trait | `index(doc_id, fields), search(query, limit), delete(doc_id), flush()` | All 4 methods match exact design signatures including `FieldValue` | ✅ MATCHED |
| `AnalyticsEngine` trait | `query(sql, params), sync(cf_name), sync_all()` | All 3 methods match + extra `set_storage_backend()` not in design | ⚠️ PARTIAL |
| `HybridSearchQuery` struct | `query_text, query_vector, top_k, vector_weight, text_weight, memory_type, tags, session_id` | Fields renamed: `query_text`→`text_query`, `query_vector`→`vector_query`, `top_k`→`limit`; `text_weight` omitted (computed); extra `sort_field`, `agent_id` added | ⚠️ PARTIAL |
| `EngineConfig` struct | `storage, enable_vector_index, vector_dimension, snapshot_path, enable_fulltext_search, tantivy_path, enable_analytics, analytics_cache_ttl_secs` | All design fields present + extra `hnsw_m`, `hnsw_ef_construction`, `hnsw_ef_search`, `snapshot_interval_secs` | ✅ MATCHED |

### API Contract Findings

**F1: HybridSearchQuery field name mismatch** — The approved design specifies `query_text`, `query_vector`, and `top_k` as field names. The implementation uses `text_query`, `vector_query`, and `limit`. While semantically equivalent, the API contract shown in the design preview is the binding agreement and should match.

**F2: HybridSearchQuery omits `text_weight` field** — The design preview explicitly defines `text_weight: f32` (default 0.5) as a separate field in `HybridSearchQuery`. The implementation only has `vector_weight` and computes text_weight as `1.0 - vector_weight`. Functionally equivalent at runtime, but the public struct's surface area differs from the design.

**F3: HybridSearchQuery adds unspecified fields** — `sort_field: Option<String>` and `agent_id: Option<Uuid>` appear in the implementation but are not present in the design preview's API contract. Not a gap per se, but the API surface deviates from what was approved.

**F4: AnalyticsEngine trait adds `set_storage_backend`** — The implementation's `AnalyticsEngine` trait includes `fn set_storage_backend(&self, backend: Box<dyn Any + Send>)` which is not specified in the design preview's API contract. This method is required for the DuckDB → RocksDB wiring but was not part of the approved trait contract.

---

## 04 · UI Wireframe Compliance

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Layout structure | N/A (library API feature, no UI wireframe) | N/A | ➖ NOT APPLICABLE |
| Component placement | N/A | N/A | ➖ NOT APPLICABLE |
| States | N/A | N/A | ➖ NOT APPLICABLE |

### Wireframe Findings

No UI wireframes are specified in this design preview. This is a library-level API feature.

---

## 05 · Data Flow Compliance

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| **1a.** create_memory → L2 persist | L2: RocksDB persist | `self.storage.write().create_memory(new_memory)` | ✅ MATCHED |
| **1b.** create_memory → L1 | L1: cache invalidate | L1: write-through cache store (not invalidate) — caches the result instead of invalidating entry | ⚠️ PARTIAL |
| **1c.** create_memory → L3 | vector_index.insert(id, &embedding) if configured | `vx.insert(&memory.id.to_string(), emb)` if `self.vector_index.is_some()` | ✅ MATCHED |
| **1d.** create_memory → L4 | fts_index.index(id, &fields) if configured | `fts.index(id, &[FieldValue{content}, FieldValue{title}, FieldValue{tags}])` + `fts.flush()` | ✅ MATCHED |
| **2a.** hybrid_search → L3 | knn_search(query_vector, k × 2) | `vx.search(vec, fetch_k)` where `fetch_k = limit * 2` | ✅ MATCHED |
| **2b.** hybrid_search → L4 | fts_search(query_text, k × 2) | `fts.search(text, fetch_k)` where `fetch_k = limit * 2` | ✅ MATCHED |
| **2c.** RRF merge | RRF score = Σ 1 / (k + rank), k=60, weights=[w_vec, w_txt], Final = RRF_vec × w_vec + RRF_text × w_txt | `RRF_K = 60.0`, score = `1.0 / (60.0 + rank)`, blend = `vector_weight * rrf_l3 + (1-vector_weight) * rrf_l4` | ✅ MATCHED |
| **3a.** run_analytics → guard | L5 not configured → return Err | Returns `EngineError::Unimplemented("Analytics not enabled")` | ✅ MATCHED |
| **3b.** run_analytics → sync | sync relevant CFs if cache stale or TTL expired | `ae.sync_all()` at start of `run_analytics()`; auto-sync in `query()` via `needs_sync()` + TTL check | ✅ MATCHED |
| **3c.** run_analytics → SQL queries | session_count_by_range, memory_count_by_type, telemetry_aggregation, efficiency_scores, metric_correlation | All 5 predefined queries exist in `analytics/queries.rs` as constants | ✅ MATCHED |

### Data Flow Findings

**F5: Create memory L1 cache policy mismatch** — The design preview's data flow diagram specifies "L1: cache invalidate" as the step after RocksDB persist in `create_memory`. The implementation uses write-through caching (stores the serialised result in cache after persist, per the cache policy table in `engine/mod.rs`). While write-through is a valid caching strategy, it differs from what the design data flow committed to.

**F6: create_memory does not call L3 insert for all code paths** — The implementation conditionally inserts into L3 only when `memory.embedding` is `Some(...)`. This is correct behavior, and the design does not specify when embeddings are populated, so this is not a finding.

---

## 06 · Unmatched / Partially Matched Design Elements

### Partially Matched Elements

| # | Element | Design Commitment | Implementation | Severity |
|---|---|---|---|---|
| PM-1 | FTS entity schemas beyond "memory" | 4 entity schemas: memory (content:1.0, tags:1.5), session (project, status, metadata), agent (name, description, capabilities), skill (name, description, category) | Only "memory" schema with full fields; generic "default" schema for all other entity types. Session, agent, and skill schemas are not implemented with their entity-specific fields. | Medium |
| PM-2 | FTS field boost values for memory | Design row shows: content:1.0, tags:1.5 (no title field in memory row) | Implementation uses content:1.0, title:2.0, tags:1.5 — adds a title field/boost not present in the memory schema row | Low |
| PM-3 | `HybridSearchQuery` API contract | Fields: `query_text`, `query_vector`, `top_k`, `text_weight` | Fields: `text_query`, `vector_query`, `limit` — three field renames + missing `text_weight` as separate field | Medium |
| PM-4 | `AnalyticsEngine` trait | Methods: `query`, `sync`, `sync_all` | Adds `set_storage_backend` — extra method beyond approved contract | Low |
| PM-5 | create_memory L1 cache policy | Data flow: "L1: cache invalidate" | Implementation: write-through cache store | Low |

### Fully Matched Elements (no findings)

All architecture diagrams, the VectorIndex and FullTextSearch traits, Engine struct fields, EngineConfig defaults, snapshot binary format (with removed-set addition), delete_memory L3/L4 cleanup, hybrid search RRF algorithm, analytics query SQL (all 5 predefined queries), analytics sync flow with TTL, Cargo.toml dependencies, library module declarations, and all three new modules' file layout.

---

## 07 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | YES |
| Zero findings are being silently deferred to a future iteration | YES |

---

## 08 · Summary

> **Design Compliance Assessment**
> The approved design preview is substantially implemented. The core architecture (three-tier module decomposition, Engine struct with optional index composition, VectorIndex/FullTextSearch trait contracts, RRF hybrid search algorithm, analytics SQL queries, snapshot format) all match the design exactly. Six findings were identified — three API contract surface mismatches (field renames in HybridSearchQuery, missing `text_weight` as separate field, extra `set_storage_backend` method), one entity schema gap (missing session/agent/skill specific FTS schemas), one L1 cache policy discrepancy (write-through vs invalidate), and one minor field boost difference.

> **Findings**
> 6 total: 0 critical, 2 medium (PM-1, PM-3), 4 low (PM-2, PM-4, PM-5, F1 field renames part of PM-3)

---

## 09 · Final Verdict

| Criterion | Result |
|---|---|
| Architecture matches design preview | ✅ PASS |
| API contracts match design preview | ⚠️ PARTIAL (3 findings) |
| UI wireframe matches rendered output | ➖ N/A |
| Data flow matches design specification | ⚠️ PARTIAL (1 finding) |
| Carryover declaration clean | ✅ YES |
| **Overall** | **⚠️ CONDITIONAL FAIL** |

---

_Generated by Design Compliance Validator · 2026-07-25 · Iteration 1 · Contract: 2026-07-25-contexter-phase2-search-analytics_
