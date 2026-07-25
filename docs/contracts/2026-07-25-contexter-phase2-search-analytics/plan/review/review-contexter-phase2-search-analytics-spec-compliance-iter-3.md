# SPEC Compliance Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Hybrid search (L3 HNSW + L4 Tantivy) + L5 DuckDB analytics engine wiring, 24 bug fixes

**Verdict:** CONDITIONAL PASS (class: ACCEPTED_LIMITATION)

2026-07-25 · 94/95 requirements matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

### Parent SPEC — L3/L4/L5 Engine (35 reqs, unchanged from iter-2)

All 35 requirements remain fully matched. No changes in this iteration.

### Bug-Validation (2) · Bug-Search-Validation (4) · Bug-HNSW-Config (3) · Bug-DB-Analytics (3) · Bug-FTS (5) · Bug-Poison (2) · Bug-Errors (4) · Bug-File-Security (2) · Bug-Efficiency (4) · Bug-Snapshot (3) · Bug-Test-Flakiness (1) · Bug-Snapshot-Robustness (3) · Bug-Efficient-Cache (1) · Bug-Analytics-Sync (1) · Bug-API-Conformance (4) · Bug-Perf-Queryparser (1) · Bug-HNSW-Batch-Insert (3) · Bug-Startup-Rebuild-Check (1) · Bug-Engine-Drop (3)

All requirements across these contracts remain ✅ MATCHED as in iter-2. No changes.

### Bug-Permissions-Hardening (4 reqs) — REQ-FIX-004 resolved

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** — TempDirGuard 0o700 | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:65` | `set_permissions(0o700)` in `TempDirGuard::new()` |
| **REQ-FIX-002** — Tantivy index 0o700 | ✅ MATCHED | `contexter-core/src/fts/tantivy.rs:58` | `set_permissions(0o700)` after directory creation |
| **REQ-FIX-003** — Snapshot file 0o600 | ✅ MATCHED | `contexter-core/src/vector/snapshot.rs:195` | `set_permissions(0o600)` on snapshot output |
| **REQ-FIX-004** — Test for 0o700 behavior | ✅ MATCHED | `contexter-core/tests/storage/rocksdb_test.rs:130-144` | **NOW RESOLVED.** `test_engine_dir_has_0700_permissions` added — creates Engine, checks directory has 0o700 |

### Bug-DuckDB-Concurrency (3 reqs) — REQ-FIX-002 remains unmatched (accepted limitation)

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** — Batch `get_memories` | ✅ MATCHED | `contexter-core/src/storage/mod.rs:183`, `rocksdb.rs:795`, `engine/memory.rs:161`, `search.rs:215` | Batch fetch via `multi_get_cf()` in RocksDB, cache-aside in Engine, hybrid search uses batch |
| **REQ-FIX-002** — Split DuckDB connection | ❌ UNMATCHED | `contexter-core/src/analytics/duckdb.rs:107-111` | **Accepted limitation.** `duckdb::Connection` uses `RefCell` internally and is `!Sync`. Two file-backed connections create independent DB instances with no shared catalog. **Mitigation:** incremental sync (REQ-FIX-003) keeps write duration minimal. Documented in struct doc as "Known limitation". |
| **REQ-FIX-003** — Incremental sync | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:296-309,542,748-760` | `INSERT OR REPLACE` with `last_sync_timestamp` tracking; first sync truncates, subsequent syncs UPSERT only delta |

### Bug-Permissions-Test (1 req) — NEW for iter-3

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** — Add test verifying 0o700 permissions | ✅ MATCHED | `contexter-core/tests/storage/rocksdb_test.rs:128-144` | `test_engine_dir_has_0700_permissions` — `#[cfg(unix)]`, creates Engine at TempDir path, drops engine, asserts `metadata().permissions().mode() & 0o777 == 0o700` |

### Bug-DuckDB-Docs-Cleanup (2 reqs) — NEW for iter-3

| Req | Status | File | Evidence |
|-----|--------|------|----------|
| **REQ-FIX-001** — Fix misleading doc comment | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:91-95` | Struct doc now states "single `Mutex<Connection>`". No mention of "two separate connections" or "read-write connection split" in struct-level doc. |
| **REQ-FIX-002** — Document known limitation | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:107-111` | "Known limitation" section: single connection serialises reads and writes; "incremental sync mitigates write duration so the impact is negligible for typical analytics queries" |

