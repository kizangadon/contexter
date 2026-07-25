# Bug: HNSW Configuration Not Exposed

**Severity:** MEDIUM  
**Root Cause:** `HnswVectorIndex` hardcodes `M=16`, `ef_construction=200`, `ef_search=50` instead of accepting config from `EngineConfig`.

## Requirements

### REQ-FIX-001: Add hnsw_M, hnsw_ef_construction, hnsw_ef_search to EngineConfig
Add `hnsw_M: usize` (default 16), `hnsw_ef_construction: usize` (default 200), `hnsw_ef_search: usize` (default 50) fields to `EngineConfig`.

### REQ-FIX-002: Pass hnsw params to HnswVectorIndex::new()
`Engine::with_config()` should pass these config values to `HnswVectorIndex::new()`.

### REQ-FIX-003: Use params in index construction
`HnswVectorIndex::new()` should use supplied M, ef_construction, ef_search values instead of hardcoded constants.
