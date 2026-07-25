# Bug 10 Design Preview — Missing Tests

## Files to create
1. `tests/common/fixtures.rs`: shared constants + helpers
2. `tests/storage/column_families_test.rs`: CF integration tests
3. `tests/engine/search_test.rs`: search integration tests

## Files to modify
4. `tests/common/mod.rs`: add `pub mod fixtures;`
