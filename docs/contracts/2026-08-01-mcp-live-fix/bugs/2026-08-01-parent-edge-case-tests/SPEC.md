# SPEC — Parent Edge-Case Test Coverage (iter-1 finding SPEC-2)

## Context
SPEC Compliance Validator: parent EDGE_CASES.md items EC-015, EC-017, EC-018, EC-021 remain
untested (P2/P3). Add tests covering them.

## Requirements
- REQ-PEC-001: Read parent EDGE_CASES.md (/home/don/Code/contexter/docs/contracts/2026-08-01-mcp-live-fix/EDGE_CASES.md) and identify EC-015, EC-017, EC-018, EC-021.
- REQ-PEC-002: Write tests covering each identified edge case (in the appropriate existing test
  file, following project patterns).
- REQ-PEC-003: Each test must assert the documented behavior — if the documented behavior
  conflicts with implementation, fix implementation to match docs (bug if found).
- REQ-PEC-004: Full suite passes.
