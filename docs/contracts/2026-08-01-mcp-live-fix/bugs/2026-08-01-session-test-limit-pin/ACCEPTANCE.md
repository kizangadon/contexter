# ACCEPTANCE — Session Concurrent Test: Pin Explicit Limit

## AC-SL-001 — Explicit limit in filter
- **Given** `contexter-core/tests/engine/session_test.rs`
- **When** `test_concurrent_operations` constructs its `SessionFilter`
- **Then** it sets an explicit `limit` larger than the test's row count (e.g., `u64::MAX`), not relying on `SessionFilter::default()`

## AC-SL-002 — Test intent intact
- **Given** the test's assertion
- **When** it runs
- **Then** it still asserts all 100 concurrent writes are present (no lost writes)

## AC-SL-003 — Suite green
- **Given** the full Rust suite
- **Then** `cargo test` passes (469 + any changes, 0 failed); full Python suite stays 881 passed / 0 failures
