# Acceptance Criteria — Bug 17

## AC-B17-001: Boundary test exists
- Given: The test module in `src/python.rs`
- When: Tests are run
- Then: A test `test_json_depth_exceeds_limit` exercises 65+ nesting and asserts rejection

## AC-B17-002: Safety comments on 6 call sites
- Given: `src/python.rs` at lines 708, 798, 918, 1053, 1088, 1160
- When: Each call site is inspected
- Then: A `// SAFETY:` comment explains why depth checking is unnecessary

## AC-B17-003: All tests pass
- Given: The codebase
- When: `cargo test` is run
- Then: All 194+ tests pass
