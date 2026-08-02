# Preview — Bridge Log Hygiene

## Approach
```mermaid
flowchart LR
  A[args_summary] --> B[content-bearing args]
  B --> C[cap &lt;= 64 chars]
  C --> D[no full content, no secrets]
```
Bound content-bearing arg summaries at ≤64 chars; unit-test cap and no-leak; keep summaries useful.

## Fix boundary
`_truncated_args_summary` (bridge.py) + TDD cap tests.

## Acceptance mapping
AC-BH-001..003, EC-BH-001..004.
