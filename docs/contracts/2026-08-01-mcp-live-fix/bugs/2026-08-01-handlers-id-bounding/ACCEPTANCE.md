# ACCEPTANCE — Handler ID Bounding
- AC-HIB-001: GIVEN a 1MB agent_id, WHEN not_found_error is raised, THEN the error message length ≤ 256 chars AND no raw 1MB id appears in it.
- AC-HIB-002: GIVEN a 1MB session_id, WHEN the handler logs the request, THEN the log line id field ≤ 64 chars.
- AC-HIB-003: GIVEN a normal 36-char UUID id, WHEN errors/logs are produced, THEN output is byte-identical to prior behavior.
- AC-HIB-004: GIVEN the mcp test suite, WHEN run, THEN all tests pass (no regressions in error-shape tests).
