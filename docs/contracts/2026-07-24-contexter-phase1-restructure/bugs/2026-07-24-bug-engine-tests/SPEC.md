# Bug 16: Engine Test Extraction

## REQ-ETX-001: Extract inline engine tests to integration test files
`engine/mod.rs` contains ~998 lines of inline `#[cfg(test)] mod tests { ... }` code (lines 208-1205). This creates excessive test density in the module file and makes it hard to navigate.

**Fix**: Extract all tests from `engine/mod.rs` into the existing `tests/engine/` directory:
- `tests/engine/session_test.rs` — session CRUD + cache + list/count tests (lines 267-449)
- `tests/engine/memory_test.rs` — memory CRUD + cache + content size limit tests (lines 455-536, 697-727, 1027-1101)
- `tests/engine/agent_skill_test.rs` — agent + skill CRUD + cache tests (lines 542-667)
- `tests/engine/settings_test.rs` — settings + audit tests (lines 674-834)
- `tests/engine/maintenance_test.rs` — flush/checkpoint/storage_size/cache_telemetry tests (lines 840-973)
- `tests/engine/error_test.rs` — not found + error path tests (lines 978-1022, 1103-1204)

Remove the inline `#[cfg(test)] mod tests { ... }` block from `engine/mod.rs` entirely.
