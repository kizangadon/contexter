# Preview — Session Limit Pushdown

## Approach
```mermaid
flowchart LR
  A[list_recent_sessions limit] --> B[Clamp 0..max]
  B --> C[Engine call with limit\nhonored at boundary]
  C --> D[Slice-safe result\nmost-recent-first]
```
Push limit into the engine call; keep ordering; clamp edges; spy test proves engine receives the limit.

## Fix boundary
`services/session_service.py` (or handler) + TDD spy test.

## Acceptance mapping
AC-SL-001..003, EC-SL-001..005.
