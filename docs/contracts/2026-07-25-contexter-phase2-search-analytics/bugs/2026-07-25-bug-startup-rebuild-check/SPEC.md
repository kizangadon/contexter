# Bug: Missing L2 Memory Count vs HNSW Entry Count Verification at Startup

**Severity:** LOW  
**Root Cause:** `Engine::with_config()` does not verify that the L2 storage memory count matches the HNSW vector index entry count on startup. If they drift (e.g., from a partial snapshot restore), the system silently operates with inconsistent state.

## Requirements

### REQ-FIX-001: Add startup consistency check
In `Engine::with_config()`, after loading the vector index snapshot, compare L2 storage memory count with HNSW entry count. Log a warning if they differ (but do not fail — the inconsistency may be expected, e.g., during migration).
