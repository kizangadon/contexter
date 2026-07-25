# Security Review Report

# Contexter Phase 2 — Search & Analytics Engine

> Security review of L3 HNSW vector index, L4 Tantivy full-text search, L5 DuckDB analytics engine, hybrid search (RRF), and analytics efficiency/correlation. All tiers are optional and disabled by default.

**Verdict:** CONDITIONAL PASS (class: amber)

2026-07-25 · 10 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 2 |
| High | 1 |
| Medium | 3 |
| Low | 4 |

> **Security Scope**
> Review covers SQL injection via DuckDB parameterization, unsafe Rust patterns, path traversal in snapshot/index file I/O, data validation (vectors, queries), denial-of-service vectors, race conditions, error information leakage, and secure file handling across 12 implementation files in contexter-core.

---

## 02 · Vulnerability Findings

### SQLI-01 [CRITICAL] — DuckDB `query()` Ignores All Parameters

**Location:** `contexter-core/src/analytics/duckdb.rs:135` (fn query), `contexter-core/src/analytics/duckdb.rs:163-165` (stmt.query([]))
**Risk:** SQL query parameterization is completely absent — all `?` placeholders are never bound.

The `AnalyticsEngine::query()` trait method (defined in `analytics/mod.rs:61`) accepts a `params: &[Value]` parameter intended for parameterized query binding. The `DuckDbEngine` implementation receives this parameter at line 135 as `_params` (prefixed underscore, explicitly discarded). At lines 163-165, `stmt.query([])` is called with an **empty slice** regardless of the caller's intent to pass parameters.

This is a **critical security control failure**: the parameterization interface exists in the trait but the implementation is a no-op. Any future code path that supplies user-controlled values via the `params` argument will have them silently ignored, effectively constructing **dynamic SQL via string concatenation** at the call site.

**Impact:** Undefined — no call site currently constructs dynamic SQL with user input, but the interface is actively misleading. The `SESSION_COUNT_BY_RANGE` predefined query (which uses `?` placeholders for start/end timestamps) will **always fail** because the required parameters are never bound.

**Remediation:** Implement parameter binding in `DuckDbEngine::query()` using `stmt.query(params)` or `stmt.execute(params)` where `params` are properly converted to `duckdb::params!`.

---

### SQLI-02 [CRITICAL] — SESSION_COUNT_BY_RANGE Has Unbound Parameters

**Location:** `contexter-core/src/analytics/queries.rs:10-16`, `contexter-core/src/engine/analytics.rs:121-125`
**Risk:** Application crash + silently broken functionality. The predefined `SESSION_COUNT_BY_RANGE` query defines `?` placeholders for start/end timestamps (line 12: `WHERE created_at >= ? AND created_at <= ?`). The `get_session_count_by_range()` method in `engine/analytics.rs` passes `&[Value::Text(start.into()), Value::Text(end.into())]` as parameters (line 123-124).

Because `DuckDbEngine::query()` ignores all parameters (SQLI-01), these placeholders are never bound. DuckDB will reject the query with a parameter count mismatch error. **This method is functionally broken** and any caller will receive an `EngineError::Internal` wrapping a DuckDB error.

**Impact:** Any code path that calls `get_session_count_by_range()` (or any future user of parameterized queries) will fail at runtime. The `run_analytics()` method at line 68 does not use `SESSION_COUNT_BY_RANGE` directly, but `get_session_count_by_range()` is a public method (line 111).

**Remediation:** Fix SQLI-01 first. Then verify the parameter values are properly validated (start/end timestamps should be in ISO 8601 format, not arbitrary strings).

---

### STR-001 [HIGH] — Mutex Poisoning DoS via Widespread `.unwrap()` on Locks

**Location:** Multiple files — `duckdb.rs:85,95,157,303`, `hnsw.rs:84,107-110,120-126,150-163,204-233`, `tantivy.rs:99,171,178`, `memory.rs:29-31`
**Risk:** Single panic in any critical section permanently disables the component.

The codebase makes extensive use of `.lock().unwrap()` (and `.write().unwrap()` / `.read().unwrap()`) on `Mutex` and `RwLock` instances. If a thread panics while holding a lock (due to a bug, assertion failure, or unexpected condition), the mutex becomes **poisoned**. Every subsequent `.unwrap()` call on the poisoned lock will itself panic, creating a cascade that disables the entire component — vector index, FTS, or analytics engine — until the process restarts.

Examples of at-risk locations:
- `duckdb.rs:85,95,157,303`: All access to DuckDB `Connection` and `synced_tables` via `.lock().unwrap()`
- `hnsw.rs:107,114,120,150,160,181-182,204,211,230-233`: All access to HNSW `embeddings`, `hnsw`, `removed`, `mutation_count`, `snapshot_path`
- `memory.rs:29,31`: `storage.write().unwrap()` in the memory write path

**Impact:** An attacker who can trigger a panic while a lock is held (e.g., by sending a malformed vector that causes allocation failure, or triggering an assertion in DuckDB) can permanently disable indexing or search operations. Recovery requires process restart.

