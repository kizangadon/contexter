# Security Review Report

# Contexter Phase 2 — Search & Analytics Engine (Iteration 1)

> Security scrutiny for parent feature + 10 bug contracts validating Mutex poison recovery, file permissions, input validation, snapshot TOCTOU, error handling, and unsafe block analysis.

**Verdict:** CONDITIONAL PASS (class: informational)

2026-07-25 · 6 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 2 |
| Low | 2 |
| Informational | 2 |

> **Security Scope**
> Reviewed all 10 bug fix contracts against the Rust implementation: `engine/mod.rs`, `engine/search.rs`, `engine/analytics.rs`, `engine/memory.rs`, `engine/session.rs`, `engine/agent.rs`, `engine/skill.rs`, `engine/maintenance.rs`, `engine/settings.rs`, `vector/hnsw.rs`, `vector/snapshot.rs`, `vector/error.rs`, `error/mod.rs`, `analytics/duckdb.rs`, `analytics/sync.rs`, `analytics/error.rs`, `fts/tantivy.rs`, `storage/rocksdb.rs`. Checked: unsafe blocks, TOCTOU races, file permissions, input validation, panics/unwraps, path traversal, info leaks in errors, poison recovery, buffer over-reads, and resource exhaustion.

---

## 02 · Vulnerability Findings

### Finding S-01: `read_string` in snapshot.rs lacks max-length guard (MEDIUM)

**File:** `contexter-core/src/vector/snapshot.rs:113-121`

```rust
pub(crate) fn read_string<R: Read>(r: &mut R) -> std::io::Result<String> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;
    let mut bytes = vec![0u8; len];
    if len > 0 {
        r.read_exact(&mut bytes)?;
    }
    Ok(String::from_utf8_lossy(&bytes).to_string())
}
```

The length prefix (`u32`) is cast directly to `usize` without an upper bound check. A crafted or corrupted snapshot file with a length value near `u32::MAX` would attempt to allocate ~4 GiB, causing OOM or panic. This is a denial-of-service vector against the vector index snapshot loading.

**Impact:** An attacker with write access to the snapshot file (or a path traversal that lets them supply a crafted snapshot) can crash the engine with OOM on load.

**Risk:** Low in practice — requires local file access to the snapshot file — but violates the defence-in-depth principle.

**Recommendation:** Add a sanity check, e.g. `if len > MAX_STRING_LENGTH { return Err(...) }`. For context, vector IDs are UUIDs (36 bytes), so a reasonable max would be 256 or 1024 bytes.

---

### Finding S-02: Tantivy index directory permissions not set (MEDIUM)

**Files:**
- `contexter-core/src/storage/rocksdb.rs:181-187` — RocksDB dir sets `0o700` ✅
- `contexter-core/src/fts/tantivy.rs:38-48` — Tantivy open/create does NOT set permissions ❌

```rust
// tantivy.rs:42-48 — no permission hardening
if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)
        .map_err(|e| FtsError::Io(format!("create dir: {e}")))?;
}
let index = Index::create_in_dir(path, schema.clone())
    .map_err(|e| FtsError::Io(format!("create index: {e}")))?;
```

The RocksDB backend explicitly sets `0o700` permissions on the data directory (Bug-File-Security, AC-01). The Tantivy index directory lacks this hardening. The Tantivy index contains the raw full-text search data with all indexed content (memory titles, content, tags), which may contain sensitive or PII data.

**Impact:** If the Tantivy path is on a multi-tenant system, other users could read the indexed content.

**Recommendation:** Apply `std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))` to the Tantivy index directory after creation, consistent with the RocksDB pattern.

---

### Finding S-03: TOCTOU race window in `load_snapshot` between metadata check and file open (LOW)

**File:** `contexter-core/src/vector/hnsw.rs:397-427`

```rust
fn load_snapshot(&self, path: &Path) -> VectorIndexResult<usize> {
    // TOCTOU mitigation: metadata check before open
    let metadata = std::fs::metadata(path)?;           // line 400 — check
    if metadata.is_dir() { ... }
    if metadata.len() == 0 { ... }

    let (count, data, loaded_removed) =
        snapshot::load_snapshot_data(path, self.dimension)?;  // line 413 — use
```

The TOCTOU mitigation is present (Bug-File-Security, REQ-FIX-002) and detects empty files and directories before opening. However, there is still a race window between the `metadata()` syscall (line 400) and the `File::open()` inside `load_snapshot_data` (line 413, `snapshot.rs:182`). A local attacker with write access to the snapshot path could replace a valid snapshot file with a malicious one after the metadata check but before the file is opened.

**Impact:** The mitigation reduces the attack surface (catches zero-length files, directories) but does not fully close the TOCTOU window. On a single-user system this is acceptable; on multi-tenant or containerised deployments with shared volumes, the window is exploitable.

**Recommendation:** Open the file once, then stat the opened `File` handle with `file.metadata()?.len()` instead of calling `std::fs::metadata(path)` separately. This eliminates the TOCTOU race entirely:

```rust
let file = File::open(path)?;
let meta = file.metadata()?;
if meta.is_dir() { ... }
if meta.len() == 0 { ... }
// then pass file (or BufReader) to load_snapshot_data
```

