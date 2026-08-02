# SPEC — Session Limit Pushdown (PF-01)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-session-limit-pushdown

## Problem
`list_recent_sessions` always fetches all 100 sessions from the engine, then slices in Python to `limit`. Wasteful; does not honor limit at the engine boundary (Performance finding PF-01).

## Requirements
- REQ-SL-001: `limit` is pushed down to the engine call (or an efficient partial fetch) so only the needed number of sessions is retrieved.
- REQ-SL-002: Result ordering (most-recent-first) preserved exactly.
- REQ-SL-003: `limit=0`/negative/huge handled per frozen contract intent (clamped; never crash).
- REQ-SL-004: TDD: test asserting engine receives the limited request (spy) and results match; full suite green.

## Constraints
Auth unchanged. DDD applies. Performance must not regress; no N+1.
