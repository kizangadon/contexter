# ACCEPTANCE — EFS Test Module Precision

## AC-EP-001 — No redundant assertion
- **Given** `test_framework_efs_stderr.py`
- **When** `test_concurrent_failures_each_bounded` is inspected
- **Then** it contains no redundant `len(stderr) <= n * _STDERR_LIMIT` assertion (or the kept assertion demonstrably adds information)

## AC-EP-002 — Docstring accurate
- **Given** the module docstring
- **When** it is read
- **Then** it accurately describes the in-process capfd observation model (framework-only under pytest; bridge line covered live end-to-end) — no claim that capfd captures the bridge line

## AC-EP-003 — Evidence computation consistent
- **Given** the harness/evidence computation
- **When** failures are measured
- **Then** reported `failure_specific_bytes` (or equivalent) is always non-negative and matches the assertion semantics (≤512 per failure)

## AC-EP-004 — Suite green
- **Given** the full suite
- **Then** `python -m pytest -q` shows 881 + any new tests passed, 0 failures; the 13 EFS tests remain green and discriminating (≤512, 0 boxes)
