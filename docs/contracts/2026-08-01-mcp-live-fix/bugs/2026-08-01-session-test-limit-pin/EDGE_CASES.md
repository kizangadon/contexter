# EDGE CASES — Session Concurrent Test: Pin Explicit Limit

## EC-SL-001 — Keep concurrency semantics
The explicit limit MUST NOT change what the test exercises (concurrent writes visible after join).

## EC-SL-002 — No magic numbers
If a limit value other than `u64::MAX` is used, it MUST be a named constant or clearly commented (> row count).

## EC-SL-003 — Minimal diff
Single-line (or near-single-line) change; no reformatting or unrelated edits in session_test.rs.
