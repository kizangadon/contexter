# Bug: Missing Replacement Test for 0o700 Permission Behavior

**Severity:** LOW  
**Root Cause:** `test_read_only_path_error` was removed and replaced with `test_writable_path_succeeds`, but no test verifies that the 0o700 permission is actually applied.

## Requirements

### REQ-FIX-001: Add test verifying 0o700 permissions
Add a test in `tests/storage/rocksdb_test.rs` that:
1. Creates a temp directory
2. Opens an Engine at that path
3. Verifies the Engine's storage directory (or the TempDirGuard dir) has `0o700` permissions (on Unix)