---

## 02 · Implementation Mapping

### Iteration 3 Changes

| Bug Contract | File | Key Changes |
|---|---|---|
| **Bug-Permissions-Test** | `contexter-core/tests/storage/rocksdb_test.rs:128-144` | New test `test_engine_dir_has_0700_permissions` — validates Engine storage directory has 0o700 permissions on Unix |
| **Bug-DuckDB-Docs-Cleanup** | `contexter-core/src/analytics/duckdb.rs:91-111` | Struct doc updated: no "two separate connections" language; accurate single-`Mutex<Connection>` description; "Known limitation" section documents !Sync constraint and incremental sync mitigation |
| **Bug-DuckDB-Concurrency** | `contexter-core/src/analytics/duckdb.rs` | Connection split documented as infeasible in bug-fix-report; code unchanged (single `Mutex<Connection>`) |

### Full Implementation Mapping (unchanged from iter-2 baseline)

All previously mapped implementations remain in place. See iter-2 report Section 02 for full mapping of L3, L4, L5, Engine, and Storage code locations.

---

## 03 · Unmatched Requirements

### REQ-FIX-002 (Bug-DuckDB-Concurrency) — Split DuckDB Connection

**Severity:** MEDIUM
**Status:** ACCEPTED LIMITATION (not implemented)

**SPEC text:** "Replace the single `Mutex<Connection>` with a read-write split: one read connection (not locked for writes) and one write connection. Reads use the read connection (no contention); sync uses the write connection."

**Current implementation:** `DuckDbEngine` at `contexter-core/src/analytics/duckdb.rs:116` has a single `conn: Mutex<Connection>`. No read-write split exists.

**Root cause (infeasible):** `duckdb::Connection` (version 0.10) uses `RefCell` internally for its schema cache and prepared statement cache. `RefCell` is `!Sync`, meaning `Connection` cannot be shared across threads without external serialization. Opening two file-backed connections to the same path creates two independent database instances with no shared catalog — writes on one connection are invisible to reads on the other.

**Mitigation documented:** The struct doc at `duckdb.rs:107-111` explicitly documents this as a "Known limitation" and notes that incremental sync (REQ-FIX-003) keeps write duration to O(Δ) operations, minimizing lock contention.

**Resolution path:** If DuckDB's Rust bindings ever expose proper multi-connection mode with shared catalog concurrency, this can be revisited. The `AnalyticsEngine` trait hides the connection strategy behind a clean interface.

---

## 04 · Partially Matched Requirements

**None.** The previous partial match (Bug-Permissions-Hardening REQ-FIX-004) has been resolved by the new test `test_engine_dir_has_0700_permissions`.

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

All edge cases from previous iterations remain covered. The following new coverage was added in iteration 3:

| Edge Case Context | Status | Notes |
|-------------------|--------|-------|
| Engine storage directory 0o700 permission check | ✅ Added | `test_engine_dir_has_0700_permissions` at `rocksdb_test.rs:130-144` |
| DuckDB connection serialization documented | ✅ Added | "Known limitation" section at `duckdb.rs:107-111` |

---

## 07 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | **YES** — Bug-DuckDB-Concurrency REQ-FIX-002 is explicitly documented as an accepted infeasible limitation in code (duckdb.rs:107-111) and in the bug-fix-report. Bug-Permissions-Hardening REQ-FIX-004 is resolved via Bug-Permissions-Test. |
| Zero findings are being silently deferred to a future iteration | **YES** — The one remaining unmatched requirement (REQ-FIX-002 connection split) has been evaluated, determined infeasible, and explicitly documented. It is not deferred — it is accepted and mitigated. |

---

## 08 · Summary

> **SPEC Compliance Assessment**
> 94 of 95 requirements (98.9%) are fully matched with implementation code. One requirement (REQ-FIX-002 from Bug-DuckDB-Concurrency) remains unmatched due to a fundamental `duckdb` crate constraint: `Connection` is `!Sync` (internal `RefCell`), and two file-backed connections produce independent DB instances with no shared catalog. This limitation is documented in code as a "Known limitation" with incremental sync as mitigation. The previous partial match (Bug-Permissions-Hardening REQ-FIX-004) is now resolved with a dedicated test.

