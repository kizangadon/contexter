# Security Review Report

# Contexter Phase 2 — Auto Bug Loop Iteration 2 (Security)

> Re-validation of 5 bug contracts: Permissions-Hardening, Snapshot-Robustness, Engine-Drop, Analytics-Sync, and File-Security (previous). All fixes verified against source code at `contexter-core/src/`.

**Verdict:** **PASS** (class: pass)

2026-07-25 · 0 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

> **Security Scope**
> File permissions (TempDir, Tantivy, snapshots), snapshot deserialisation robustness (OOM guard, strict UTF-8, TOCTOU), Engine Drop thread safety (zombie thread prevention, idempotent shutdown), analytics sync input validation (CAST safety), and cross-platform `#[cfg(unix)]` discipline.

---

## 02 · Vulnerability Findings

**Zero findings.** All 5 bug contracts verified correct at the source-code level. Detailed per-contract breakdown follows.

---

### 2.1 Bug-Permissions-Hardening ✅

**SPEC: `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-permissions-hardening/SPEC.md`**

| REQ | Status | File & Lines | Detail |
|-----|--------|-------------|--------|
| REQ-FIX-001 | ✅ PASS | `duckdb.rs:62-68` | `TempDirGuard::new()`: `#[cfg(unix)]` → `set_permissions(&dir, Permissions::from_mode(0o700))`. Error is logged via `eprintln`, does not abort construction. |
| REQ-FIX-002 | ✅ PASS | `tantivy.rs:55-61` | `TantivyIndex::open()`: `#[cfg(unix)]` → `set_permissions(path, Permissions::from_mode(0o700))`. Applied after `Index::create_in_dir()`. |
| REQ-FIX-003 | ✅ PASS | `snapshot.rs:192-198` | `save_snapshot_data()`: `#[cfg(unix)]` → `set_permissions(path, Permissions::from_mode(0o600))`. Applied after `writer.flush()`. |
| REQ-FIX-004 | ✅ PASS | `rocksdb_test.rs:113-121` | `test_read_only_path_error` replaced with `test_writable_path_succeeds`. Test now expects `Ok(..)` since 0o700 auto-fix makes dir writable. |

**Security rationale for 0o600 on snapshot file:** Snapshot files contain raw embedding vectors and string IDs — a full dump of all indexed data. 0o600 (owner read/write only) is appropriate. 0o700 on directories ensures temp and index data is not readable by other OS users.

---

### 2.2 Bug-Snapshot-Robustness ✅

**SPEC: `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-snapshot-robustness/SPEC.md`**

| REQ | Status | File & Lines | Detail |
|-----|--------|-------------|--------|
| REQ-FIX-001 | ✅ PASS | `snapshot.rs:117-124` | **Max-length guard (OOM prevention):** `MAX_STRING_LEN = 1024`. `read_string()` reads `u32` length prefix, checks `len > MAX_STRING_LEN`, returns `std::io::ErrorKind::InvalidData` if exceeded. Prevents crafted snapshots from triggering large `vec![0u8; len]` allocation. |
| REQ-FIX-002 | ✅ PASS | `snapshot.rs:132-137` | **Strict UTF-8:** `String::from_utf8(bytes).map_err(|e| std::io::Error::new(InvalidData, ...))` replaces `String::from_utf8_lossy(buf)`. Malformed UTF-8 in snapshot now produces an error instead of silently substituting replacement characters, which could corrupt string IDs or produce unexpected search behaviour. |
| REQ-FIX-003 | ✅ PASS | `hnsw.rs:454-472` | **TOCTOU fix:** `load_snapshot()` now opens the file first (`File::open(path)`), then calls `file.metadata()` on the opened handle. Previously the code checked `path.exists()` before opening — a TOCTOU window. The opened `File` is passed to `load_snapshot_data()` (which now takes `File`, not `&Path`). |

**Security analysis of TOCTOU:** The classic race is: attacker replaces a legitimate snapshot with a malicious one between the `path.exists()` check and the `File::open()`. By opening the file first then checking metadata on the already-opened handle, the handle is pinned to the inode at open time. The `metadata()` result reflects the actual opened file, not whatever replaced it after open.

---

### 2.3 Bug-Engine-Drop ✅

**SPEC: `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-engine-drop/SPEC.md`**

| REQ | Status | File & Lines | Detail |
|-----|--------|-------------|--------|
| REQ-FIX-001 | ✅ PASS | `engine/mod.rs:451-457` | `impl Drop for Engine { fn drop(&mut self) { let _ = self.shutdown(); } }`. Errors are discarded explicitly (`let _ =`) to prevent `Drop` panics. |
| REQ-FIX-002 | ✅ PASS | `engine/mod.rs:419-444` | **Idempotent shutdown:** Uses `self.snapshot_handle.take()` (Option + `take()` pattern) so the join handle is consumed on first call. Subsequent calls are no-ops. Cancel flag is set early so the snapshot thread sees the stop signal regardless. |
| REQ-FIX-003 | ✅ PASS | `engine/mod.rs:428` | **Thread join:** `handle.join().map_err(...)?` blocks until the snapshot thread terminates before returning from `shutdown()`. |

**Security rationale:** Without `Drop`, an `Engine` value that goes out of scope without an explicit `shutdown()` call would leave a background snapshot thread running — a zombie thread that could try to write to a partially-dropped RocksDB handle, producing UB or data corruption. The `Drop` impl is the last line of defence.

---

### 2.4 Bug-Analytics-Sync ✅

