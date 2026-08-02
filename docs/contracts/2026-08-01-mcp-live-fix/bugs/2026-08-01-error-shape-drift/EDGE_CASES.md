# EDGE_CASES — Error-Shape Drift Repair

| ID | Scenario | Expected |
|---|---|---|
| EC-ES-001 | Nonexistent session id | Structured isError: `Resource not found: <id>` |
| EC-ES-002 | Nonexistent memory/agent id | Same convention |
| EC-ES-003 | Missing required param | Structured validation error |
| EC-ES-004 | Engine failure mid-call | Structured error; next call works |
| EC-ES-005 | Wrong `_api_key` | Auth error shape unchanged |
| EC-ES-006 | Error then success call sequence | No state corruption; frames intact |
