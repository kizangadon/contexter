# ACCEPTANCE — Engine-Open Failure Handling

## AC-LH-001
GIVEN a locked/unwritable data dir
WHEN the MCP server launches
THEN client-visible output contains a clean structured error and NO raw Python/Rust traceback

## AC-LH-002
GIVEN the same failure
THEN full diagnostic detail appears in server logs

## AC-LH-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; launch-failure test present
