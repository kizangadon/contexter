# ACCEPTANCE — Test Hardening

## AC-TH-001
GIVEN a repo-wide grep
THEN no test uses `pytest.raises(Exception)` (all precise exception types)

## AC-TH-002
GIVEN the test suite
THEN tests exist for empty-engine calls, empty content, limit edges, and launch failure

## AC-TH-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing
