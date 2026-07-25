# Acceptance Criteria — Bug-DB-Analytics

- AC-01: `DuckDbEngine::query()` with `params` binds them correctly to DuckDB prepared statement (not discarded)
- AC-02: `get_session_count_by_range()` with start/end timestamps returns filtered results (not all sessions)
- AC-03: `Engine::with_config()` passes `StorageBackend` to `DuckDbEngine`
- AC-04: `sync("sessions")` reads real data from RocksDB instead of hardcoded sample data
- AC-05: `run_analytics()` returns results based on actual stored data, not sample data
- AC-06: All existing tests continue to pass
