# SPEC — Agent/Skill Schema Drift Repair

**Parent:** 2026-08-01-mcp-live-fix · **Bug contract:** 2026-08-01-agent-skill-schema-drift
**Status:** FROZEN (Auto Bug Loop Iteration 1)

## Problem

Live MCP calls to `get_agent_info`, `list_skills`, and `contexter://agent/{id}` fail with pydantic ValidationError because Python entity models are out of sync with the Rust engine serde shapes:

- Engine `get_agent` returns `{id, name, type, description, capabilities, status, config, version, createdAt, updatedAt}` — Python `Agent` (`models/agent.py:10`) requires `provider` and `model` (missing).
- Engine `create_agent` rejects Python payloads: `missing field 'type'`, then `missing field 'description'` — `AgentService.create` (`services/agent_service.py:15`) sends `model_dump()` without them.
- Engine `Skill` returns `{id, name, description, category, version(int), filePath, createdAt, updatedAt}` — Python `SkillCreate` (`models/skill.py:24`) requires `type: str` and `version: Optional[str]`; `category` missing from Python model.
- Rust `SkillFilter` (`contexter-core/src/models/skill.rs:61-73`) silently drops the `type` filter — the `list_skills` `type` parameter is not applied.

## Requirements

- REQ-AG-001: `Agent`/`AgentCreate` models align with the engine serde contract (provider/model handling resolved — either added to engine serde, mapped from existing fields, or removed from required set) while preserving domain semantics.
- REQ-AG-002: `get_agent_info` returns a real agent over live stdio with all contract fields.
- REQ-AG-003: `AgentService.create` works against the real engine (payload includes engine-required `type`, `description`).
- REQ-SK-001: `Skill`/`SkillCreate` models align with engine serde (`category` present; `version` type harmonized int/str).
- REQ-SK-002: `list_skills` returns real skills over live stdio.
- REQ-SK-003: `list_skills` `type` filter is applied (engine-side or translation-side) — no silent drop.
- REQ-RS-001: `contexter://agent/{id}` resource resolves a real agent.
- REQ-TS-001: Translation layer (mirroring `memory_service.py` pattern) exists for agent/skill services OR models corrected at the source — with tests proving live parity.
- REQ-DD-001: Domain-Driven Design respected — entity semantics in domain models, translation at service boundary.

## Constraints

- Auth model unchanged. DDD applies. TDD: reproduce-tests-first (RED), then fix (GREEN). Full suite must stay ≥ 647 passing (1 known pre-existing failure untouched).
