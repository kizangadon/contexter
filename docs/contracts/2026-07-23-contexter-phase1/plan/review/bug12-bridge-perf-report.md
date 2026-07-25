# Bug 12: Python Bridge Performance — Fix Report

## Summary

Removed double JSON serialization overhead for large memories (>100 KB) and made `max_workers` configurable instead of hardcoded to 4.

## Files Changed

| File | Change |
|------|--------|
| `python/core_bridge.py` | Instance-level `ThreadPoolExecutor` with configurable `max_workers`; PyBytes path for memories >100 KB |
| `src/python.rs` | New `create_memory_bytes` + `update_memory_bytes` pyfunctions accepting raw `&[u8]` content |
| `src/storage/rocksdb_backend.rs` | Pre-existing: added missing `wal_sync: true` in `RocksDbConfig` initializer inside `open()` |

## Changes Detail

### 1. Configurable `max_workers` (AC-1 ✅)

**`python/core_bridge.py`** — Constructor now accepts `max_workers: int = 4`:

```python
def __init__(self, path: str, max_workers: int = 4):
    if max_workers <= 0:
        max_workers = 4
    self._pool = ThreadPoolExecutor(max_workers=max_workers)
```

- `max_workers=0` defaults to 4 (EDGE_CASES compliance)
- `max_workers=1` works as sequential execution (graceful fallback)
- Class-level `_executor` removed — each `Engine` instance owns its pool
- `open()` classmethod also accepts `max_workers`

### 2. PyBytes Path for Large Memories (AC-2 ✅)

**Python bridge routing:**
- `_MAX_MEMORY_JSON_SIZE = 102_400` (100 KB threshold)
- `create_memory`: if `len(content) > 100 KB`, splits metadata + content bytes, calls `_engine.create_memory_bytes`
- `update_memory`: same split when patch contains content > 100 KB

**Rust pyfunctions added:**

- `fn create_memory_bytes(&self, meta_json: &str, content: &[u8]) -> PyResult<String>`
  - Deserializes `NewMemory` from `meta_json`, overrides `content` with `String::from_utf8(content)`
  
- `fn update_memory_bytes(&self, id: &str, patch_meta_json: &str, content: &[u8]) -> PyResult<Option<String>>`
  - Parses UUID, deserializes `MemoryPatch` from `patch_meta_json`, sets `patch.content`, calls engine

Both functions follow existing patterns: `catch_panic`, depth-limited JSON parsing, `map_err` error propagation.

### 3. Pre-existing Bug Fix

`src/storage/rocksdb_backend.rs:168` — missing `wal_sync: true` field in `RocksDbConfig` struct initializer within the `open()` convenience method. This was a compilation error on the current branch.

## Performance Impact

| Before | After |
|--------|-------|
| ThreadPoolExecutor hardcoded to 4 workers | Configurable per `Engine` instance |
| All memories pass through `json.dumps` → `json.loads` round-trip | Memories >100 KB bypass JSON for content field |
| Double serialization: Python dict → JSON str → Rust `&str` → JSON parse → `NewMemory` | Single serialization: metadata → JSON str, content → raw `&[u8]` → `String::from_utf8` |

## Tests Added

4 new Rust tests in `src/python.rs`:

| Test | What it verifies |
|------|-----------------|
| `test_py_create_memory_bytes` | Full create → get round-trip via bytes path |
| `test_py_update_memory_bytes` | Create → update (bytes) → version bump |
| `test_py_memory_bytes_invalid_utf8_produces_error` | Invalid UTF-8 bytes → `PyValueError` |
| `test_py_memory_bytes_update_nonexistent` | Update non-existent UUID → `Ok(None)` |

## Verification Results

```
cargo test   → 181 passed (168 unit + 13 integration)
cargo clippy → 0 warnings, clean
```
