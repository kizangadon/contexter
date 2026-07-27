# Edge Cases — `agent_id` Optional

## E-01: Null vs missing
Both `{"agent_id": null}` and `{}` (no `agent_id` key) SHALL produce `agent_id=None`. No difference in behavior.

## E-02: Invalid UUID string
`{"agent_id": "not-a-uuid"}` SHALL still raise a `ValidationError`. The `Optional` only affects the null/missing case, not the invalid-value case.

## E-03: CamelCase alias
`{"agentId": null}` SHALL also work via `validation_alias`. Both `agent_id` and `agentId` must be accepted.
