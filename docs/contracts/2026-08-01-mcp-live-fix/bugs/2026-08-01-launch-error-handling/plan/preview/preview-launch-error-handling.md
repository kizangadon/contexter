# Preview — Launch Error Handling

## Approach
```mermaid
flowchart TD
  A[Engine open] --> B{success?}
  B -->|yes| C[Normal serving]
  B -->|no| D[Structured error to client]
  D --> E[Full diagnostics to server logs]
  E --> F[Clean exit or documented degraded mode]
```
Clean, structured client error on engine-open failure; raw detail only in logs; defined process behavior.

## Fix boundary
Launcher path (engine open error handler) + TDD launch-failure test (locked/unwritable dir).

## Acceptance mapping
AC-LH-001..003, EC-LH-001..004.
