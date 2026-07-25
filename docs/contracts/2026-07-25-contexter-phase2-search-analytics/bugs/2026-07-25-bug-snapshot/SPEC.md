# Bug: Snapshot Lifecycle — Load-on-Boot, Save-on-Shutdown, Periodic Save

**Severity:** MEDIUM  
**Root Cause:** `Engine::with_config()` uses `HnswVectorIndex::load_or_new()` but only loads from disk; no explicit save-on-shutdown or periodic snapshot mechanism.

## Requirements

### REQ-FIX-001: Add explicit save() to HnswVectorIndex
Add `pub fn save(&self, path: &Path) -> Result<()>` that writes the current HNSW graph state to disk.

### REQ-FIX-002: Add periodic snapshot
Add `pub fn periodic_snapshot(&self, interval_secs: u64) -> JoinHandle<()>` that spawns a background task saving every N seconds. Accept a `CancellationToken` for clean shutdown.

### REQ-FIX-003: Wire snapshot in Engine drop or shutdown
Implement `Engine::shutdown()` that calls `save()` on the vector index and joins the snapshot task.
