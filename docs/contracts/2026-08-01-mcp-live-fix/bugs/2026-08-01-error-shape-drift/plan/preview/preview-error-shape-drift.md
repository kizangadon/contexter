# Preview — Error-Shape Drift Repair

## Approach
```mermaid
flowchart TD
  A[Handler] --> B{Success?}
  B -->|yes| C[result frame\nisError=False]
  B -->|no| D[structured error frame\nisError=True\nmessage convention]
  D --> E["Resource not found: <id>"]
```
Centralize error framing in a helper: not-found → `Resource not found: <id>`; validation → structured; engine failure → structured. Never return `{"error":...}` as success.

## Fix boundary
Handler error paths (get_session, get_memory, get_agent, etc.), shared error helper, + TDD tests.

## Acceptance mapping
AC-ES-001..005, EC-ES-001..006.
