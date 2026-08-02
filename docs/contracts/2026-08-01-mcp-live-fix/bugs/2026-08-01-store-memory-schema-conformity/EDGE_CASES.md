# EDGE_CASES — store_memory Schema Conformity

| ID | Scenario | Expected |
|---|---|---|
| EC-SM-001 | Call without `content` | Missing required param error (unchanged) |
| EC-SM-002 | Call with legacy extra params | Frozen contract behavior |
| EC-SM-003 | Registered schema vs handler signature mismatch | Zero drift; test locks it |
