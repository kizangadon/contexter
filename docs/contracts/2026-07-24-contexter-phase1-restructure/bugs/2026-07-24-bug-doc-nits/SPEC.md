# Bug 24: Fix Doc Comment Nits in rocksdb.rs

## REQ-DOC-001: Fix duplicated doc comment for maybe_flush_wal
In `contexter-core/src/storage/rocksdb.rs`, lines 394-413, the doc comment for `maybe_flush_wal()` is duplicated — two identical blocks stacked consecutively. Remove the duplicate.

## REQ-DOC-002: Fix doc comment indentation
In `contexter-core/src/storage/rocksdb.rs`, lines 32-36, the doc comment for `RocksDbBackend` struct is indented 4 spaces (should start at column 0).