**Remediation:** Replace `.unwrap()` with `.expect("descriptive message")` on `std::sync::PoisonError` paths. For long-lived critical services, implement a "poisoned state" pattern that degrades gracefully (logs error and skips the operation) rather than panicking.

---

### PATH-001 [MEDIUM] — No Symlink Validation in File I/O Operations

**Location:** `contexter-core/src/vector/snapshot.rs:153,182`, `contexter-core/src/fts/tantivy.rs:39-43`, `contexter-core/src/engine/mod.rs:253`
**Risk:** Symlink following attacks during snapshot save/load and index creation.

The code opens files and creates directories at paths derived from `EngineConfig` without any verification that the path is not a symlink to an unintended location. Specifically:

1. `snapshot.rs:153` — `File::create(path)` on the snapshot path — if the path is a symlink, the process writes sensitive embedding data to whichever file the symlink points to.
2. `snapshot.rs:182` — `File::open(path)` on the snapshot path — if the path is a symlink, the process reads from an attacker-controlled file, potentially causing deserialization of attacker-controlled data.
3. `tantivy.rs:39` — `create_dir_all(path.parent())` — could follow symlinks during directory creation.
4. `engine/mod.rs:253` — `path.exists()` check — could be a symlink.

**Impact:** An attacker with local filesystem access (e.g., in a multi-tenant environment or container escape) could:
- Redirect snapshot writes to overwrite sensitive files (e.g., `~/.ssh/authorized_keys`)
- Create symlinks to sensitive files that get read and deserialized

**Remediation:** Before opening file handles, resolve the real path with `std::fs::canonicalize()` and verify it's within the expected base directory. Use platform-specific open flags (e.g., `O_NOFOLLOW` on Linux) where available. For creation, use `OpenOptions::new().create_new(true)` instead of `File::create()` to avoid overwriting existing files.

---

### RACE-001 [MEDIUM] — TOCTOU Race in Auto-Sync TTL Check

**Location:** `contexter-core/src/analytics/duckdb.rs:84-91` (needs_sync), `contexter-core/src/analytics/duckdb.rs:137-155` (query → auto-sync calls)
**Risk:** Double-sync on concurrent queries from different threads.

The `needs_sync()` method (line 84) locks `synced_tables` mutex, checks TTL, unlocks. Then `sync()` (line 212) locks `synced_tables` again and updates it. There is a **time-of-check-to-time-of-use** gap between these two lock acquisitions. If two threads both determine `needs_sync() == true`, they will both call `sync()`, causing double-sync. While the truncate-reinsert pattern is idempotent, this wastes I/O and CPU.

More critically, between the `needs_sync()` check and the actual `sync()` call in the `query()` method (lines 137-155), the sync timestamps are not protected by a single atomic operation. This is a wasted work concern, not a data corruption risk (since syncs are read-only).

**Impact:** Wasted CPU/I/O on concurrent queries. Under high concurrency, the number of redundant sync operations could multiply, creating a self-DoS scenario.

**Remediation:** Use a single mutex scope for the check-and-insert-if-needed pattern. Consider `tokio::sync::RwLock` or use a compare-and-swap approach with `AtomicI64` tracking last-sync timestamp.

---

### DOS-001 [MEDIUM] — No Upper Bound on Hybrid Search `limit`

**Location:** `contexter-core/src/engine/search.rs:129` (fetch_k = query.limit * 2)
**Risk:** Unbounded memory allocation on hybrid search.

The `hybrid_search()` method at line 129 computes `fetch_k = query.limit * 2`. If the caller passes `limit = usize::MAX`, the multiplication overflows (in debug mode, Rust panics; in release mode, it wraps to a small number indirectly). Even `limit = 10_000_000` would request 20M results from both L3 and L4, causing excessive memory allocation for the `merged` HashMap and the result vectors.

**Impact:** An attacker who can control the `HybridSearchQuery` (directly or indirectly) can trigger OOM conditions by specifying an extremely large `limit`.

**Remediation:** Add a configurable hard cap on `limit` (e.g., `MAX_HYBRID_RESULTS = 1000`). Clamp `fetch_k` to `min(query.limit * 2, MAX_HYBRID_RESULTS * 2)`.

---

### ERR-001 [LOW] — DuckDB Error Messages May Leak Internal Schema

**Location:** `contexter-core/src/analytics/duckdb.rs:56-58,71-72,96-97,158-165,231-301`
**Risk:** DuckDB error strings embedded in returned errors may contain table schema details.

Throughout `duckdb.rs`, errors from DuckDB are captured with `.to_string()` and wrapped in `AnalyticsError::QueryError` (e.g., line 57: `AnalyticsError::QueryError(e.to_string())`). These errors can contain:
- Table names and column definitions
- SQL syntax context
- Internal DuckDB error details

While these errors are ultimately wrapped in `EngineError::Internal` (via the `From` impl at `error.rs:17-20`), any code that logs or exposes the `Display` output of these errors could leak internal schema information to users.

**Impact:** Low in the current architecture (errors returned to library callers, not directly to end users). But if the `Engine` is used in a web service, these error messages could reach API clients.

