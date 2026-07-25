# Bug 13: RwLock Contention (Perf H5 Remainder, M5)

## Problem
`SharedBackend` RwLock serializes all writes and long-running iterators hold read locks blocking writers. Additionally, `storage_size()` acquires 24 RocksDB property locks. L1 `extract_entity_type` string split adds overhead per cache op. No WriteBatch API.

## Fix Requirements
1. Move search/list iterators to release the read lock between batch reads (process in chunks)
2. Optimize `storage_size()` to batch property queries
3. Add WriteBatch::default() to `RocksDbBackend` for atomic multi-CF writes (future use)
4. Remove `inserted_at` dead code from `CacheEntry` (L4 finding)
