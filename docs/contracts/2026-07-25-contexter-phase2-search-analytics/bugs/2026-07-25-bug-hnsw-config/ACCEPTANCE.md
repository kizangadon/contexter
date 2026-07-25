# Acceptance Criteria — Bug-HNSW-Config

- AC-01: `EngineConfig` has `hnsw_M`, `hnsw_ef_construction`, `hnsw_ef_search` fields with defaults
- AC-02: `HnswVectorIndex::new()` accepts M, ef_construction, ef_search parameters
- AC-03: Values are wired from EngineConfig through `Engine::with_config()` to `HnswVectorIndex`
- AC-04: All existing tests continue to pass
