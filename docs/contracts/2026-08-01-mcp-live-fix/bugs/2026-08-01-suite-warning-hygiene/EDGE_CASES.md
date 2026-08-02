# EDGE CASES — Suite Warning Hygiene

## EC-SW-001 — Identify the exact source first
Verify the warning stack (starlette → python-multipart `DeprecationWarning`/`PendingDeprecationWarning`) before choosing filter vs upgrade. If a small dependency bump removes it cleanly, prefer that (no filter debt).

## EC-SW-002 — Narrow matching
If filtering: scope by module (`python_multipart`/`multipart`), warning class, and optionally message regex — never `ignore::Warning` or `ignore:.*`.

## EC-SW-003 — No test-content churn
Do not silence by editing tests to catch warnings; keep test content stable.

## EC-SW-004 — CI parity
The filterwarnings change lives in the same config CI uses (`pyproject.toml`), so local and CI runs behave identically.
