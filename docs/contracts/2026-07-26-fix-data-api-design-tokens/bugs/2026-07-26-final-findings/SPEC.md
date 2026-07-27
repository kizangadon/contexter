# SPEC: Final remaining findings — search endpoint + Session test + optional coverage tests

Fixes F-01 partial, B4-REQ-03, N-06, N-08, N-09

## Fix 1 (F-01 partial): Strip embedding from search endpoint raw data

**Files:** `contexter-server/src/contexter_server/services/search_service.py`, `contexter-server/src/contexter_server/services/memory_service.py`

In both files, before constructing `SearchResult` with `data=r`, strip `embedding` from the raw dict:

```python
# Before creating SearchResult, strip embedding from raw dict to prevent
# embedding vectors from leaking through the search endpoint's raw dict pass-through
safe_data = {k: v for k, v in r.items() if k != 'embedding'}
SearchResult(
    id=r.get("id", ""),
    type="memory",
    score=r.get("score", 0.0),
    data=safe_data,  # embedding-free
    snippet=r.get("content", "")[:200] if r.get("content") else None,
)
```

This ensures the search endpoint (which returns `SearchResult.data: dict[str, Any]`) does NOT expose embedding vectors, even though the CRUD endpoints are protected by the Memory model's model_serializer.

**Affected lines:**
- `memory_service.py:54-63` — the `search` method
- `search_service.py:44-53` — the `search` method

## Fix 2 (B4-REQ-03): Add Session null agent_id test

**File:** `contexter-server/tests/models/test_session.py`

Add to `TestSessionModel` class:
```python
def test_session_agent_id_optional(self):
    """Session with no agent_id should default to None."""
    session = Session(project="test-project")
    assert session.agent_id is None
```

Note: Session model now has `agent_id: Optional[UUID] = Field(default=None, ...)` so omitting agent_id should be valid.

## Fix 3 (N-06, N-08, N-09): Optional coverage tests

**File:** `contexter-server/tests/models/test_memory.py`

Add to `TestMemoryModel` class:
```python
def test_embedding_excluded_from_serialization(self):
    """model_dump and model_dump_json should not include embedding."""
    mem = Memory(
        session_id=uuid.uuid4(),
        agent_id=uuid.uuid4(),
        content="test",
        embedding=[0.1, 0.2, 0.3],
    )
    data = mem.model_dump()
    assert 'embedding' not in data
    json_str = mem.model_dump_json()
    assert 'embedding' not in json_str

def test_naive_datetime_coerced_to_utc(self):
    """A naive datetime should be coerced to UTC-aware."""
    mem = Memory(
        session_id=uuid.uuid4(),
        agent_id=uuid.uuid4(),
        content="test",
        created_at=datetime(2024, 1, 1, 12, 0, 0),  # naive
    )
    assert mem.created_at.tzinfo is not None
    assert mem.created_at.tzinfo.utcoffset(mem.created_at) == timezone.utc.utcoffset(mem.created_at)

def test_status_done_normalized(self):
    """Session status 'done' should be normalized to 'completed'."""
    session = Session(agent_id=uuid.uuid4(), project="test", status="done")
    assert session.status == "completed"
```
