# Bug 4 — Telemetry + Test Fix Implementation Report

**Date:** 2026-07-24
**Feature:** contexter-phase1-core
**Bug Slug:** telemetry-tests

---

## Changes Made

### 1. `python/core_bridge.py` — 4 fixes

| # | Issue | Before | After |
|---|-------|--------|-------|
| A | `list_sessions` passed extra positional args | `.list_sessions(filter_json, 0, 50)` | `.list_sessions(filter_json)` |
| B | `status` called nonexistent `health` method | `self._engine.health` | `self._engine.status` |
| C | Missing `cache_telemetry()` wrapper | Not present | Added after `clear_cache` |
| D | Missing `clear_cache_type()` wrapper | Not present | Added after `cache_telemetry` |

### 2. `tests/integration_test.rs` — 3 fixes

| # | Issue | Fix |
|---|-------|-----|
| 1 | `SessionPatch` not imported | Added to `use contexter_core::{...}` block |
| 2 | Second `#[cfg(unix)]` block in `test_read_only_path_error` missing `PermissionsExt` import | Added `use std::os::unix::fs::PermissionsExt;` inside block |
| 3 | Top-level `PermissionsExt` import was unused (both usage sites had inner `use` already) | Removed top-level `#[cfg(unix)] use std::os::unix::fs::PermissionsExt;` |

---

## Test Results

### `cargo test` — 179 tests, 0 failed

- **Unit tests:** 166 passed
- **Integration tests:** 13 passed (including `test_read_only_path_error`)
- **Doc tests:** 0 (none)
- **Exit code:** 0

### `cargo clippy --all-targets -- -D warnings` — Clean

- **Warnings:** 0
- **Errors:** 0
- **Exit code:** 0

---

## Acceptance Criteria Verification

| AC | Description | Status |
|----|-------------|--------|
| AC‑1 | `list_sessions` — extra `0, 50` args removed | ✅ Fixed |
| AC‑2 | `status` — calls correct Rust `status()` method | ✅ Fixed |
| AC‑3 | `cache_telemetry` + `clear_cache_type` methods added | ✅ Added |
| AC‑4 | `test_read_only_path_error` compiles and passes | ✅ Passes |
| AC‑5 | `cargo test` passes | ✅ 179/179 |
| AC‑6 | `cargo clippy --all-targets -- -D warnings` clean | ✅ 0 warnings |

---

## Edge Case Verification

| Edge Case | Status |
|-----------|--------|
| EC‑1: Python async wrapper invalid path raises on construction (not on first call) | ✅ Constructor calls `_SyncEngine.open(path)` which raises immediately on invalid path |
| EC‑2: `test_read_only_path_error` correctly handles read-only dir error | ✅ Engine::open returns error on read-only path |
| EC‑3: `test_read_only_path_error` restores permissions on non-Unix gracefully | ✅ Uses `#[cfg(unix)]` — no-op on non-Unix |
