# ACCEPTANCE — Session Limit Pushdown

## AC-SL-001
GIVEN an engine with 100+ sessions
WHEN `list_recent_sessions(limit=5)` is called
THEN exactly 5 sessions (most recent first) are returned and the engine call honors the limit (spy/test evidence)

## AC-SL-002
GIVEN `limit=100000`
THEN clamped to max; success, no crash

## AC-SL-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; pushdown test present
