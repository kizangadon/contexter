# ACCEPTANCE — Handler Limit Passthrough

- AC-HLP-001: GIVEN a request with `limit=5`, WHEN `handle_list_recent_sessions` runs,
  THEN `session_service.list()` is called with limit=5 AND the engine call honors it
  (spy evidence in tests) AND the response contains exactly 5 sessions.
- AC-HLP-002: GIVEN no `limit` (None), WHEN the handler runs, THEN the service receives
  limit=None and the engine default (100) applies, AND no Python re-slice happens.
- AC-HLP-003: GIVEN `limit=-1` or `limit=0`, WHEN the handler runs, THEN the service receives
  the clamped value (0) — no negative/zero passthrough to the engine.
- AC-HLP-004: GIVEN `limit=10**9`, WHEN the handler runs, THEN the service receives
  MAX_SESSION_LIST_LIMIT (10_000) — no unbounded fetch.
- AC-HLP-005: GIVEN a service returning N sessions, WHEN the handler formats the response,
  THEN no additional truncation is applied.