**Remediation:** Strip or sanitize DuckDB error messages before wrapping. Map known error patterns (e.g., "table ... does not exist") to typed variants like `AnalyticsError::TableNotFound`.

---

### PATH-002 [LOW] — No File Permission Hardening on Snapshot Files

**Location:** `contexter-core/src/vector/snapshot.rs:153` (File::create)
**Risk:** Snapshot files created with default umask permissions may be readable by other users.

Snapshots contain raw embedding vectors and memory IDs — potentially sensitive internal data. `File::create()` at line 153 creates files with the process's default umask, which on many systems defaults to `022` (world-readable). On a multi-tenant system or shared CI runner, other processes or users could read snapshot data.

**Remediation:** On Unix, create the file with `OpenOptions::new().write(true).create(true).mode(0o600)` to restrict access to the owner. Or use `std::os::unix::fs::PermissionsExt` to set permissions after creation.

---

### DOS-002 [LOW] — No Limit on Embedding Count (Memory Exhaustion)

**Location:** `contexter-core/src/vector/hnsw.rs:48-49,148-157` (embeddings grows unbounded)
**Risk:** Unbounded vector storage can exhaust process memory.

The `HnswVectorIndex` stores all embeddings in a `Vec<Embedding>` (line 48). There is no upper bound on the number of embeddings. Each insert pushes a new entry into the `Vec` (or updates existing). With a dimension of 384, each embedding consumes ~1.5KB of RAM. An attacker who can insert vectors (via the `insert()` method) could exhaust the process's memory allocation by inserting millions of vectors.

Additionally, `rebuild()` (line 106) clones the entire embedding list (`let points = embeddings.clone()`) on every insert, temporarily doubling memory usage during rebuild.

**Impact:** On-premise memory exhaustion. A process that accepts external memory storage could be driven OOM.

**Remediation:** Add a configurable `max_embeddings` limit to `HnswVectorIndex`. Reject inserts beyond the limit with a `VectorError::Internal("index full")` error. Consider periodic rebuild instead of per-insert rebuild for large indexes.

---

### VAL-001 [LOW] — No Cache Size Limit on DashMapCache for Vector/FTS Data

**Location:** `contexter-core/src/engine/mod.rs:242-246` (cache creation)
**Risk:** Unbounded cache growth for large memory stores.

The `DashMapCache` is created with default config when not explicitly configured (line 244-246). While the cache has per-type LRU buckets, the total number of cached values across all types could grow large if the working set of memories is big. This is a general resource exhaustion concern.

**Impact:** Cache grows with working set size — expected behavior but could be a concern in memory-constrained environments.

**Remediation:** Document the default cache configuration and recommend explicit `CacheConfig` tuning. Consider a global memory budget in addition to per-type entry limits.


---

## 03 · Security-Critical Code Highlights

### Parameterized Query Interface Defect (SQLI-01)

```rust
// duckdb.rs:135 — params received but ignored
fn query(&self, sql: &str, _params: &[Value]) -> AnalyticsResult<Vec<Vec<Value>>> {
    // ...
    let mut stmt = conn.prepare(sql)...;
    let mut rows = stmt.query([])  // ← empty params, always!
        .map_err(|e| AnalyticsError::QueryError(e.to_string()))?;
```

The `_params` underscore prefix signals the parameter is intentionally unused. Any `?` in SQL is never bound.

### All Unbounded File I/O (PATH-001)

```rust
// snapshot.rs:153 — No symlink validation
let file = File::create(path)?;      // ← follows symlinks
let mut writer = std::io::BufWriter::new(file);
```

### HNSW Rebuild on Every Insert (DOS-002)

```rust
// hnsw.rs:106-116 — Clones entire Vec<Embedding> on every insert
fn rebuild(&self) {
    let embeddings = self.embeddings.read().unwrap();
    let points = embeddings.clone();  // ← O(n) memory, O(n log n) CPU per insert
    let (new_hnsw, _pids) = Builder::default().build_hnsw(points);
    *self.hnsw.write().unwrap() = new_hnsw;
}
```


---

## 04 · Remediation Recommendations

> **Must Fix**
> 1. [SQLI-01] Implement parameter binding in `DuckDbEngine::query()` — stop ignoring `params`.
2. [SQLI-02] Fix `SESSION_COUNT_BY_RANGE` parameter binding so `get_session_count_by_range()` works.
3. [STR-001] Replace all `.lock().unwrap()` with poison-safe error handling to prevent cascading panics.


> **Should Fix**
> 1. [PATH-001] Add `O_NOFOLLOW` / `canonicalize()` checks for all file I/O paths.
2. [RACE-001] Atomic check-and-sync in `DuckDbEngine::query()` to prevent redundant syncs.
3. [DOS-001] Add hard upper bound on hybrid search `limit` parameter.


> **Consider**
> 1. [PATH-002] Set `0o600` permissions on snapshot files.
2. [DOS-002] Add max_embeddings limit to `HnswVectorIndex`.
3. [ERR-001] Sanitize DuckDB error messages to avoid leaking schema internals.
4. [VALID-001] Document `CacheConfig` recommendations for memory-constrained deployments.


---

_Generated by Security Architect · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase2-search-analytics_
