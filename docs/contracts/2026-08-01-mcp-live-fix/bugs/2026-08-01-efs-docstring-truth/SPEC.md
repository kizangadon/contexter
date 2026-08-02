# SPEC — EFS Coverage Test: Docstring Accuracy

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 5 (found in iter-4)
> Finding: **Code Reviewer [LOW]** — `review-mcp-live-fix-scrutiny-code-review-iter-4.md`

## Problem

`contexter-server/tests/mcp/test_framework_efs_coverage.py` module docstring (lines 31–32) states records "below WARNING pass through". This is **self-contradictory** and contradicts the same file's `test_covered_records_below_warning_dropped` test and the implemented drop-at-every-level policy (`_SuppressFrameworkTracebackBox.filter()` returns `False` for all covered framework records at every level). The docstring also references non-existent requirement IDs `REQ-FF-002`/`REQ-FF-003`; the applicable contracts use `REQ-FC-*` (fastmcp-filter-coverage) and `REQ-FL-*` (fastmcp-framework-logging).

## Requirements

### REQ-DT-001 — Accurate drop policy
The module docstring SHALL accurately describe the filter's behavior: covered framework emitter records are dropped at EVERY level (including below-WARNING), not merely "below WARNING pass through".

### REQ-DT-002 — Correct requirement references
The document SHALL cite the real requirement IDs used by the contracts (`REQ-FC-*` and/or `REQ-FL-*`) — the fabricated `REQ-FF-*` IDs SHALL be removed.

### REQ-DT-003 — No behavior change
Comment/docstring-only change. No test logic, no filter logic, no other file changes.

## Non-Goals

- No change to filtering behavior, test assertions, or coverage.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-efs-docstring-truth/`
- Reference: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-4.md`