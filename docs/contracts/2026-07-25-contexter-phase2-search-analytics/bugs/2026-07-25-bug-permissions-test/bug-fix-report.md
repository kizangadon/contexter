# Bug Fix Report: Permissions Test

**Bug:** Missing replacement test for 0o700 permission behavior

## Changes Made

### `contexter-core/tests/storage/rocksdb_test.rs`

Added new test `test_engine_dir_has_0700_permissions`:
- `#[cfg(unix)]` — only runs on Unix
- Creates a `TempDir`
- Opens an `Engine` at that path
- Drops the engine (flush + close)
- Checks `std::fs::metadata().permissions().mode() & 0o777 == 0o700`
- Uses local `use std::os::unix::fs::PermissionsExt` import

## Verification
- `cargo build --workspace` — passes
- `cargo test --workspace` — all tests pass
- `test_engine_dir_has_0700_permissions` shows `ok` in test output

## Status
✅ FIXED
