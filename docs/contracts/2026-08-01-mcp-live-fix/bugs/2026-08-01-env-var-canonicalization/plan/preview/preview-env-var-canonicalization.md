# Preview — Env Var Canonicalization

## Approach
```mermaid
flowchart TD
  A[CONTEXTER_BRIDGE_POOL_SIZE] --> B[Bridge pool size]
  C[CONtexTER_* typo] --> D[removed / explicit deprecated alias with log]
```
Canonical `CONTEXTER_*` only. Typo variant removed; if alias kept: explicit, logged, tested.

## Fix boundary
bridge.py:112 + any `CONtexTER_` reads + TDD canonical-var test + grep audit.

## Acceptance mapping
AC-EV-001..003, EC-EV-001..004.
