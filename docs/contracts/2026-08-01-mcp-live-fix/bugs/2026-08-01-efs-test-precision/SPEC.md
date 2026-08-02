# SPEC — EFS Test Module Precision

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Findings: Code Reviewer F-3 (NIT), F-4 (INFO), User-Testing LOW (evidence mismatch — `failure_specific_bytes=-262`)

## Problem

Three precision defects in the EFS framework-level test module `contexter-server/tests/mcp/test_framework_efs_stderr.py`:

1. **F-3 (NIT):** `test_concurrent_failures_each_bounded:297` asserts `len(stderr) <= n * _STDERR_LIMIT` — redundant, since the preceding per-failure `_assert_bounded` (≤512) already implies it.
2. **F-4 (INFO):** The module docstring claims capfd observes "bridge line + any framework output"; empirically it observes **framework-only** in-process — pytest's root `LogCaptureHandler` captures bridge records, so bridge ERROR lines never reach fd-2; `lastResort` never fires. The docstring misdescribes what is measured (tests remain fully discriminating).
3. **UT LOW (evidence mismatch):** The worker's own evidence artifact (`iter3-harness-out.json`) reported `failure_specific_bytes=-262` for FL001_engine — internally inconsistent with the ≤512-byte assertions and the validator's direct measurement (326B). The test harness's byte-computation must be self-consistent (non-negative, matching what the assertions measure).

## Requirements

### REQ-EP-001 — Remove redundant assertion
`test_concurrent_failures_each_bounded` SHALL drop the redundant `n * _STDERR_LIMIT` assertion (or replace it with a meaningful assertion, e.g., total across concurrent failures stays bounded as n×512 — if kept, it must add information).

### REQ-EP-002 — Correct docstring
The module docstring SHALL accurately describe what capfd observes under pytest: framework-level stderr in-process (bridge records captured by pytest's LogCaptureHandler; the ≤512-byte assertions measure the framework contribution), while noting the live end-to-end path (bridge line + framework) is covered by validator live probes and/or a subprocess-level test if one exists.

### REQ-EP-003 — Self-consistent evidence computation
The harness/test SHALL compute `failure_specific_bytes` (or equivalent) so it is always non-negative and consistent with the byte assertions — no negative or internally contradictory evidence values.

## Non-Goals

- No change to the filter implementation (covered by `fastmcp-filter-coverage` contract).
- No weakening of discriminating assertions (≤512 bytes, 0 box chars, 0 traceback).

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-efs-test-precision/`
- References: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-3.md` (F-3, F-4), `...-user-testing-review-iter-3.md` (LOW evidence mismatch)
