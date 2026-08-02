# ACCEPTANCE — Input Validation Gaps Repair

## AC-IV-001
GIVEN `store_memory` with `content=""` (or whitespace)
THEN a structured error is returned and nothing persists

## AC-IV-002
GIVEN `export_data(format="xml")` (unsupported)
THEN a structured error is returned (not `completed`)

## AC-IV-003
GIVEN `list_recent_sessions(limit=-5)` / `limit=0` / `limit=100000`
THEN limit is clamped to sane bounds; call succeeds without crash

## AC-IV-004
GIVEN oversized content/query
THEN structured error (no unbounded echo)

## AC-IV-005
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; new validation tests (RED→GREEN)
