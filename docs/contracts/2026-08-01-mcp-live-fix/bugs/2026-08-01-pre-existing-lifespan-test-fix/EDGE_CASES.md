# EDGE_CASES — Pre-Existing Lifespan Test Fix

| ID | Scenario | Expected |
|---|---|---|
| EC-LS-001 | Two test engines same dir | Never happens (unique dirs) |
| EC-LS-002 | Shutdown before startup complete | Clean join/no hang |
| EC-LS-003 | Double shutdown | Idempotent, no error |
