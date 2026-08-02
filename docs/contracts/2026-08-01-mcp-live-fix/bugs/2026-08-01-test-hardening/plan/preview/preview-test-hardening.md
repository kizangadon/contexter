# Preview — Test Hardening

## Approach
```mermaid
flowchart TD
  A[pytest.raises(Exception)] --> B[precise exception types]
  C[missing edge tests] --> D[empty-engine, empty-content,\nlimit edges, launch failure]
```
Replace broad raises with precise types; add missing edge tests.

## Fix boundary
Existing test files + new edge tests.

## Acceptance mapping
AC-TH-001..003, EC-TH-001..003.