> **Findings**
> | # | Finding | Severity | Category |
> |---|---------|----------|----------|
> | 1 | `DuckDbEngine` has only a single `Mutex<Connection>` — read-write connection split not implemented. Infeasible due to `duckdb::Connection` being `!Sync` (internal `RefCell`). Documented as accepted limitation with incremental sync mitigation. | MEDIUM | UNMATCHED REQ-FIX-002 (Bug-DuckDB-Concurrency) — ACCEPTED LIMITATION |
> | 2 | Module-level doc comment at `duckdb.rs:1` still says "read-write connection split" while the struct-level doc correctly describes single `Mutex<Connection>`. Minor contradiction — module header should be updated for consistency. | LOW | OBSERVATION (not in any SPEC) |

---

## 09 · Final Verdict

| Criterion | Result |
|---|---|
| All REQ-XXX matched with implementation code | ❌ (1 unmatched — accepted limitation) |
| All CON-XXX constraints respected | ✅ |
| All EDGE_CASES covered by implementation or tests | ✅ (all edge cases verified) |
| Carryover declaration clean | ✅ |
| **Overall** | **⚠️ CONDITIONAL PASS** (1 unmatched requirement explicitly documented as infeasible and mitigated) |

**Verdict rationale:** This iteration resolves all actionable findings. The single remaining unmatched requirement (REQ-FIX-002, DuckDB connection split) is infeasible due to `duckdb::Connection` being `!Sync` — no code change can resolve it without a upstream `duckdb` crate change. The limitation is explicitly documented in code with mitigation (incremental sync). All other requirements (94/95) are fully matched.

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
| Bug-Permissions-Hardening | 4 | 4 | 0 | 0 |
| Bug-Analytics-Sync | 1 | 1 | 0 | 0 |
| Bug-API-Conformance | 4 | 4 | 0 | 0 |
| Bug-Perf-Queryparser | 1 | 1 | 0 | 0 |
| Bug-HNSW-Batch-Insert | 3 | 3 | 0 | 0 |
| Bug-Startup-Rebuild-Check | 1 | 1 | 0 | 0 |
| Bug-DuckDB-Concurrency | 3 | 2 | 0 | 1 |
| Bug-Engine-Drop | 3 | 3 | 0 | 0 |
| Bug-Permissions-Test *(NEW)* | 1 | 1 | 0 | 0 |
| Bug-DuckDB-Docs-Cleanup *(NEW)* | 2 | 2 | 0 | 0 |
| **Total** | **95** | **94** | **0** | **1** |

---

## Appendix B: Iteration 3 Changes from Iteration 2

| Finding from Iteration 2 | Status in Iteration 3 | Resolution |
|--------------------------|----------------------|------------|
| **REQ-FIX-004 (Bug-Permissions-Hardening)** — `test_read_only_path_error` removed, no replacement test for 0o700 | ✅ RESOLVED | `test_engine_dir_has_0700_permissions` added at `rocksdb_test.rs:130-144`. Validates Engine storage directory has 0o700 permissions on Unix. |
| **REQ-FIX-002 (Bug-DuckDB-Concurrency)** — Connection split not implemented | ❌ ACCEPTED LIMITATION | Evaluated as infeasible: `duckdb::Connection` is `!Sync` (internal `RefCell`). Two connections create independent DB instances. Documented in code as "Known limitation" (duckdb.rs:107-111). Mitigation via incremental sync. |

### New Items in Iteration 3

| Item | Bug Contract | Status | Evidence |
|------|-------------|--------|----------|
| 0o700 permission test | Bug-Permissions-Test REQ-FIX-001 | ✅ MATCHED | `contexter-core/tests/storage/rocksdb_test.rs:130-144` |
| Fix DuckDB doc comment | Bug-DuckDB-Docs-Cleanup REQ-FIX-001 | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:91-95` — single `Mutex<Connection>`, no "two separate connections" |
| Document known limitation | Bug-DuckDB-Docs-Cleanup REQ-FIX-002 | ✅ MATCHED | `contexter-core/src/analytics/duckdb.rs:107-111` — "Known limitation" section |

---

_Generated by SPEC Compliance Validator · 2026-07-25 · Validation Contract: contexter-phase2-search-analytics_
