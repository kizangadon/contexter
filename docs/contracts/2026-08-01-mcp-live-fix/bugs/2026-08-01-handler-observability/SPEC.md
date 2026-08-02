# SPEC — Handler Observability Logs (CON-003 gap)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-handler-observability

## Problem
Security INFO: missing CON-003 logs — handler-level structured logs (tool call, auth decision, engine result summary, error) are not consistently emitted; observability is a core principle and CON-003 expected server-side logs.

## Requirements
- REQ-HO-001: Every handler emits structured logs at meaningful points: call received (tool, session id, correlation id), auth decision (allow/reject, no secrets), engine result (success/error, duration), and error path.
- REQ-HO-002: Logs contain no content payloads or secrets (bounds from B9 apply).
- REQ-HO-003: A correlation id flows from request to log lines where feasible.
- REQ-HO-004: TDD: test asserting handler logs are emitted (caplog) for success and error paths; full suite green.

## Constraints
Auth unchanged. DDD applies. Observability must not regress auth hygiene.
