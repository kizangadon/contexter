# SPEC: Update approved design preview to reflect Optional[UUID] fields

Fixes 3 Design Compliance findings: #1, #2, #3

## Context
The approved design preview at `plan/preview/preview-fix-data-api-design-tokens-approved.md` shows `session_id` and `agent_id` as required `UUID` fields. During implementation, these were changed to `Optional[UUID]` as a defensive measure against null values from the Rust engine (documented in bug contracts `2026-07-26-pydantic-hardening` and `2026-07-26-agent-id-optional`).

## Changes needed in the design preview

1. In the `GET /api/v1/memories` response schema section: Change `session_id: UUID` to `session_id: Optional[UUID]` and `agent_id: UUID` to `agent_id: Optional[UUID]`
2. In the `GET /api/v1/sessions` response schema section: Change `agent_id: UUID` to `agent_id: Optional[UUID]`
3. In the D-A1 column (Decision Log): Add note that `session_id` and `agent_id` were relaxed to Optional as a defensive null-safety measure

Read the approved design preview file first, then make these changes.
