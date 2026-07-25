# Bug: Efficiency Cache Missing Column Family and Per-Session Caching

**Severity:** HIGH  
**Root Cause:** `DuckDbEngine` has `SESSIONS_CF` hardcoded but `efficiency_map` column family is never iterated in `sync()`. No per-session caching.

## Requirements

### REQ-FIX-001: Add EFFICIENCY_CF constant to DuckDbEngine
Add `const EFFICIENCY_CF: &str = "efficiency_map";` in `analytics/duckdb.rs`.

### REQ-FIX-002: Sync efficiency_map data in DuckDbEngine::sync()
When `column_family = "efficiency_map"`, iterate the RocksDB column family and populate an `efficiency_cache: HashMap<String, f64>` field.

### REQ-FIX-003: Add per-session caching for get_efficiency_scores()
`get_efficiency_scores()` should check an in-memory cache before hitting DuckDB. Cache TTL should be configurable (default 60s).

### REQ-FIX-004: Implement cache eviction
Add `time::Instant` based TTL check on cache entries. Expired entries are evicted on next read.
