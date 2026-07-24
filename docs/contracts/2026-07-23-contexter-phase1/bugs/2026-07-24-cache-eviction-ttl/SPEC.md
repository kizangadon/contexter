# Bug 9: Cache TTL Eviction (Perf H4)

## Problem
LRU cache has no TTL eviction. Entries live forever until pushed out by LRU. In pathological cases, total cache memory could reach 20+ GB.

## Fix Requirements
1. Add TTL field to `CacheEntry` (use existing `inserted_at: Instant` field)
2. Add `max_ttl: Duration` to `CacheConfig` (default: None = no TTL eviction)
3. On cache get/store, evict entries whose TTL has expired
4. Remove `#[allow(dead_code)]` from `inserted_at` and use it
