# ACCEPTANCE — Pydantic Alias Annotated
- AC-PAA-001: GIVEN legacy payload with `tools`, WHEN Agent model parses it, THEN `capabilities` is populated (existing tests still pass).
- AC-PAA-002: GIVEN skill payload with `category`, WHEN Skill model parses it, THEN `type` is set.
- AC-PAA-003: WHEN the full test suite runs, THEN 0 failures AND 0 `UnsupportedFieldAttributeWarning` warnings.
