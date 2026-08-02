# Preview — Handler Observability Logs

## Approach
```mermaid
flowchart LR
  A[Handler call] --> B[log: call received + corr id]
  B --> C[log: auth decision]
  C --> D[log: engine result + duration]
  D --> E[log: error path]
```
Structured handler logs with correlation id; no content/secrets (B9 bounds apply).

## Fix boundary
Handlers + logging helper + caplog tests.

## Acceptance mapping
AC-HO-001..003, EC-HO-001..004.
