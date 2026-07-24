# Bug: Test structure does not mirror source structure

## Problem
The SPEC (REQ-TST-001 through REQ-TST-007) and design preview Section 13 require test files to mirror source structure:
```
tests/storage/rocksdb_test.rs
tests/cache/lru_test.rs
tests/compression/codecs_test.rs
tests/engine/session_test.rs
tests/engine/memory_test.rs
tests/bridges/pyo3_test.rs
tests/common/mod.rs  (with TempRocksDb::new(), sample data generators)
```

Currently, only a monolithic `tests/integration_test.rs` (1,086 lines, 13 tests) exists. All 6 test subdirectories are empty.

## Requirements
- REQ-001: Create `tests/common/mod.rs` with `TempRocksDb::new()` helper and sample data generators (extracted from integration_test.rs)
- REQ-002: Split integration_test.rs tests into per-module test files: `tests/storage/rocksdb_test.rs`, `tests/cache/lru_test.rs`, `tests/compression/codecs_test.rs`, `tests/engine/session_test.rs`, `tests/engine/memory_test.rs`
- REQ-003: Create `tests/bridges/pyo3_test.rs` with bridge integration tests
- REQ-004: Remove the monolithic `tests/integration_test.rs` after splitting
- REQ-005: All 13 integration tests MUST pass after restructuring
- REQ-006: `cargo test` must produce the same or higher test count
