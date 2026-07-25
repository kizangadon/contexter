# Bug Fix Report: Cache TTL Eviction, WriteBatch, storage_size Batching, Chunked Iteration

## Summary
Four related fixes for cache/storage performance and correctness in contexter phase-1 core.

## Changes

### 1. Cache TTL Eviction (Lazy)
**File:** `src/cache/mod.rs`

- Added `max_ttl: Option<Duration>` to `CacheConfig` (default: `None`)
- Removed `#[allow(dead_code)]` from `CacheEntry::inserted_at`
- `get()` now checks TTL lazily: if entry exists but `inserted_at.elapsed() > max_ttl`, evict and treat as miss
- Fixed bug: original `get()` used `?` operator on `entity_type.and_then(|et| inner.get_mut(et))?`, which returned early when no LruCache existed, **skipping** `misses.fetch_add`. Fixed by using `.and_then()` chain to always increment misses on lookup failure.
- Clippy fix: `map_or(false, ...)` → `is_some_and(...)`
- Updated all `CacheConfig` constructors (3 cache tests, 1 engine test, 1 integration test) to include `max_ttl`

### 2. WriteBatch API
**Files:** `src/storage/mod.rs`, `src/storage/rocksdb_backend.rs`

- Added `write_batch(&self, column_family: &str, entries: Vec<(Vec<u8>, Vec<u8>)>) -> EngineResult<()>` to `StorageBackend` trait
- Implemented in `RocksDbBackend` using `rocksdb::WriteBatch`
- Added `scan_cf_keys(&self, column_family: &str, prefix: &str) -> EngineResult<Vec<Vec<u8>>>` to trait + impl

### 3. storage_size Batching
**File:** `src/storage/rocksdb_backend.rs`

- Reduced from 3 property-value calls per CF (`estimate-live-data-size`, `cur-size-all-mem-tables`, `total-sst-files-size`) to 2 (`estimate-live-data-size` + `cur-size-all-mem-tables`) using `max()`
- Uses `max()` to ensure non-zero results when data is in memtable but not yet flushed to SST
- Total calls: 19 (was 28) = 18 property CF calls + 1 WAL size

### 4. Chunked Iteration (RwLock Performance)
**File:** `src/engine/mod.rs`

- Added `BATCH_SIZE = 100` constant
- Added imports for `CF_*` and `KEY_PREFIX_*` constants
- Rewrote `search_memories`: `scan_cf_keys` → batch of 100 at a time → release read lock between batches
- Same pattern applied to `list_sessions`, `list_agents`, `list_skills`, `query_audit`

## Tests
- 168 unit tests pass
- 13 integration tests pass
- `cargo clippy --all-targets -- -D warnings`: clean
- `cargo check`: clean

## Test Failures Fixed
1. `test_cache_hit_ratio`: miss counter skipped on early return from `?` operator
2. `test_cache_telemetry_tracks_hits_and_misses`: same root cause
3. `test_cache_clear_and_clear_type`: same root cause
4. `test_storage_size_non_zero`: `estimate-live-data-size` returned 0 for unflushed data
5. `test_storage_size_report`: same root cause
