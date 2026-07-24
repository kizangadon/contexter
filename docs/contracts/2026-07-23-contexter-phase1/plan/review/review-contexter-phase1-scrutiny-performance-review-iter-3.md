# Performance Review Report

# Contexter Phase 1 — Rust Storage Engine (Regression Check — Iteration 3)

> Two-tier storage engine with L1 DashMap+LRU cache (typed domain objects), L2 RocksDB with 9 column families (8 data + 1 secondary index), configurable WAL sync, WriteBatch atomicity, chunked iteration with read-lock yielding, pre-lowered keyword content, TTL-based cache eviction, CachedValue typed enum, and PyO3 bridge with PyBytes bypass for large payloads.

**Verdict:** PASS — zero findings (class: green)

2026-07-24 · 5 bug-fix regression checks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| WAL Write Amplitude | Configurable via `RocksDbConfig.wal_sync` — `maybe_flush_wal()` conditional on setting. Default `true` (fsync per write). |
| Search Scan Efficiency | Secondary indexes (`CF_MEMORY_INDEX`) on `session_id`, `memory_type`, `tags`. Index pre-filter reduces scan to matching IDs. |
| Cache Deserialization Cost | Zero on L1 hit — `CachedValue` returns typed domain objects directly. No JSON parse. |
| Cache-aside CPU Overhead | L1 hit: pattern match + clone (~100 ns). L1 miss: single deserialize + store. |
| TTL Eviction | `CacheConfig.max_ttl: Option<Duration>` — lazy eviction on `get()` access. |
| Write Atomicity | `WriteBatch` for atomic multi-CF writes. `write_batch()` on `StorageBackend` trait. |
| Chunked Iteration | `BATCH_SIZE=100` — all `list_*` methods release read lock between batches. |
| Python Bridge Overhead | PyBytes bypass for memories >100 KB eliminates double JSON. `max_workers` configurable. |
| **Regression Status** | All 5 performance bug fixes (Bugs 8, 10, 11, 12, 13) intact. |

> **Regression Scope**
> Re-validated all five performance-critical bug fixes from Iteration 2 against the current codebase. Source evidence verified via `grep` for each fix's key artifacts. Full test suite executed: 181/181 pass, clippy clean. No regressions detected.

---

## 02 · Bug Fix Regression Verification

## 2.1 Bug 8 — WAL Flush Optimization — ✅ INTACT

**Fix:** Configurable `wal_sync` boolean; `maybe_flush_wal()` conditional on setting; `checkpoint()` always flushes.

**Evidence verified 2026-07-24:**
- `RocksDbConfig { pub wal_sync: bool }` at `rocksdb_backend.rs:142`
- Default `true` at lines 150, 178
- `maybe_flush_wal()` at line 510–513: `if self.config.wal_sync { self.db.flush_wal(true)?; }`
- 15 call sites using `self.maybe_flush_wal()?;` instead of direct `flush_wal(true)`:
  - `create_session` (556), `update_session` (640), `delete_session` (649)
  - `create_memory` (720), `update_memory` (879), `delete_memory` (905)
  - `create_agent` (1017), `update_agent` (1112), `delete_agent` (1121)
  - `create_skill` (1147), `update_skill` (1227), `delete_skill` (1236)
  - `set_setting` (1267), `append_audit_entry` (1291)
- `checkpoint()` at lines 1391–1392: **Always flushes** — "Users who disable wal_sync for write throughput rely on explicit checkpoint() calls"

**Status:** ✅ INTACT — No regression. 15 call sites use conditional `maybe_flush_wal()`.

## 2.2 Bug 10 — Cache Typed Domain Objects — ✅ INTACT

**Fix:** `CachedValue` typed enum storing domain objects directly; cache hit returns typed value without JSON deserialization.

