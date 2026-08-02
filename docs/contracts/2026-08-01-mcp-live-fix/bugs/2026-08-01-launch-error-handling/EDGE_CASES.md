# EDGE_CASES — Engine-Open Failure Handling

| ID | Scenario | Expected |
|---|---|---|
| EC-LH-001 | Data dir locked (concurrent process) | Clean structured error |
| EC-LH-002 | Data dir unwritable | Clean structured error |
| EC-LH-003 | Corrupt engine data | Clean structured error; logs have detail |
| EC-LH-004 | Normal launch | Unchanged behavior |
