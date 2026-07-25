# Bug: Engine Drop — Snapshot Thread Zombie and Shutdown Not On Drop

**Severity:** HIGH  
**Root Cause:** `Engine` has `shutdown()` but no `Drop` impl — dropping an `Engine` without calling `shutdown()` leaves the snapshot thread running and may write to a closed RocksDB.

## Requirements

### REQ-FIX-001: Implement Drop for Engine
Add `impl Drop for Engine` that calls `shutdown()` to join the snapshot thread and save the vector index.

### REQ-FIX-002: Idempotent shutdown
`Engine::shutdown()` MUST be idempotent — calling it twice or calling it then dropping MUST NOT panic or produce UB. Use an `Option<JoinHandle>` + `take()` pattern.

### REQ-FIX-003: Verify thread join
The `Drop` impl MUST join the snapshot thread before returning from `shutdown()`.
