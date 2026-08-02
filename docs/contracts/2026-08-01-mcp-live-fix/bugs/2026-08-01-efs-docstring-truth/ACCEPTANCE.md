# ACCEPTANCE — EFS Test Docstring Accuracy

## AC-DG-001 — Accurate drop-policy statement (docstring)
- **Given** `contexter-server/tests/mcp/test_framework_efs_coverage.py`
- **When** the module docstring is read
- **Then** it states that covered framework records are dropped at all levels (including below-WARNING), matching `test_covered_records_below_warning_dropped`

## AC-DG-002 — Requirement references correct (entire file)
- **Given** the entire `test_framework_efs_coverage.py` file
- **When** its requirement IDs are checked (docstring AND inline section comments)
- **Then** it references only real contract IDs (`REQ-FC-*`, `REQ-FL-*`) — zero `REQ-FF-*` (or other fabricated) IDs remain anywhere in the file

## AC-DG-003 — No behavior change
- **Given** the full Python suite
- **Then** `cd contexter-server && python3 -m pytest -q` passes 904 + tests, 0 failed, 0 warnings — filter behavior and all tests unchanged

## AC-DG-004 — Minimal diff
- **Given** git diff
- **Then** the change touches only `test_framework_efs_coverage.py` (docstring + inline comment citations) — no production files, no other tests