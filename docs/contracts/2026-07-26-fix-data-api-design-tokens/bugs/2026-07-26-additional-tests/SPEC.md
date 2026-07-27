# SPEC: Additional tests — null agent_id + role=None

Fixes 2 findings: N-02 (Security), B2-AC-04 (SPEC Compliance)

## Fix 1 (N-02): Test null agent_id acceptance

**File:** `contexter-server/tests/models/test_memory.py`

Add to `TestMemoryModel` class:
```python
def test_agent_id_optional_none(self):
    """Memory with no agent_id defaults to None."""
    mem = Memory(
        session_id=uuid.uuid4(),
        content="test memory without agent_id",
    )
    assert mem.agent_id is None
```

**Also check** if there's a `tests/models/test_session.py` — if so, add a similar test there. If not, just the Memory test is sufficient.

## Fix 2 (B2-AC-04): Test explicit role=None

**File:** `contexter-server/tests/models/test_memory.py`

Add to `TestMemoryModel` class:
```python
def test_role_explicit_none(self):
    """Memory with explicit role=None should be None, not 'system'."""
    mem = Memory(
        session_id=uuid.uuid4(),
        agent_id=uuid.uuid4(),
        content="explicit null role",
        role=None,
    )
    assert mem.role is None
```
