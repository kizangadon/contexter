# Bug 10: Missing Test Infrastructure

## REQ-TST-001: Create tests/common/fixtures.rs
Create a test fixtures module in `tests/common/fixtures.rs` that provides shared test constants and helper functions:
- `TEST_PROJECT: &str` = "test-project"
- `TEST_AGENT_ID: Uuid` = Uuid::nil() or a well-known UUID
- Helper function `setup_engine() -> (Engine, TempDir)`
- Helper function `setup_rocksdb() -> (RocksDbBackend, TempDir)`

Update `tests/common/mod.rs` to declare `pub mod fixtures;`.

## REQ-TST-002: Create tests/storage/column_families_test.rs
Create an integration test file in `tests/storage/column_families_test.rs` that tests:
- All 9 CF names resolve via `cf_handle`
- `ColumnFamilyMap::new()` returns all 9 CFs
- Key encoding functions produce correct prefixed keys

## REQ-TST-003: Create tests/engine/search_test.rs
Create an integration test file in `tests/engine/search_test.rs` that tests:
- Memory search by session_id, memory_type, tags, keywords
- Multi-keyword scoring
- Pagination (offset + limit)
- No-match returns empty results
