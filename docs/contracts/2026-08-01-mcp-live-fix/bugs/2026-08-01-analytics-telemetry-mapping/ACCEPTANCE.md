# ACCEPTANCE — Analytics/Health Telemetry Mapping Repair

## AC-AN-001
GIVEN an engine seeded with ≥1 session, ≥2 memories, ≥1 agent, ≥1 skill
WHEN `contexter://analytics/overview?_api_key=...` is read
THEN total_sessions/total_memories/total_agents/total_skills reflect real counts (non-zero)

## AC-AN-002
GIVEN the same engine
WHEN `get_system_health(_api_key=...)` is called
THEN storage/ops fields are populated from real telemetry (not structurally zero)

## AC-AN-003
GIVEN the mapping fix
THEN full suite ≥647 passed / 1 known pre-existing failure; new analytics tests (RED→GREEN)
