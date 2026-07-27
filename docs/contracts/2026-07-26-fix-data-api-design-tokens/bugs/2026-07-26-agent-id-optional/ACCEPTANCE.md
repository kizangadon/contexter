# Acceptance Criteria — `agent_id` Optional

## AC-AGENT-01: Memory model accepts null agent_id
**Given** a `Memory` model with no `agent_id` field in the input JSON
**When** the model is constructed (e.g., via `Memory.model_validate(data)` where `data` has no `agent_id`)
**Then** the model SHALL construct successfully with `agent_id = None`

## AC-AGENT-02: Session model accepts null agent_id
**Given** a `Session` model with no `agent_id` field in the input JSON
**When** the model is constructed
**Then** the model SHALL construct successfully with `agent_id = None`

## AC-AGENT-03: agent_id is serialized correctly
**Given** a `Memory` with `agent_id=None`
**When** serialized via `.model_dump(mode='json')`
**Then** the output SHALL contain `"agent_id": null`

## AC-AGENT-04: No regressions
**Given** a `Memory` with a valid `agent_id=UUID('...')`
**When** constructed and serialized
**Then** the round-trip SHALL preserve the UUID value
