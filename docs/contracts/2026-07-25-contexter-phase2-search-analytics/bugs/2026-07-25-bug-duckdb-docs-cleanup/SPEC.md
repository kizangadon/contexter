# Bug: DuckDB Doc Comment Claims Read/Write Connection Split That Doesn't Exist

**Severity:** LOW  
**Root Cause:** A doc comment in `DuckDbEngine` describes "two separate DuckDB connections" for reads and writes, but `duckdb::Connection` is `!Sync` making this infeasible. The doc is misleading.

## Requirements

### REQ-FIX-001: Fix misleading doc comment
Update the struct-level doc comment in `DuckDbEngine` (analytics/duckdb.rs) to accurately state that a single `Mutex<Connection>` is used. Remove any mention of "two separate connections".

### REQ-FIX-002: Lock contention documentation
Add a doc comment noting that the single connection is a known limitation (due to duckdb::Connection being !Sync) and that incremental sync mitigates write duration.
