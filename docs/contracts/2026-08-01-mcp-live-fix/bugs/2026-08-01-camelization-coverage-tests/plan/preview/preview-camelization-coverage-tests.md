# Preview — Camelization Live-Coverage Tests

## Approach
```mermaid
flowchart LR
  A[All 34 engine methods] --> B[Live-engine harness]
  B --> C[response -> pydantic model parse]
  C --> D[34/34 verified or documented]
```
Systematic live-engine coverage of every bridge method; shape-locked mock tests for untestable ones with documented reasons.

## Fix boundary
New live coverage tests (tests/core/) + documentation of exceptions.

## Acceptance mapping
AC-CM-001..003, EC-CM-001..004.
