# Acceptance Criteria

1. `session_id` uses same `AliasChoices` pattern as `agent_id` in session.py
2. New test verifies Memory with no agent_id has `agent_id is None`
3. New test verifies Memory with explicit `role=None` has `role is None`
4. All 610+ existing tests still pass
