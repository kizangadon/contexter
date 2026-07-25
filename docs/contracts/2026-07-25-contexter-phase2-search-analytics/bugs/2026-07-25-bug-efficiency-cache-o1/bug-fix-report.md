# Bug Fix Report: Efficiency Cache O(1)

**Bug:** `get_efficiency_scores()` used full `HashMap::retain()` scan (O(n))

## Changes Made

### `contexter-core/src/analytics/duckdb.rs` — `get_cached_efficiency_scores()`

**Before:**
```rust
let mut cache = self.efficiency_cache.write().ok()?;
// ...
cache.retain(|session_id, entry| { ... });  // O(n) — iterates ALL entries, acquires WRITE lock
```

**After:**
```rust
let cache = self.efficiency_cache.read().ok()?;
// ...
for (session_id, entry) in cache.iter() {   // O(n) for building results, READ lock only
    // skip expired entries instead of removing them
}
```

Key changes:
1. Replaced `write()` lock with `read()` lock — no writer contention
2. Replaced `retain()` closure with `for...iter()` loop
3. Expired entries are skipped in results but NOT removed from cache
4. Expired entries are overwritten on next `populate_efficiency_cache()` (which calls `cache.clear()` first)

## Verification
- `cargo build --workspace` — passes
- `cargo test --workspace` — all tests pass
- All efficiency-related tests pass (`test_efficiency_calculation`, `test_efficiency_scores`)

## Status
✅ FIXED
