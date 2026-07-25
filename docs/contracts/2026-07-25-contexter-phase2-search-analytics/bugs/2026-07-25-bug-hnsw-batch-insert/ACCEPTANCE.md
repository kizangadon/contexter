# Acceptance Criteria

### AC-01: Batch insert builds graph once
GIVEN N embeddings are inserted via `insert_batch()`  
WHEN the graph is built  
THEN the underlying HNSW graph MUST be built once (not N times)

### AC-02: Snapshot load uses batch
GIVEN a snapshot with N embeddings  
WHEN `load_snapshot()` loads it  
THEN the insert should use `insert_batch()` or equivalent optimization

### AC-03: Backward compatible
GIVEN existing code that calls single `insert()`  
WHEN compiled  
THEN it MUST still work with no API changes
