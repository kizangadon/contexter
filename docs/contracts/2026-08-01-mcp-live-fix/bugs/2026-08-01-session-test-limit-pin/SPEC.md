# SPEC — Session Concurrent Test: Pin Explicit Limit

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Finding: **Code Reviewer F-2 (NIT)** — `review-mcp-live-fix-scrutiny-code-review-iter-3.md`

## Problem

`contexter-core/tests/engine/session_test.rs` `test_concurrent_operations` asserts exactly 100 sessions via `SessionFilter::default()` (whose `limit` defaults to 100). The exactness of the assertion silently depends on the default limit — if the test ever exceeds 100 rows (e.g., the loop count is raised), the assertion breaks for the wrong reason.

## Requirements

### REQ-SL-001 — Explicit limit
`test_concurrent_operations` SHALL construct its `SessionFilter` with an explicit `limit` (e.g., `u64::MAX` or a value > the test's row count) so the exact-count assertion tests concurrency semantics, not the default limit.

### REQ-SL-002 — Intent preserved
The test SHALL continue to assert that all concurrent writes are visible (no lost writes), exactly as intended — only the limit source changes.

## Non-Goals

- No change to production code.
- No change to other tests.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-session-test-limit-pin/`
- References: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-3.md` (F-2)
