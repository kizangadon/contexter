# SPEC — Pydantic Alias Annotated (iter-1 findings)

## Context
pydantic 2.13.4 emits `UnsupportedFieldAttributeWarning` (5×) for `validation_alias=AliasChoices(...)`
inside `Field()` at models/agent.py L43-46 and models/skill.py L43-58. Functionality verified
correct (legacy `tools`→`capabilities`, `category`→`type` maps work), but the pattern is fragile
and noisy.

## Requirements
- REQ-PAA-001: Replace `Field(validation_alias=AliasChoices(...))` with `Annotated[...,
  AliasChoices(...)]` metadata (or the pydantic-2.13-supported alias mechanism) in models/agent.py
  and models/skill.py.
- REQ-PAA-002: Behavior MUST remain byte-identical: `tools` legacy alias maps to `capabilities`,
  `category`→`type`, version coercion unchanged — all existing tests pass with ZERO warnings from
  these models.
- REQ-PAA-003: Run the test suite; confirm 0 `UnsupportedFieldAttributeWarning` occurrences and
  0 failures.
