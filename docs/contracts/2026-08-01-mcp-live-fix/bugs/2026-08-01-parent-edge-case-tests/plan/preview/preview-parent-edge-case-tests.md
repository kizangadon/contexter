# DESIGN PREVIEW — Parent Edge-Case Test Coverage
```mermaid
flowchart LR
    A[EC-015] --> T[new tests]
    B[EC-017/018] --> T
    C[EC-021] --> T
    T --> F[full suite green]
```
- Test-only contract (implementation fixes only if docs/behavior conflict).
