# Bug: Test Flakiness — PID-Based Temp Dir Path Collision

**Severity:** MEDIUM  
**Root Cause:** `TempDirGuard` uses `/tmp/contexter_duckdb_{PID}` for its temp directory — PID-based paths collide across parallel test threads, causing `remove_dir_all` in one thread to interfere with another test.

## Requirements

### REQ-FIX-001: UUID-based temp dir
Replace PID-based temp directory naming in `TempDirGuard::new()` with UUID-based naming (e.g., `/tmp/contexter_duckdb_{UUID}`).
