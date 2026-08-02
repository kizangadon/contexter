# EDGE CASES — EFS Test Module Precision

## EC-EP-001 — Keep discriminating assertions
Removing redundancy MUST NOT remove the per-failure ≤512-byte and 0-box assertions — they are the contract's teeth.

## EC-EP-002 — Docstring vs subprocess tests
If any subprocess-level EFS test exists (real stderr), the docstring should distinguish it from in-process capfd scope.

## EC-EP-003 — Negative bytes impossible
The computation SHALL be structured so negative values are impossible (e.g., baseline measured before failure, delta computed as max(0, after - before) with the failure section isolated) — and a test asserts non-negativity if feasible.

## EC-EP-004 — No scope creep
This contract touches ONLY `test_framework_efs_stderr.py` (+ optional harness scripts under `/tmp/opencode`, deleted after use). No filter changes.
