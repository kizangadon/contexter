# EDGE_CASES — Session Limit Pushdown

| ID | Scenario | Expected |
|---|---|---|
| EC-SL-001 | 0 sessions | Empty result, success |
| EC-SL-002 | limit=1 | 1 session, most recent |
| EC-SL-003 | limit=0 | Clamped/valid per contract |
| EC-SL-004 | limit negative | Clamped to ≥0 |
| EC-SL-005 | limit huge | Clamped to max |
