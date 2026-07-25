# Acceptance Criteria — Bug-Efficiency

- AC-01: `sync("efficiency_map")` reads real data from RocksDB into efficiency cache
- AC-02: `get_efficiency_scores()` returns cached data for repeated calls within TTL
- AC-03: `get_efficiency_scores()` returns fresh data after TTL expires
- AC-04: Cache is populated from actual RocksDB data, not hardcoded values
- AC-05: All existing tests continue to pass
