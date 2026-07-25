# Bug: HNSW Full Graph Rebuild on Every Insert

**Severity:** MEDIUM  
**Root Cause:** Every `insert()` call clones all embeddings and rebuilds the HNSW graph from scratch. This is O(n²) for batch inserts and limits scalability.

## Requirements

### REQ-FIX-001: Add batch_insert method
Add `pub fn insert_batch(&mut self, embeddings: &[(String, Vec<f32>)]) -> Result<()>` to `HnswVectorIndex` that builds the graph once for all embeddings instead of rebuilding per insert.

### REQ-FIX-002: Use batch in load_snapshot
Update `load_snapshot()` to use `insert_batch()` instead of calling `insert()` in a loop — avoids the full rebuild per item.

### REQ-FIX-003: Preserve existing single-insert API
Keep the existing `insert()` method working, but internally use batch optimizations where possible.
