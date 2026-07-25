# Bug: DuckDB Analytics Pipeline Not Functional

**Severity:** CRITICAL  
**Root Cause:** Three interrelated issues prevent the L5 analytics tier from working with real data.

## Requirements

### REQ-FIX-001: Implement parameter binding in DuckDbEngine::query()
The `DuckDbEngine::query()` method accepts `params: &[Value]` but calls `stmt.query([])` with an empty slice. Fix by converting `Value` to `duckdb::ToSql` types and passing them to `stmt.query(params)`.

### REQ-FIX-002: Wire StorageBackend from Engine to DuckDbEngine
`Engine::with_config()` must call `duckdb_engine.set_storage_backend(backend)` after constructing the analytics engine. The `DuckDbEngine` already has the `set_storage_backend()` method; it just needs to be called.

### REQ-FIX-003: Implement real RocksDB sync in DuckDbEngine::sync()
Replace hardcoded sample data in `sync()` with real RocksDB column family iteration via the storage backend. Iterate the specified column family's entries and insert them into the DuckDB table.
