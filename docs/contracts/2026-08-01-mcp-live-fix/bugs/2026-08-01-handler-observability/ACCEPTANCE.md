# ACCEPTANCE — Handler Observability Logs

## AC-HO-001
GIVEN a store_memory call with valid auth
WHEN the handler runs
THEN structured log lines exist for: call received, auth decision, engine result (with duration), and no content/secret leakage

## AC-HO-002
GIVEN an error path (e.g., not found)
THEN a structured error log is emitted

## AC-HO-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; caplog tests present
