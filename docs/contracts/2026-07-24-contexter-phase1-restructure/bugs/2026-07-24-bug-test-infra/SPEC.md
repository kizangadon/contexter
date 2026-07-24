# Bug 20: Missing Test Infrastructure

## REQ-TFI-001: Create tests/common/fixtures.rs
Move shared test fixtures from `tests/common/mod.rs` and engine inline tests into a proper `fixtures.rs` file that provides reusable test data factories (e.g., `create_sample_session`, `create_sample_memory`, `create_sample_agent`, `create_sample_skill`).

This file is imported by `tests/common/mod.rs` via `#[path = "fixtures.rs"] mod fixtures;`.

## REQ-TFI-002: Create tests/storage/column_families_test.rs
Integration tests that validate column family creation, naming, and iteration. Tests should:
- Verify all 12 CFs exist after engine open
- Verify CF_SETTINGS, CF_AUDIT, CF_SESSION_INDEX are properly created
- Verify ColumnFamilyMap returns all expected names via iter()

## REQ-TFI-003: Create tests/engine/search_test.rs
Integration tests for the search functionality. Tests should:
- Create memories with content and tags
- Search by content substring
- Search by memory_type filter
- Search by tags
- Verify search returns empty for non-matching queries
