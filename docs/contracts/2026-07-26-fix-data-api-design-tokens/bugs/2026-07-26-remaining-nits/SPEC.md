# SPEC: Remaining Nits — Align alias strategy + Add missing tests

## Fixes

### 1. Align alias strategy (N-05)
`session.py` line ~29 uses `validation_alias="sessionId"` (single string), while `memory.py` and `session.py` for `agent_id` use `AliasChoices("agent_id", "agentId")` (tuple). This is inconsistent within the same module.

**Fix:** Change `session_id` in `session.py` from single-string `validation_alias` to use `AliasChoices("session_id", "sessionId")` to match the `agent_id` pattern. Import `AliasChoices` (should already be imported).

### 2. Add null agent_id test (N-02)
Add one test to `tests/models/test_memory.py`:
```python
def test_agent_id_optional_none(self):
    """Memory without agent_id defaults to None."""
    mem = Memory(id=uuid4(), session_id=None, content="test")
    assert mem.agent_id is None
```

### 3. Add role=None test (B2-AC-04)
Add to existing `test_role_default_is_system` test or as a new test in `tests/models/test_memory.py`:
```python
def test_role_explicit_none(self):
    """Memory with explicit role=None should be None, not 'system'."""
    mem = Memory(id=uuid4(), session_id=None, agent_id=None, content="test", role=None)
    assert mem.role is None
```

## Design Decisions (removed from scope)
- **F-01** (embedding exposure): Accepted trade-off. The feature exposes memory data including embeddings by design.
- **F-02** (timezone consistency): Data is already UTC. No conversion needed.
- **F-03** (session status mismatch): Out of scope per design preview.
- **Design partials (Optional UUID deviations)**: Documented in D-A4 as accepted defensive null-safety pattern.
