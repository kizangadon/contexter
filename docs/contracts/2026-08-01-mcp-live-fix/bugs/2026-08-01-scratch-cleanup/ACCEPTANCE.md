# ACCEPTANCE — Scratch File Cleanup

## AC-SC-001
GIVEN the repo
WHEN listing `docs/tests/` and `contexter-server/docs/tests/`
THEN no scratch files remain (empty or absent)

## AC-SC-002
GIVEN the cleanup
THEN full suite ≥647 passed / 1 known pre-existing

## AC-SC-003
GIVEN a git status check
THEN no new files are staged/committed from docs/tests/
