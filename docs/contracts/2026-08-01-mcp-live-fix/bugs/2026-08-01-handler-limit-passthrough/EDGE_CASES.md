# EDGE_CASES — Handler Limit Passthrough

- EC-HLP-001: limit=None → engine default (100) applied, no slice.
- EC-HLP-002: limit=0 → clamped to 0 → empty list returned, engine called with 0.
- EC-HLP-003: limit=-5 → clamped to 0 (matching service rule).
- EC-HLP-004: limit=10**9 → clamped to MAX_SESSION_LIST_LIMIT (10_000).
- EC-HLP-005: limit as non-numeric string → FastMCP validation rejects before handler
  (tool signature unchanged — verify no regression).
- EC-HLP-006: engine returns fewer than limit → response returns what engine returned.
- EC-HLP-007: engine error → handler error path unchanged (structured HandlerError).
