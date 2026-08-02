# ACCEPTANCE — Agent/Skill Schema Drift Repair

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-agent-skill-schema-drift

## AC-AG-001
GIVEN a real engine with a seeded agent
WHEN `get_agent_info(id=...)` is called over live stdio with valid `_api_key`
THEN it returns real agent data (no pydantic ValidationError; `provider`/`model` resolved)

## AC-AG-002
GIVEN a real engine
WHEN `AgentService.create(...)` is invoked with a valid domain payload
THEN the agent persists in the engine (no `missing field 'type'/'description'` error)

## AC-AG-003
GIVEN a real engine with a seeded agent
WHEN resource `contexter://agent/{id}?_api_key=...` is read
THEN it returns the real agent JSON

## AC-SK-001
GIVEN a real engine with seeded skills
WHEN `list_skills(_api_key=...)` is called over live stdio
THEN it returns real skill data (no ValidationError; `category`/`version` harmonized)

## AC-SK-002
GIVEN a real engine with skills of known types
WHEN `list_skills(type=<known_type>, _api_key=...)` is called
THEN only matching skills are returned (filter applied, not silently dropped)

## AC-SK-003
GIVEN the fixed models/services
WHEN the full test suite runs
THEN ≥647 tests pass (1 known pre-existing failure only), including new RED→GREEN reproduction tests for agent/skill drift
