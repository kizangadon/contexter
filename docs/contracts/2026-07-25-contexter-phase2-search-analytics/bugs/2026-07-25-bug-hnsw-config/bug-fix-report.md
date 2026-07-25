# Bug Fix Report: Bug-HNSW-Config — HNSW Configuration Not Exposed

**Date:** 2026-07-25  
**Branch:** `feature/contexter-phase2-search-analytics`  
**Bug contract:** `docs/contracts/2026-07-25-contexter-phase2-search-analytics/bugs/2026-07-25-bug-hnsw-config/`

---

## Summary

The `HnswVectorIndex` hardcoded its HNSW parameters (`M=16`, `ef_construction=200`, `ef_search=50`) inside the implementation instead of accepting them from `EngineConfig`. This prevented callers from tuning recall-vs-performance tradeoffs at the engine level.

This fix exposes all three parameters through `EngineConfig` and wires them through `Engine::with_config()` to `HnswVectorIndex`.

---

## Changes Made

### 1. `contexter-core/src/engine/mod.rs` — `EngineConfig`

**Added three new fields:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `hnsw_m` | `usize` | `16` | Max connections per element (M parameter). Reserved for future library support. |
| `hnsw_ef_construction` | `usize` | `200` | Candidate neighbours during construction (efConstruction). |
| `hnsw_ef_search` | `usize` | `50` | Candidate neighbours during search (ef). |

- `Default` impl updated to return the default values.
- `Engine::with_config()` now passes all three values to `HnswVectorIndex::new()`.

### 2. `contexter-core/src/vector/hnsw.rs` — `HnswVectorIndex`

**Struct changes:**
- Added `m: usize`, `ef_construction: usize`, `ef_search: usize` fields.
- `Debug` impl updated to display the new fields.

**Constructor change:**
- `new(dimension: usize)` → `new(dimension: usize, m: usize, ef_construction: usize, ef_search: usize)`.
- Uses `Builder::default().ef_construction(ef_construction).ef_search(ef_search)` to configure the `instant_distance` builder.

**`rebuild()` change:**
- Now uses stored `ef_construction` and `ef_search` fields when constructing the `Builder` (instead of `Builder::default()`).

### 3. Test call sites (12 locations in `hnsw.rs`)

All `HnswVectorIndex::new(dimension)` calls updated to `HnswVectorIndex::new(dimension, 16, 200, 50)`.

---

## Design Decisions

- **`hnsw_m` is stored for forward-compatibility**: The underlying `instant_distance` library (v0.6.1) hardcodes `const M: usize = 32` and does not expose it on `Builder`. We accept and store the value so it can be plumbed when the library adds support. It is documented in both `EngineConfig` and `HnswVectorIndex`.
- **`ef_construction` and `ef_search` are fully wired**: `Builder` exposes `.ef_construction()` and `.ef_search()`, so these are actively used during graph construction and search.
- **Defaults match the original hardcoded values**: `M=16`, `ef_construction=200`, `ef_search=50` preserve backward compatibility.

---

## Verification

- `cargo build`: ✅ Compiles with no new warnings
- `cargo clippy --all-targets --all-features`: ✅ No new warnings from our changes (pre-existing warnings in `tantivy.rs`, `analytics.rs`, test files unchanged)
- `cargo test --lib`: ✅ 298/298 tests pass

---

## Edge Cases Assessment

From `EDGE_CASES.md`:
- **EC-01 (M=0 or M=1)**: Not validated — `instant_distance` does not expose M via Builder, so validation applies only when the library adds support.
- **EC-02 (ef_construction < ef_search)**: Accepted without validation; `instant_distance` handles this internally.
- **EC-03 (Very large ef_search)**: Accepted; user bears performance cost as expected.

---

## Acceptance Criteria Status

| AC | Description | Status |
|----|-------------|--------|
| AC-01 | `EngineConfig` has `hnsw_M`, `hnsw_ef_construction`, `hnsw_ef_search` fields with defaults | ✅ |
| AC-02 | `HnswVectorIndex::new()` accepts M, ef_construction, ef_search parameters | ✅ |
| AC-03 | Values are wired from EngineConfig through `Engine::with_config()` to `HnswVectorIndex` | ✅ |
| AC-04 | All existing tests continue to pass | ✅ (298/298) |
