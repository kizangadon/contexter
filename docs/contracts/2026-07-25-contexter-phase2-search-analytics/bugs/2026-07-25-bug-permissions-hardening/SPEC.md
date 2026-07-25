# Bug: File Permission Hardening (TempDir, Tantivy, Snapshots)

**Severity:** HIGH  
**Root Cause:** Missing restrictive permissions on temp directories and data directories

## Requirements

### REQ-FIX-001: TempDirGuard 0o700
Add `std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))` in `TempDirGuard::new()` at `analytics/duckdb.rs` immediately after `create_dir_all()`.

### REQ-FIX-002: Tantivy index directory 0o700
Apply `set_permissions(0o700)` to the Tantivy index directory in `fts/tantivy.rs` after directory creation.

### REQ-FIX-003: Snapshot file 0o600
Apply `set_permissions(0o600)` to the snapshot output file in `vector/snapshot.rs::save_snapshot_data()`.

### REQ-FIX-004: Update test_read_only_path_error
Fix `test_read_only_path_error` in `tests/storage/rocksdb_test.rs` to account for the new `0o700` permission behavior (the test expected an error on read-only dir but 0o700 makes it writable first).
