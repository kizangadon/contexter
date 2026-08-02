# DESIGN PREVIEW — Handler Limit Passthrough

## Approach

```mermaid
flowchart LR
    A[MCP client: list_recent_sessions limit=5] --> B[handle_list_recent_sessions]
    B -->|limit after clamp: 5| C[session_service.list filter, limit=5]
    C -->|limit pushdown| D[StorageEngine bridge]
    D --> C2[engine returns <=5 sessions]
    C2 --> E[response exactly 5 - no re-slice]
```

- Clamping lives in the handler (mirrors service rule): negative/zero → 0, huge → 10_000, None → leave None (service default 100).
- The service and engine already honor the limit; this contract only wires the handler.
- Tests: handler-level spy asserting the limit argument passed to the service; no re-slice assertion.
