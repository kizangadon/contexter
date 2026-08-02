# ACCEPTANCE — Env Var Canonicalization

## AC-EV-001
GIVEN `CONTEXTER_BRIDGE_POOL_SIZE=4`
WHEN the bridge starts
THEN the thread pool size honors the canonical var (test evidence)

## AC-EV-002
GIVEN a repo-wide grep
THEN no production code reads a `CONtexTER_` (misspelled) variable

## AC-EV-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; canonical-var test present
