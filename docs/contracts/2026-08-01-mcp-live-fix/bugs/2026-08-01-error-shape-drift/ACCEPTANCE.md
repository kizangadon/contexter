# ACCEPTANCE — Error-Shape Drift Repair

## AC-ES-001
GIVEN a call to get_session with a nonexistent id over live stdio
THEN the response is a structured MCP error (isError=True), message contains `Resource not found: <id>`, and no success result is returned

## AC-ES-002
GIVEN an invalid parameter call (e.g., search_memories without query)
THEN a structured validation error is returned (never a traceback, never a success frame)

## AC-ES-003
GIVEN an auth failure
THEN MCPAuthError serialization is unchanged (same message, isError behavior)

## AC-ES-004
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing failure; new error-shape tests present (RED→GREEN)

## AC-ES-005
GIVEN repeated error calls
THEN server process survives and stdout carries only JSON-RPC frames
