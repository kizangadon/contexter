# Bug: DuckDB Mutex<Connection> Serialization + Individual Memory Fetches + Non-Incremental Sync

**Severity:** MEDIUM  
**Root Cause:** Three related issues: (1) Single `Mutex<Connection>` blocks all queries during sync, (2) Hybrid search fetches Memory objects individually, (3) Analytics sync uses truncate+re-insert instead of incremental upsert.

## Requirements

### REQ-FIX-001: Add batch get_memories to StorageBackend
Add `fn get_memories(&self, ids: &[String]) -> EngineResult<Vec<Option<Memory>>>` to the `StorageBackend` trait and implement it in `RocksDbBackend`. Update hybrid search in `engine/search.rs` to use batch fetch instead of individual `get_memory` calls per result.

### REQ-FIX-002: Split DuckDB connection
Replace the single `Mutex<Connection>` with a read-write split: one read connection (not locked for writes) and one write connection. Reads use the read connection (no contention); sync uses the write connection.

### REQ-FIX-003: Incremental sync
Change analytics sync from truncate+re-insert to upsert/delta-sync. Track last-sync timestamp and only process records newer than that.
