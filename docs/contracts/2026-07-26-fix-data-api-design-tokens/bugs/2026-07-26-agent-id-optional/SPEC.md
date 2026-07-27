# Bug SPEC: Make `agent_id` Optional[UUID] in Memory and Session models

## Context
The Pydantic `Memory` and `Session` models require `agent_id: UUID` without a default. This means any API response where the Rust engine does not provide an `agent_id` field will fail validation. This is the same null-risk pattern that was fixed for `session_id` in B-02 but `agent_id` was missed.

## Bug I1-S-01 (from Auto Bug Loop iteration 1)
`agent_id` remains required UUID; same null-risk pattern as fixed B-02.

## Fix
- In `Memory` model: change `agent_id: UUID` to `agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices(...))`
- In `Session` model: change `agent_id: UUID` to `agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices(...))`

## Design Decision D-A4 (this contract)
`agent_id` gets the same treatment as `session_id` — Optional with None default. This is the safe default: if the Rust engine does not supply `agent_id`, the field defaults to `None` in Python rather than raising a `ValidationError`.
