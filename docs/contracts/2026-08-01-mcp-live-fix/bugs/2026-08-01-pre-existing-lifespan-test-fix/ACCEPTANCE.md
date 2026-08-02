# ACCEPTANCE — Pre-Existing Lifespan Test Fix

## AC-LS-001
GIVEN the full test suite
WHEN it runs from clean state
THEN ALL tests pass (0 failures; previously 1 failure)

## AC-LS-002
GIVEN a repeat run (2x consecutive)
THEN the lifespan test passes both times (no flake)

## AC-LS-003
GIVEN the fix
THEN no production code changed (only test/infra files)
