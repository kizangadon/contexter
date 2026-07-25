# Bug 10: Cache Store Domain Objects — Implementation Report

## Summary

Changed `DashMapCache` value storage from raw `Vec<u8>` (serialized JSON bytes) to a
typed `CachedValue` enum containing domain objects directly. Cache hits now return
typed objects without JSON deserialization.

## Changes

### `src/cache/mod.rs`

- **Added `CachedValue` enum** with variants:
  - `CachedValue::Session(Session)` — typed session entity
  - `CachedValue::Memory(Memory)` — typed memory entity
  - `CachedValue::Agent(Agent)` — typed agent entity
  - `CachedValue::Skill(Skill)` — typed skill entity
  - `CachedValue::Raw(Vec<u8>)` — raw bytes for non-domain data (settings, audit)
- **Changed `CacheEntry.data`** field type from `Vec<u8>` to `CachedValue`
- **Changed `DashMapCache::store()`** signature: `&[u8]` → `CachedValue`
- **Changed `DashMapCache::get()`** return type: `Option<Vec<u8>>` → `Option<CachedValue>`
- Updated all cache tests to use `CachedValue::Raw(...)` instead of byte slices

### `src/engine/mod.rs`

- **`create_session`**: Stores `CachedValue::Session(session)` instead of serializing to bytes
- **`get_session`**: Matches `CachedValue::Session(session)` directly — no JSON deserialization
- **`create_memory`**: Stores `CachedValue::Memory(memory)` instead of serializing
- **`get_memory`**: Matches `CachedValue::Memory(memory)` directly — no JSON deserialization
- **`create_agent`**: Stores `CachedValue::Agent(agent)` instead of serializing
- **`get_agent`**: Matches `CachedValue::Agent(agent)` directly — no JSON deserialization
- **`create_skill`**: Stores `CachedValue::Skill(skill)` instead of serializing
- **`get_skill`**: Matches `CachedValue::Skill(skill)` directly — no JSON deserialization
- **`set_setting` / `get_setting`**: Uses `CachedValue::Raw(bytes)` for backward-compatible
  string storage
- Removed all `serde_json::to_vec` / `serde_json::from_slice` calls from the hot path
- Removed `serde_json::Error` conversion dependency from CRUD methods

## Performance

- **Cache hit**: Previously required `serde_json::from_slice` → now zero-cost
  pattern match
- **Cache miss (populate)**: Previously serialized to bytes → now clones the domain
  object (clone is cheaper than JSON serialization for typical object sizes)
- **Clone independence preserved**: `CachedValue` is `Clone`, and `get()` returns
  a clone — mutating the returned value does not affect the cache

## Verification

- `cargo test` — 168 unit tests + 13 integration tests: **all pass**
- `cargo clippy --all-targets -- -D warnings` — **clean**

## Files Changed

| File | Lines Changed |
|------|--------------|
| `src/cache/mod.rs` | ~150 (added CachedValue enum, updated CacheEntry, store/get signatures, tests) |
| `src/engine/mod.rs` | ~60 (updated all CRUD methods to use CachedValue) |
