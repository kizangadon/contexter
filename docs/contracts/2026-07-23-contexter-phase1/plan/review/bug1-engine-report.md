# Bug 1: Engine Abstraction + Generic KV + StorageConfig — Fix Report

**Bug:** `2026-07-23-engine-abstraction`
**Status:** ✅ Resolved
**Date:** 2026-07-24

## Changes Made

### 1. CLI Default Path → `~/.contexter/` (AC-8)

**File:** `src/cli.rs`

Changed the default database path resolution from `dirs::data_dir().join("contexter")` (resolves to `~/.local/share/contexter` on Linux) to `dirs::home_dir().join(".contexter")` (resolves to `~/.contexter/`).

- Updated the `db_path` doc comment to reflect the new default
- Updated the resolution fallback chain: `--db-path` / `CONTEXTER_DB_PATH` → `~/.contexter/` → `./contexter_data` (last-resort fallback)

### 2. Python API: Rename `store_raw` → `store`, `get_raw` → `get` (AC-5, AC-6)

**File:** `src/python.rs`

Renamed `PyEngine::store_raw` → `PyEngine::store` and `PyEngine::get_raw` → `PyEngine::get` so the Python-facing API matches the `store(cf, key, value)` / `get(cf, key)` naming contract.

Both methods proxy to `Engine::store()` / `Engine::get()` respectively, which delegate to `StorageBackend::store_raw()` / `StorageBackend::get_raw()` on the backend.

### Pre-existing (already correct at time of bug)

The following contractual requirements were already implemented in the codebase before this fix:

| Requirement | Status | Location |
|---|---|---|
| AC-1: `SharedBackend` type alias | ✅ Exists | `src/storage/mod.rs:22` |
| AC-2: `Engine` uses `SharedBackend` | ✅ Exists | `src/engine/mod.rs:124` |
| AC-3: `Engine::store(cf, key, value)` | ✅ Exists | `src/engine/mod.rs:544` |
| AC-4: `Engine::get(cf, key)` | ✅ Exists | `src/engine/mod.rs:550` |
| AC-7: `StorageConfig` struct | ✅ Exists | `src/engine/mod.rs:104` |
| AC-9: Tests pass | ✅ Verified | 179 tests pass |
| AC-10: Clippy clean | ✅ Verified | `-D warnings` passes |

## Verification

```text
$ cargo test
... 166 passed (unit) + 13 passed (integration) = 179 total, 0 failed

$ cargo clippy --all-targets -- -D warnings
... clean, no warnings
```

## Edge Cases Covered

| Edge Case | Handling |
|---|---|
| `StorageConfig` with non-existent path | `Engine::open` creates it (RocksDB `create_if_missing: true`) |
| Generic store with empty key | Passes through to RocksDB (permitted) |
| Generic get with non-existent key | Returns `None` |
| `SharedBackend` with 0 backends | Won't compile (trait bound `StorageBackend: Send + Sync`) |

## Files Touched

- `src/cli.rs` — Default path changed to `~/.contexter/`
- `src/python.rs` — Renamed `store_raw`→`store`, `get_raw`→`get`
