# Handoff Report — Bug: pydantic-alias-annotated

**Date:** 2026-08-01
**Worker:** Distinguished Backend Engineer
**Status:** DONE — GREEN, all acceptance criteria verified

## Summary

Eliminated all 5 `UnsupportedFieldAttributeWarning` emissions from
`Field(validation_alias=AliasChoices(...))` in `agent.py` / `skill.py`
while keeping alias behavior byte-identical (REQ-PAA-001/002/003).

## Root Cause

The warning is a **FastAPI-induced false positive** (pydantic 2.13.4):

1. FastAPI `request_body_to_args` → `get_model_fields` (fastapi/_compat.py:285)
   wraps each model field as a standalone
   `TypeAdapter(Annotated[annotation, field_info])`.
2. Pydantic's `_apply_single_annotation` (pydantic/_internal/_generate_schema.py:2274)
   emits `UnsupportedFieldAttributeWarning` when a plain `FieldInfo` carrying
   field-specific metadata (`validation_alias`) is applied outside a model-field
   context.
3. The check is `type(metadata) is FieldInfo` — **subclasses are exempt**
   (pydantic's documented FastAPI escape hatch).

Key empirical facts (pydantic 2.13.4):

- `Annotated[T, AliasChoices(...)]` (bare metadata) — **silently drops the alias**
  (`model_fields[...].validation_alias is None`, validation fails). Contract's
  initial literal prescription was invalid on this version.
- `Annotated[T, Field(validation_alias=AliasChoices(...))]` — works for
  validation but the merge normalizes the FieldInfo to a plain `FieldInfo`,
  so FastAPI's standalone adapter still warns.
- **Working pattern:** assign a `FieldInfo` subclass directly:
  `capabilities: list[str] = AliasFieldInfo(validation_alias=AliasChoices("capabilities", "tools"), default_factory=list)`.
  `from_annotated_attribute` HACK 2 preserves the subclass instance in
  `model_fields`, and FastAPI's adapter then sees `type(metadata) is not FieldInfo`
  → no warning.

## Changes (2 files, no commits)

### `src/contexter_server/models/agent.py`

- Added `AliasFieldInfo(FieldInfo)` class + `from pydantic.fields import FieldInfo`.
- `Agent.capabilities`, `AgentCreate.capabilities`:
  `AliasFieldInfo(validation_alias=AliasChoices("capabilities", "tools"), default_factory=list)`.
- `AgentPatch.capabilities`:
  `AliasFieldInfo(validation_alias=AliasChoices("capabilities", "tools"), default=None)`.
- `Agent.created_at`/`updated_at`:
  `AliasFieldInfo(validation_alias=AliasChoices("created_at", "createdAt"|"updatedAt"), default_factory=_utc_now)`.
- Removed now-unused `Annotated` import.

### `src/contexter_server/models/skill.py`

- Added `AliasFieldInfo(FieldInfo)` class + `from pydantic.fields import FieldInfo`.
- `Skill.type`, `SkillCreate.type`:
  `AliasFieldInfo(validation_alias=AliasChoices("type", "category"))` (required).
- `Skill.file_path`:
  `AliasFieldInfo(validation_alias=AliasChoices("file_path", "filePath"), default=None, max_length=4096)`.
- `SkillPatch.type`:
  `AliasFieldInfo(validation_alias=AliasChoices("type", "category"), default=None)`.
- `Skill.created_at`/`updated_at`: AliasFieldInfo with `_utc_now` default_factory.
- Removed now-unused `Annotated` import.

## Verification Evidence (all commands from `/home/don/Code/contexter/contexter-server`)

| Command | Result |
|---|---|
| `python3 -m pytest -q tests/models/test_agent.py tests/models/test_skill.py tests/mcp/test_error_shape_drift.py tests/services/test_agent_skill_engine_live.py tests/api/test_agents.py tests/api/test_skills.py` | **78 passed**, 1 unrelated warning (starlette PendingDeprecationWarning) |
| `python3 -m pytest -q -W "error::pydantic.warnings.UnsupportedFieldAttributeWarning" tests/api/test_agents.py tests/api/test_skills.py` (was RED: 8 failed, 12 passed) | **20 passed**, 0 UnsupportedFieldAttributeWarning |
| `python3 -m pytest -q -W "always::pydantic.warnings.UnsupportedFieldAttributeWarning"` (full suite) | **819 passed**, 0 UnsupportedFieldAttributeWarning (baseline was 5) |
| `python3 -m py_compile src/contexter_server/models/agent.py src/contexter_server/models/skill.py` | OK |

Behavior probe (all pass):

- `tools` → `capabilities` (Agent, AgentCreate, AgentPatch)
- `category` → `type` (Skill, SkillCreate, SkillPatch); `filePath` → `file_path`
- `createdAt`/`updatedAt` accepted; canonical key wins when both present
- `version` u32→str coercion intact; `populate_by_name=True` intact
- Patch fields default to `None`; `required` semantics unchanged
  (`Skill.type` still required, `Agent.name` still required)
- Model defaults unchanged: `Agent(name="x").capabilities == []`,
  `Skill(name="y", type="m").version == "1"`

## Notes / Caveats

- **No commits created** (per instructions). Working tree on
  `feature/mcp-live-fix` contains many pre-existing uncommitted changes from
  the parent feature; only these two files were touched by this bug contract.
- The `git diff` against HEAD shows the parent feature's uncommitted work
  (docstring, `type`/`status`/`description` fields) mixed with this fix —
  do not stage/commit without review.
- Sibling-contract flakes observed during earlier runs
  (`tests/mcp/test_handlers_id_bounding.py` RED tests,
  `tests/api/test_security.py::TestBodySizeLimit`/`TestOpenApiDocs`,
  `tests/core/test_env_canonicalization.py`) were NOT caused by this change;
  the final full-suite run shows 819 passed.
- `Annotated` pattern from the bug's original description was proven invalid
  on pydantic 2.13.4 (silent alias drop); the `AliasFieldInfo` subclass pattern
  is the supported equivalent and is the documented FastAPI escape hatch.
- pydantic internals consulted: `_generate_schema.py` L2274/L2390/L2426,
  `fields.py` `merge_field_infos`/`_copy`/`from_annotated_attribute`,
  `fastapi/_compat.py` L285/L110.

## Skills Loaded

python-pro, python-expert, python-patterns, python-testing-patterns,
pytest-coverage, python-type-safety, debugging-wizard, diagnose,
systematic-debugging, debugging-strategies, handoff, clean-code,
git-workflow-and-versioning, verification-before-completion,
incremental-implementation, tdd, test-driven-development,
domain-driven-design