**SPEC: `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-analytics-sync/SPEC.md`**

| REQ | Status | File & Lines | Detail |
|-----|--------|-------------|--------|
| REQ-FIX-001 | ✅ PASS | `duckdb.rs:340-347` (sessions) | `created_at.is_empty()` check before `CAST(? AS TIMESTAMP)`. Empty-timestamp records are skipped with a structured `eprintln!` warning. Performs the same check for `last_active` (line 348-355). |
| | | `duckdb.rs:418-425` (memories) | Same `created_at.is_empty()` guard. |
| | | `duckdb.rs:495-502` (telemetry) | Same guard on `timestamp.is_empty()`. |

**Security rationale:** An empty string passed to `CAST('' AS TIMESTAMP)` produces a DuckDB error that would abort the entire sync operation. By validating and skipping these records, a single corrupt or incomplete record cannot block analytics for all other records. The `eprintln!` warning ensures observability without crashing.

---

### 2.5 Bug-File-Security (Previous) ✅

**SPEC: `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-file-security/SPEC.md`**

Both fixes remain intact and are re-verified by the same source code as above:
- **REQ-FIX-001 (0o700 on temp dirs):** ✅ (same as Permissions-Hardening REQ-FIX-001)
- **REQ-FIX-002 (TOCTOU mitigation):** ✅ (same as Snapshot-Robustness REQ-FIX-003)

---

## 03 · Security-Critical Code Highlights

### 3.1 Permission Hardening — Three `#[cfg(unix)]` Blocks

All three permission-setting sites use the correct `#[cfg(unix)]` guard to ensure cross-platform compilation:

```rust
// duckdb.rs:62-68  — TempDirGuard::new()
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)) {
        eprintln!("Warning: could not set 0o700 on temp dir: {e}");
    }
}

// tantivy.rs:55-61  — TantivyIndex::open()
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)) {
        eprintln!("Warning: could not set 0o700 on index dir: {e}");
    }
}

// snapshot.rs:192-198  — save_snapshot_data()
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
        eprintln!("Warning: could not set 0o600 on snapshot file: {e}");
    }
}
```

### 3.2 Snapshot String Max-Length Guard

```rust
// snapshot.rs:117-124
const MAX_STRING_LEN: usize = 1024;
if len > MAX_STRING_LEN {
    return Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("string length {len} exceeds maximum {MAX_STRING_LEN}"),
    ));
}
```

### 3.3 Strict UTF-8 in Snapshot Deserialisation

```rust
// snapshot.rs:132-137 — was from_utf8_lossy, now from_utf8 with error
String::from_utf8(bytes).map_err(|e| {
    std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        format!("invalid UTF-8 in snapshot string: {e}"),
    )
})
```

### 3.4 TOCTOU-Free Snapshot Load

```rust
// hnsw.rs:454-472
fn load_snapshot(&self, path: &Path) -> VectorIndexResult<usize> {
    let file = std::fs::File::open(path)?;          // Open first
    let metadata = file.metadata()?;                 // Check on opened handle
    // ... validate ...
    let (count, data, loaded_removed) =
        snapshot::load_snapshot_data(file, self.dimension)?;  // Pass opened File
    // ...
}
```

### 3.5 Engine Drop + Idempotent Shutdown

```rust
// engine/mod.rs:419-444 — shutdown()
pub fn shutdown(&mut self) -> EngineResult<()> {
    if let Some(ref cancel) = self.snapshot_cancel {
        cancel.store(true, Ordering::Relaxed);
    }
    if let Some(handle) = self.snapshot_handle.take() {  // Idempotent via take()
        handle.join().map_err(|_| {
            EngineError::Internal("snapshot thread panicked".into())
        })?;
    }
    // ...
}

// engine/mod.rs:451-457 — Drop
impl Drop for Engine {
    fn drop(&mut self) {
        let _ = self.shutdown();  // Best-effort, no panic
    }
}
```

### 3.6 Analytics Timestamp Validation

```rust
// duckdb.rs:340-347 (sessions)
let created_at = json["createdAt"].as_str().unwrap_or("");
if created_at.is_empty() {
    eprintln!("[contexter] Warning: skipping session '{key_str}': missing/empty created_at");
    continue;  // Skip before CAST
}
```

---

## 04 · Remediation Recommendations

> **Must Fix**
> *(none — all 5 bug contracts verified zero issues)*

> **Should Fix**
> *(none)*

> **Consider**
> *(none)*

---

## 05 · Verification Summary

| Bug Contract | REQs | Result |
|---|---|---|
| Bug-Permissions-Hardening | 4/4 | ✅ All applied (TempDir 0o700, Tantivy 0o700, Snapshot 0o600, Test updated) |
| Bug-Snapshot-Robustness | 3/3 | ✅ All applied (Max-length 1024, Strict UTF-8, TOCTOU fix) |
| Bug-Engine-Drop | 3/3 | ✅ All applied (Drop impl, Idempotent shutdown, Thread joined) |
| Bug-Analytics-Sync | 1/1 | ✅ Applied (created_at validated in sessions, memories, telemetry) |
| Bug-File-Security (prev) | 2/2 | ✅ Both still correct (0o700 on dirs, TOCTOU mitigation) |

**Total: 13/13 requirements verified · 0 findings · 0 vulnerabilities introduced**

---

_Generated by Security Architect · 2026-07-25 · Validation Contract: `2026-07-25-contexter-phase2-search-analytics` · Auto Bug Loop Iteration 2_
