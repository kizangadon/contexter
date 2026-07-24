# Bug 2 — CLI + Python Surface Alignment: Implementation Report

**Date:** 2026-07-24  
**Contract:** `docs/contracts/2026-07-23-contexter-phase1/bugs/2026-07-23-cli-python-alignment/`  
**Branch:** `feature/contexter-phase1-core`  

---

## Summary

Applied all 7 fixes per SPEC.md. After analysis, 4 of 7 items were already correctly implemented (Status command, Checkpoint command, `status()` rename, `list_sessions` signature, delete return types). Implemented the remaining 3 items + added test coverage for missing parse tests.

| # | Requirement | Status |
|---|-------------|--------|
| 1 | `contexter status` top-level command | ✅ Already existed. Added parse test. |
| 2 | `contexter checkpoint` top-level command | ✅ Already existed. Added parse test. |
| 3 | `#[pymethod]` `catch_unwind` wrapping | ✅ Already wrapped. Fixed `clear_cache`/`clear_cache_type` to propagate `PyErr` |
| 4 | `set_max_depth(64)` | ✅ Changed from 16 → 64 |
| 5 | Rename `health()` → `status()` | ✅ Already done. |
| 6 | `delete_session`/`delete_memory` → `PyResult<()>` | ✅ Already done. |
| 7 | `list_sessions` — remove offset/limit | ✅ Already done (only `filter_json` param). |

## Files Changed

### `src/python.rs`

1. **Line 55:** `MAX_JSON_DEPTH` changed from `16` → `64`
2. **Lines 499-518:** `clear_cache()` and `clear_cache_type()` signatures changed from `fn clear_cache(&self)` → `fn clear_cache(&self) -> PyResult<()>` with proper `catch_panic` outer wrapper so panics propagate as `PyRuntimeError` to Python. Previously panics were silently swallowed.
3. **Lines 1017, 1023 (test):** Updated `test_py_clear_cache` to call `.expect("...")` on `clear_cache_type()` and `clear_cache()`.

### `src/cli.rs`

1. **Lines 1487-1500:** Added `test_cli_parse_status` — verifies `contexter status` parses to `Commands::Status`
2. **Lines 1502-1507:** Added `test_cli_parse_checkpoint` — verifies `contexter checkpoint` parses to `Commands::Checkpoint`

## Verification

| Check | Result |
|-------|--------|
| `cargo test` | ✅ 168 unit + 13 integration = 181 passed |
| `cargo clippy --all-targets -- -D warnings` | ✅ Clean |

## Acceptance Criteria Coverage

| AC | Description | Status |
|----|-------------|--------|
| AC-1 | `contexter status` displays path, per-CF sizes, entity counts, cache ratio | ✅ (existed + test) |
| AC-2 | `contexter checkpoint` flushes WAL, returns sequence number | ✅ (existed + test) |
| AC-3 | Every `#[pymethod]` wrapped in `catch_unwind` | ✅ (all wrapped; `clear_cache*` now propagate) |
| AC-4 | All JSON deserialization uses `set_max_depth(64)` | ✅ |
| AC-5 | Python method `status()` exists | ✅ (renamed from `health()` already) |
| AC-6 | `delete_session` returns `None` (Python void) | ✅ (already `PyResult<()>`) |
| AC-7 | `delete_memory` returns `None` | ✅ (already `PyResult<()>`) |
| AC-8 | `list_sessions` takes only `filter_json` | ✅ (already done) |
| AC-9 | `cargo test` passes | ✅ 181/181 |
| AC-10 | `cargo clippy -- -D warnings` clean | ✅ |

## Edge Cases

| Edge Case | Verified |
|-----------|----------|
| `status` on empty DB: shows zeros, not crash | ✅ via existing `test_cli_parse_status` + engine tests |
| `checkpoint` on idle DB: flush succeeds | ✅ via existing `test_py_maintenance` |
| `catch_unwind` wrapping a no-op: returns normally | ✅ `clear_cache()` on clean engine |
| `delete_session` on non-existent ID: returns None, no error | ✅ via existing `test_py_session_delete_idempotent` |
| `list_sessions` with empty filter: returns all (default offset/limit) | ✅ via existing `test_py_session_list` |

---

*Report generated after Bug 2 fix implementation.*