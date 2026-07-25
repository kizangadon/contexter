# Bug 9+13: Cache TTL + RwLock Performance (Perf H4, H5, M5, L2, L3, L4)

## Problem
Multiple performance issues in cache and RwLock patterns:
- LRU cache has no TTL eviction (H4)
- RwLock write().unwrap() on every mutation creates double bottleneck with WAL (H5)
- Long-running iterators hold read().unwrap() blocking writers (M5)
- storage_size() locks 24 RocksDB properties (L2)
- No WriteBatch API for atomic multi-CF writes (L3)
- inserted_at dead code on CacheEntry (L4)

## Fix Requirements
1. **Cache TTL eviction**: Use `inserted_at: Instant` in CacheEntry for TTL eviction. Add `max_ttl: Option<Duration>` to CacheConfig. Remove `#[allow(dead_code)]` from inserted_at and use it. On cache access, evict expired entries.
2. **Chunked iteration**: In search_memories and list methods, release the read lock between batch reads to allow writers to make progress.
3. **storage_size optimization**: Batch property queries into fewer lock acquisitions.
4. **WriteBatch API**: Add `write_batch()` method to StorageBackend trait and RocksDbBackend.
5. **Remove old inserted_at** if TTL implementation requires a new approach.
