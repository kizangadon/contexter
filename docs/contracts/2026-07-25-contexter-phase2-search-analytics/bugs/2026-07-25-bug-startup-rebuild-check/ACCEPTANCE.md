# Acceptance Criteria

### AC-01: Startup count comparison
GIVEN `Engine::with_config()` starts with both L2 storage and HNSW index  
WHEN the vector index is loaded  
THEN a comparison of L2 memory count vs HNSW entry count MUST be performed
