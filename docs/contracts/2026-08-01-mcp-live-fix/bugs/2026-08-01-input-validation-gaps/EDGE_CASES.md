# EDGE_CASES — Input Validation Gaps Repair

| ID | Scenario | Expected |
|---|---|---|
| EC-IV-001 | `content=""` | Structured error |
| EC-IV-002 | `content="   "` | Structured error (whitespace-only) |
| EC-IV-003 | `format="xml"` | Structured error |
| EC-IV-004 | `format="json"` | Success (supported) |
| EC-IV-005 | `limit=0` | Clamped/valid (per contract intent) |
| EC-IV-006 | `limit=-1` | Clamped to ≥0 |
| EC-IV-007 | `limit=10**9` | Clamped to max |
| EC-IV-008 | Content > cap | Structured error, no unbounded echo |
| EC-IV-009 | Query > cap | Structured error, no unbounded echo |
