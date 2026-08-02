# Preview — Scratch File Cleanup

## Approach
```mermaid
flowchart LR
  A[docs/tests/] --> B[remove all scratch files]
  C[contexter-server/docs/tests/] --> B
  B --> D[verify gitignored, suite green]
```
Delete leftover scratch files in both `docs/tests/` dirs; confirm nothing references them; suite green.

## Fix boundary
Filesystem cleanup only (Worker executes), no production code.

## Acceptance mapping
AC-SC-001..003, EC-SC-001..002.