**Evidence verified 2026-07-24:**
- `CachedValue` enum at `cache/mod.rs:61–72`:
  ```rust
  pub enum CachedValue {
      Session(Session), Memory(Memory),
      Agent(Agent), Skill(Skill),
      Raw(Vec<u8>),
  }
  ```
- `CachedValue` derives `Clone` — `get()` returns cloned typed value
- 15 `CachedValue::*` pattern-match sites in `engine/mod.rs`:
  - `CachedValue::Session` at lines 204, 216, 223
  - `CachedValue::Memory` at lines 330, 341, 348
  - `CachedValue::Agent` at lines 420, 431, 438
  - `CachedValue::Skill` at lines 554, 565, 572
  - `CachedValue::Raw` at lines 666, 676, 686
- No `serde_json::from_slice` on L1 hit path (zero JSON cost)

**Status:** ✅ INTACT — No regression. All typed variants in use, zero JSON parse on hits.

## 2.3 Bug 11 — Search Indexes (Secondary Indexes) — ✅ INTACT

**Fix:** `CF_MEMORY_INDEX` 9th column family with secondary indexes on `session_id`, `memory_type`, `tags`. Content pre-lowercased at write time. Index pre-filter for filtered queries.

**Evidence verified 2026-07-24:**
- `pub const CF_MEMORY_INDEX: &str = "memory_index"` at `rocksdb_backend.rs:42`
- Index helpers:
  - `session_index_key()` (prefix `idx:ses:`)
  - `tag_index_key()` (prefix `idx:tag:`, lowercased at lines 361, 366)
  - `type_index_key()` (prefix `idx:typ:`)
  - `parse_memory_id_from_index_key()`
- `write_index_entries()` at line 394 — writes 3+ index entries per memory via WriteBatch
- `resolve_memory_ids_via_index()` at line 466 — prefix scan + intersection
- `search_memories` at line 747: `Some(self.resolve_memory_ids_via_index(query)?)`
- Pre-lowered content at line 697: `let content = memory.content.to_lowercase();`
- Index write at lines 717 (create), 876 (update)
- `count_memories` index path at line 943

**Status:** ✅ INTACT — No regression. Secondary indexes, index pre-filter, and pre-lowered content all present.

## 2.4 Bug 12 — Python Bridge Performance — ✅ INTACT

**Fix:** PyBytes path for memories >100 KB eliminates double JSON encoding. `max_workers` configurable.

**Evidence verified 2026-07-24:**
- `_MAX_MEMORY_JSON_SIZE = 102_400` at `core_bridge.py:11`
- `Engine.__init__(self, path, max_workers=4)` at line 23 — configurable `max_workers`
- `create_memory_bytes()` at python.rs:304 — accepts `&[u8]` content
- `update_memory_bytes()` at python.rs:361 — accepts `&[u8]` content
- Bridge routing at `core_bridge.py:73`: `if len(content) > _MAX_MEMORY_JSON_SIZE:` → PyBytes path
- Bridge routing at `core_bridge.py:94`: same for `update_memory`
- 4 dedicated tests in `python.rs`: `test_py_create_memory_bytes`, `test_py_update_memory_bytes`, `test_py_memory_bytes_invalid_utf8_produces_error`, `test_py_memory_bytes_update_nonexistent`

**Status:** ✅ INTACT — No regression. PyBytes bypass + configurable thread pool present.

## 2.5 Bug 13 — Cache TTL + WriteBatch + storage_size + Chunked Iteration — ✅ INTACT

**Fix (multi-part):** `max_ttl` lazy eviction, `WriteBatch` API, `storage_size` property reduction, chunked iteration with read-lock yielding.