---

### Finding S-04: `read_string` uses `from_utf8_lossy` — silent data corruption (LOW)

**File:** `contexter-core/src/vector/snapshot.rs:120`

```rust
Ok(String::from_utf8_lossy(&bytes).to_string())
```

When loading snapshot strings (IDs, etc.), invalid UTF-8 bytes are silently replaced with U+FFFD characters. This could cause:
1. Corrupted vector IDs that don't match any memory UUID
2. Silent data loss — entries appear loaded but have garbage IDs
3. Subtle bugs where removed-set entries fail to match their originals

The snapshot format stores Rust `String` objects (guaranteed valid UTF-8), so a corrupted snapshot with non-UTF-8 bytes indicates either file corruption or a deliberately crafted malicious file.

**Impact:** Low — valid snapshots always produce valid UTF-8. Malicious/corrupt snapshots would degrade gracefully (unmatched IDs just don't appear in search results). Still, `from_utf8` with error propagation is the more correct choice.

**Recommendation:** Replace with `String::from_utf8(bytes).map_err(|e| VectorError::Io(e.to_string()))?` to return an error on invalid UTF-8 rather than silently replacing data.

---

### Finding S-05: `snapshot::save_snapshot_data` has no permissions hardening on output file (INFORMATIONAL)

**File:** `contexter-core/src/vector/snapshot.rs:153`

```rust
let file = File::create(path)?;
let mut writer = std::io::BufWriter::new(file);
```

The snapshot file is created with default (umask-dependent) permissions. On systems where the snapshot path is shared, this could expose vector embedding data to other users. Compare with RocksDB's directory-level `0o700` hardening.

**Risk:** Informational — the snapshot path is typically inside the user's home directory or app data directory.

**Recommendation:** After creating the snapshot file, set permissions to `0o600` (owner read/write only). Note that the `save()` method in `hnsw.rs` uses atomic write (temp + rename), so permissions should be set on the temp file before rename.

---

### Finding S-06: `debug` format of `HnswVectorIndex` leaks internal state via lock access (INFORMATIONAL)

**File:** `contexter-core/src/vector/hnsw.rs:448-449`

```rust
.field("active_count", &self.len())
.field("total_count", &self.embeddings.read().unwrap_or_else(|e| e.into_inner()).len())
.field("removed_count", &self.removed.read().unwrap_or_else(|e| e.into_inner()).len())
```

The `Debug` implementation acquires live locks. While `Debug` is typically only used for logging/diagnostics, a panic in a debug-format call while holding the lock (via `unwrap_or_else` inside the `Debug::fmt` call's lambda evaluation) could theoretically cause a double-panic or lock poisoning. The poison recovery handles this, but it's a subtle issue.

**Risk:** Informational — standard practice in Rust, and the level of indirection makes double-panics unlikely. Not actionable in the current iteration.

---

## 03 · Security-Critical Code Highlights

### ✅ Bug-File-Security: RocksDB directory permissions (`0o700`)

**File:** `contexter-core/src/storage/rocksdb.rs:181-187`

Locked. The `create_dir_all` + `set_permissions` with `Permissions::from_mode(0o700)` pattern correctly restricts database access to the owner. This matches Bug-File-Security REQ-FIX-001.

### ✅ Bug-File-Security: Snapshot metadata check (TOCTOU mitigation)

**File:** `contexter-core/src/vector/hnsw.rs:400-411`

Locked. The `is_dir()` and `len() == 0` checks reject directories and empty files before attempting to load. The `EmptySnapshot` error variant provides clear diagnostics. This matches Bug-File-Security REQ-FIX-002.

### ✅ Bug-Validation: Dimension guard

**File:** `contexter-core/src/engine/mod.rs:272-276`

Locked. `if config.enable_vector_index && config.vector_dimension == 0` returns `Err(EngineError::InvalidConfig(...))`. Prevents creating a zero-capacity HNSW index. Matches Bug-Validation REQ-FIX-001.

### ✅ Bug-Search-Validation: Clamp and cap

**File:** `contexter-core/src/engine/search.rs:129-141`

Locked. `vector_weight.clamp(0.0, 1.0)`, `query.limit.min(1000)`, `limit == 0` early return, and whitespace-only `sort_field` check. Matches Bug-Search-Validation REQ-FIX-001, REQ-FIX-002, REQ-FIX-003.

### ✅ Bug-Poison: Poison recovery on all Mutex/RwLock

73 occurrences across the codebase using `.unwrap_or_else(|e| e.into_inner())`. Matches Bug-Poison REQ-FIX-001 and REQ-FIX-002.

### ✅ Bug-Errors: Bare `.unwrap()` replaced

**Files:** `engine/session.rs`, `engine/agent.rs`, `engine/skill.rs`, `engine/maintenance.rs`, `engine/settings.rs`

All bare `.unwrap()` calls replaced with poison recovery or error propagation. `UnsupportedOperation` variant added. `TempDirGuard` drop-based cleanup added. Matches Bug-Errors REQ-FIX-001, REQ-FIX-002, REQ-FIX-004.

### ✅ Bug-Snapshot: Periodic save + shutdown wiring

**Files:** `engine/mod.rs:334-366` (periodic snapshot thread), `engine/mod.rs:389-411` (shutdown)

Locked. Periodic snapshot with `Arc<AtomicBool>` cancellation token, final save on `shutdown()`. Matches Bug-Snapshot REQ-FIX-002, REQ-FIX-003.

### ✅ Bug-Errors: `UnsupportedOperation(String)` variant

**File:** `contexter-core/src/error/mod.rs:53-54`

Locked. Added with `sanitized()` handler. Matches Bug-Errors REQ-FIX-002.

### ✅ Bug-Errors: `TempDirGuard` cleanup

**File:** `contexter-core/src/analytics/duckdb.rs:44-68`

Locked. Drop-based `remove_dir_all` on engine teardown. Matches Bug-Errors REQ-FIX-004.

### ✅ Bug-HNSW-Config: HNSW params exposed in `EngineConfig`

**File:** `contexter-core/src/engine/mod.rs:168-176`

Locked. `hnsw_m`, `hnsw_ef_construction`, `hnsw_ef_search` fields with defaults. Matches Bug-HNSW-Config REQ-FIX-001, REQ-FIX-002.

### ✅ Bug-Efficiency: `EFFICIENCY_CF` constant and cache wiring

**File:** `contexter-core/src/analytics/duckdb.rs:27` (constant), `analytics/duckdb.rs:92` (`efficiency_cache` field)

Locked. Per-session caching with TTL-based eviction. Matches Bug-Efficiency REQ-FIX-001, REQ-FIX-002, REQ-FIX-003, REQ-FIX-004.

### ✅ Bug-DB-Analytics: Parameter binding and storage backend wiring

**File:** `contexter-core/src/analytics/duckdb.rs:484-493`

Locked. Real parameter binding via `duckdb::ToSql` conversion. Storage backend wired in `engine/mod.rs:326`. Matches Bug-DB-Analytics REQ-FIX-001, REQ-FIX-002.

### ✅ Bug-FTS: Title/tags fields, TextContent, path wiring

**File:** `contexter-core/src/fts/tantivy.rs:172-178` (field boosting with title=2.0, tags=1.5). `engine/mod.rs:309` (tantivy_path wiring).

Locked. Matches Bug-FTS REQ-FIX-002, REQ-FIX-004.

---

## 04 · Remediation Recommendations

> **Must Fix**
> (none)

> **Should Fix**
> - **[S-01]** Add max-length guard in `read_string()` at `snapshot.rs:115`. Cap `len` to 1024 bytes and return `VectorError::Io("...")` if exceeded.
> - **[S-02]** Add `set_permissions(0o700)` on Tantivy index directory in `tantivy.rs:open()`.

> **Consider**
> - **[S-03]** Close the TOCTOU window by opening the file first, then calling `file.metadata()`, then passing the `File`/`BufReader` to `load_snapshot_data` instead of the path.
> - **[S-04]** Use `String::from_utf8(bytes).map_err(...)` instead of `from_utf8_lossy` to fail on invalid snapshot data.
> - **[S-05]** Set `0o600` permissions on snapshot file after creation in `save_snapshot_data()` and the temp-file write in `save()`.
> - **[S-06]** No action needed — informational.

---

## 05 · Bug Contract Verification Summary

| Bug Contract | Security ACs | Status | Notes |
|---|---|---|---|
| Bug-File-Security | 0o700 perms, TOCTOU check | ✅ PASS | RocksDB `0o700` set; metadata check present; `EmptySnapshot` variant exists |
| Bug-Validation | InvalidConfig for bad dim | ✅ PASS | `vector_dimension == 0` guard at line 272; `InvalidConfig` variant exists |
| Bug-Search-Validation | weight clamp, limit cap | ✅ PASS | `clamp(0.0, 1.0)`, `min(1000)`, `limit==0` return empty |
| Bug-Poison | `.unwrap_or_else(e->e.into_inner())` | ✅ PASS | 73 occurrences across all engine files |
| Bug-Errors | no bare unwrap, unsupported variant, temp cleanup | ✅ PASS | All 5 remaining `.unwrap()` calls fixed; `UnsupportedOperation` added; `TempDirGuard` added |
| Bug-Snapshot | save, periodic, shutdown | ✅ PASS | Already implemented: `save_snapshot`, periodic thread, `shutdown()` |
| Bug-HNSW-Config | M/ef_construction/ef_search exposed | ✅ PASS | Fields in `EngineConfig` with defaults (16, 200, 50) |
| Bug-Efficiency | EFFICIENCY_CF, per-session cache | ✅ PASS | `EFFICIENCY_CF` constant; `efficiency_cache: RwLock<HashMap>` with TTL |
| Bug-DB-Analytics | param binding, storage backend | ✅ PASS | Real `duckdb::params![]` binding; `set_storage_backend(backend)` wired |
| Bug-FTS | title/tags fields, path wiring | ✅ PASS | Fields in schema with boosts; tantivy_path from config |

---

_Generated by Security Architect · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase2-search-analytics · Iteration 1_
