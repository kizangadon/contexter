# Bug: Snapshot read_string Robustness

**Severity:** MEDIUM  
**Root Cause:** `read_string()` in `snapshot.rs` lacks max-length guard on `u32` length prefix, allows OOM via crafted snapshot, and uses `from_utf8_lossy` which silently corrupts IDs

## Requirements

### REQ-FIX-001: Add max-length guard
In `read_string()` at `vector/snapshot.rs:113-121`, add a max-length check (e.g., 1024 bytes) on the `u32` length prefix before allocating the read buffer. Return error on oversized length.

### REQ-FIX-002: Use strict UTF-8
Replace `String::from_utf8_lossy(&buf)` with `String::from_utf8(buf).map_err(...)` so malformed snapshots produce an error instead of silently corrupting data.

### REQ-FIX-003: TOCTOU fix
Fix the TOCTOU window in `HnswVectorIndex::load_snapshot()`: open the file first, then check `metadata()` on the opened `File` handle instead of checking the path before opening.