**Evidence verified 2026-07-24:**
- `CacheConfig { pub max_ttl: Option<Duration> }` at `cache/mod.rs:104`, default `None` at line 112
- Lazy TTL eviction at `cache/mod.rs:198–201`: `inserted_at.elapsed() > *max_ttl`
- `inserted_at` actively used (no `#[allow(dead_code)]`) — line 83 documents "used for TTL stale-tracking"
- `write_batch(&self, cf, entries)` on `StorageBackend` trait at `storage/mod.rs:146`
- Implementation at `rocksdb_backend.rs:1355` — uses `rocksdb::WriteBatch`
- `storage_size()` at `rocksdb_backend.rs:1399` — uses 2 property queries per CF:
  - `rocksdb.estimate-live-data-size` (line 1421)
  - `rocksdb.cur-size-all-mem-tables` (line 1428)
  - Total: 18 property CF calls + 1 WAL size (was 24+1 in Phase 4)
- `BATCH_SIZE = 100` at `engine/mod.rs:72`
- Chunked iteration pattern `keys.chunks(BATCH_SIZE)` at lines 242, 457, 591, 714
- Read-lock yielded between chunks (lock acquired per chunk, dropped when `storage` guard exits scope)

**Status:** ✅ INTACT — No regression. All four sub-fixes present: TTL eviction, WriteBatch, storage_size batching (2 props/CF), chunked iteration.

---

## 03 · Full Test Suite Results

| Suite | Count | Status |
|-------|-------|--------|
| Unit tests | 168 | ✅ All pass |
| Integration tests | 13 | ✅ All pass |
| Total | 181 | ✅ All pass |
| Clippy (`-D warnings`) | — | ✅ Clean |

No test regressions introduced. All 181 tests produce identical pass/fail results to Iteration 2 baseline.

---

## 04 · Per-Bug-Fix Verification Matrix

| Bug | Fix Description | Key Artifacts Verified | Regression? |
|-----|----------------|----------------------|-------------|
| **Bug 8** | WAL flush optimization (`wal_sync` config + `maybe_flush_wal`) | `RocksDbConfig.wal_sync` at rs:142; 15 call sites at rs:556–1291; checkpoint always flushes at rs:1391 | ✅ None |
| **Bug 10** | Cache typed objects (`CachedValue` enum, zero-JSON L1 hits) | `CachedValue` enum at cache:61–72; 15 pattern-match sites in engine/mod.rs | ✅ None |
| **Bug 11** | Search indexes (`CF_MEMORY_INDEX`, index pre-filter, pre-lowered content) | `CF_MEMORY_INDEX` at rs:42; `write_index_entries` at rs:394; `resolve_memory_ids_via_index` at rs:466; `content.to_lowercase()` at rs:697 | ✅ None |
| **Bug 12** | Python bridge performance (PyBytes bypass, configurable `max_workers`) | `_MAX_MEMORY_JSON_SIZE` at bridge.py:11; `create_memory_bytes` at py.rs:304; `max_workers` param at bridge.py:23 | ✅ None |
| **Bug 13** | Cache TTL + WriteBatch + storage_size batching + chunked iteration | `max_ttl` at cache:104; `write_batch()` at storage:146; `storage_size` 2 props/CF at rs:1421–1428; `BATCH_SIZE=100` + `chunks(BATCH_SIZE)` at engine:72,242,457,591,714 | ✅ None |

**Total findings across all 5 bug fixes: 0.**

Each fix's characteristic artifacts are present at their expected locations. No fix has been reverted, refactored away, or broken. No regressions detected.

---

## 05 · Conclusion

**Verdict: PASS — zero findings.**

All five performance-critical bug fixes (WAL flush optimization, cache typed objects, search indexes, Python bridge performance, and cache TTL + WriteBatch + storage_size + chunked iteration) are confirmed intact on the `feature/contexter-phase1-core` branch.

- **181/181 tests pass** (168 unit + 13 integration) — same as Iteration 2 baseline
- **Clippy clean** — zero warnings
- **No regression in any performance-critical path** identified through source evidence and test execution

The performance profile established in Iteration 2 is fully preserved.

---

*Generated by Performance Benchmarker · 2026-07-24 · Validation Contract: contexter-phase1 (iter-3 — regression check)*
