# Design Preview — `agent_id` Optional

## Change
Two model fields change from `agent_id: UUID` to `agent_id: Optional[UUID] = Field(default=None, ...)`:

```python
# memory.py — line ~27
agent_id: Optional[UUID] = Field(
    default=None,
    validation_alias=AliasChoices("agent_id", "agentId"),
)

# session.py — line ~29
agent_id: Optional[UUID] = Field(
    default=None,
    validation_alias=AliasChoices("agent_id", "agentId"),
)
```

## Rationale
Same pattern as `session_id` fix in B-02. The Rust engine may or may not supply `agent_id` in its JSON output. Making it `Optional` with `default=None` prevents `ValidationError` when the field is absent. This is the safe, defensive default.

## Risk
None. This is the same well-tested pattern applied in the previous iteration.
