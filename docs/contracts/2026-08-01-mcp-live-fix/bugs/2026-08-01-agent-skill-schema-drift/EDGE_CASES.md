# EDGE_CASES — Agent/Skill Schema Drift Repair

| ID | Scenario | Expected |
|---|---|---|
| EC-AG-001 | Agent with engine fields absent in Python model (provider/model) | Translation/mapping resolves; no ValidationError |
| EC-AG-002 | Agent create payload missing engine-required `type`/`description` | Service enriches payload; persists |
| EC-AG-003 | Nonexistent agent id | Structured isError, process alive |
| EC-AG-004 | Agent with `version` as int | Harmonized; no type error |
| EC-SK-001 | Skill with `version` int vs `Optional[str]` | Harmonized (int or str coercion) |
| EC-SK-002 | `list_skills(type=...)` with no matching skills | Empty results, success |
| EC-SK-003 | `list_skills(type=unknown)` | Empty results or structured error — never traceback |
| EC-SK-004 | `category` present in engine, absent in Python model | Model accepts/preserves |
| EC-RS-001 | Agent resource with `_api_key` | Real agent JSON |
| EC-RS-002 | Agent resource without `_api_key` (key set) | Auth rejection preserved |
