# Bug-Fix Report: Efficiency Cache Column Family & Per-Session Caching

| Field | Detail |
|---|---|
| **Bug Contract** | `2026-07-25-bug-efficiency` |
| **Fix Applied** | ✅ Yes (part of initial Phase 2 implementation + Bug-Efficiency Worker) |
| **Worker** | Bug-Efficiency Worker |

## Changes Applied

### REQ-FIX-001: Add `EFFICIENCY_CF` constant
- Added `pub const EFFICIENCY_CF: &str = "efficiency_map";` at `analytics/duckdb.rs:27`

### REQ-FIX-002: Sync efficiency_map data in `sync()`
- Added `sync_efficiency_cache_from_backend()` method that iterates the RocksDB `efficiency_map` column family
- Called at `duckdb.rs:550` when `cf_name == EFFICIENCY_CF`

### REQ-FIX-003: Per-session caching for `get_efficiency_scores()`
- Added `efficiency_cache: Arc<RwLock<HashMap<String, EfficiencyEntry>>>` field at `duckdb.rs:92`
- `get_efficiency_scores()` checks cache first (line 600), falls back to DuckDB query
- `populate_efficiency_cache()` fills cache from query results (line 540)

### REQ-FIX-004: TTL-based cache eviction
- `EfficiencyEntry` struct includes `cached_at: Instant`
- Cache entries have TTL of 60 seconds (configurable)
- Expired entries are evicted on next read

## Verification

- ✅ `grep EFFICIENCY_CF` → 4 matches (constant + usages)
- ✅ `grep efficiency_cache` → 9 matches (field, init, read, write)
- ✅ `cargo build --workspace` — compiles cleanly
