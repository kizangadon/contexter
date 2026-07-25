# Bug: Error Handling — Silencing, Coverage, Poisoning, Cleanup

**Severity:** HIGH  
**Root Cause:** Several `unwrap()` calls silence errors; error types incomplete; no Mutex poisoning recovery; no temp file cleanup.

## Requirements

### REQ-FIX-001: Replace bare unwrap() in engine code with proper error propagation
Find and replace `unwrap()` calls in `engine/search.rs`, `engine/analytics.rs`, `engine/memory.rs`, `engine/mod.rs` with `?` or `.map_err()` propagation. Exceptions: test code and infallible operations.

### REQ-FIX-002: Add UnsupportedOperation error variant
Add `UnsupportedOperation(String)` to `EngineError` for methods that are called when the corresponding tier is disabled.

### REQ-FIX-003: Add Mutex poisoning recovery
Wrap `LockResult` poison errors in engine's Mutex accesses. Use `.lock().map_err(|e| ...)` or `.lock().unwrap_or_else(|e| e.into_inner())` to recover from poisoning.

### REQ-FIX-004: Add temp file cleanup guard
Add a `TempDir` cleanup guard or drop handler that cleans up temporary directories used during analytics sync.
