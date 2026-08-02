# SPEC — Handler Limit Passthrough (B6 completion)

## Context
Bug contract B6 (session-limit-pushdown) moved the limit enforcement into `session_service.list()`,
but the user-facing path is incomplete: `handle_list_recent_sessions` in `handlers.py` calls
`session_service.list(filter=...)` without passing `limit`, then slices in Python. The engine
still fetches 100 sessions per call (Performance finding PF-01 half-fixed).

## Requirements

- REQ-HLP-001: `handle_list_recent_sessions` SHALL pass the incoming `limit` (after the same
  clamping rules: negative → 0, 0 → 0, > MAX_SESSION_LIST_LIMIT → MAX_SESSION_LIST_LIMIT,
  None → engine default 100) into `session_service.list()` so the engine honors the limit.
- REQ-HLP-002: The handler SHALL NOT re-slice after the service call; the service result is
  authoritative (exactly N sessions, most recent first).
- REQ-HLP-003: Existing handler behavior for absent/invalid limit values SHALL be preserved
  (default limit = engine default 100).
- REQ-HLP-004: No change to the MCP tool signature or the `SessionFilter` shape.
- REQ-HLP-005: New tests SHALL prove the engine call receives the clamped limit (spy/mock at
  the service boundary or handler level) for: explicit limit=5, limit=None (default), negative,
  zero, huge (clamped to MAX_SESSION_LIST_LIMIT).

## Out of scope
- No changes to `session_service.py` (already correct).
- No changes to bridge or engine.
- No changes to other handlers.
