# DESIGN PREVIEW — CLI Status Test Alignment

## Approach

```mermaid
flowchart LR
    A[test_status_format.py] -->|mock real shape| B[CLI status command]
    B -->|reads keys| C[real status payload: cacheTelemetry/status/version]
    C --> D{formatter diverges?}
    D -->|yes| E[fix formatter gracefully]
    D -->|no| F[test green]
```

- The real engine status shape is the contract; the test mock and the CLI formatter must both
  conform to it.
- TDD: failing test first with the real shape, then fix formatter if needed.
