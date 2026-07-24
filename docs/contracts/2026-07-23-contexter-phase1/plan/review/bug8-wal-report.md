# Bug 8: WAL Flush Optimization — Implementation Report

**Date:** 2026-07-24
**Branch:** `feature/contexter-phase1-core`
**File:** `src/storage/rocksdb_backend.rs`
**Author:** Distinguished Backend Engineer

---

## Summary

Added `wal_sync: bool` config flag to `RocksDbConfig` that controls whether
`DB::flush_wal(true)` is called after every mutating operation. When set to
`false`, the fsync-per-write overhead is eliminated, and durability is
deferred to explicit `checkpoint()` calls.

---

## Changes

### 1. `RocksDbConfig` — new `wal_sync` field

```rust
pub struct RocksDbConfig {
    pub path: String,
    pub create_if_missing: bool,
    pub wal_sync: bool,           // ← new, default: true
}
```

- Default value: `true` (preserves existing behaviour — every write fsyncs the
  WAL, guaranteeing durability before returning).
- `RocksDbBackend::open()` uses `Default::default()` for config, so all
  existing callers automatically get `wal_sync = true` with zero changes.

### 2. `RocksDbBackend` — store config, not `_config`

Renamed the field from `_config: RocksDbConfig` to `config: RocksDbConfig` so
the new `maybe_flush_wal()` method can read `self.config.wal_sync`.

### 3. `maybe_flush_wal()` helper

Added a private method:

```rust
fn maybe_flush_wal(&self) -> EngineResult<()> {
    if self.config.wal_sync {
        self.db.flush_wal(true)?;
    }
    Ok(())
}
```

When `wal_sync = true` (default): calls `flush_wal(true)` — identical to
before.

When `wal_sync = false`: no-op. The write is buffered in the OS page cache /
RocksDB memtable and flushed on the next explicit `checkpoint()` or RocksDB
internal WAL sync.

### 4. All 14 mutating operations → `maybe_flush_wal()`

| Operation | File location (approx.) |
|-----------|------------------------|
| `create_session` | line 343 |
| `update_session` | line 427 |
| `delete_session` | line 436 |
| `create_memory` | line 500 |
| `update_memory` | line 641 |
| `delete_memory` | line 650 |
| `create_agent` | line 724 |
| `update_agent` | line 819 |
| `delete_agent` | line 828 |
| `create_skill` | line 854 |
| `update_skill` | line 934 |
| `delete_skill` | line 943 |
| `set_setting` | line 974 |
| `append_audit_entry` | line 998 |

Every `self.db.flush_wal(true)?;` was replaced with `self.maybe_flush_wal()?;`.

### 5. `checkpoint()` — unconditional flush

`checkpoint()` is the one method that **always** calls `flush_wal(true)`,
regardless of `wal_sync`. Users who disable `wal_sync` for write throughput
rely on explicit `checkpoint()` calls to guarantee durability at a point they
choose.

---

## Verification

```
cargo check        → PASS
cargo test         → 168 unit + 13 integration = 181 tests, all PASS
cargo clippy --all-targets -- -D warnings → PASS
```

Existing tests exercise `wal_sync = true` (the default) via `setup_db()` which
calls `RocksDbBackend::open()`. No existing test regressed — all 181 tests
pass with no modifications.

---

## Performance Impact

When `wal_sync = false`:

- **Zero fsync calls** per write operation (was: 1 fsync per put/delete).
- Each saved fsync avoids 1-10 ms of kernel I/O latency.
- Durability granularity: instead of per-operation fsync, durability is
  guaranteed only at `checkpoint()` boundaries.
- RocksDB's own internal WAL sync still runs periodically
  (`wal_bytes_per_sync` defaults to 0 = OS-driven, but `wal_ttl_seconds`
  and `wal_size_limit_mb` provide bounded durability windows).

Recommended pattern for bulk writes:

```rust
let config = RocksDbConfig {
    wal_sync: false,  // disable per-write fsync
    ..RocksDbConfig::default()
};
let backend = RocksDbBackend::open_with_config(config)?;
// ... perform many writes ...
backend.checkpoint()?;  // one fsync at the end
```
