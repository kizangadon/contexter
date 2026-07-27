# SPEC: Model hardening — alias alignment, embedding protection, timezone coercion, status normalization

Fixes 4 findings: N-05, F-01, F-02, F-03

## Fix 1 (N-05): Align `session_id` alias to AliasChoices

**File:** `contexter-server/src/contexter_server/models/memory.py` line 17

**Current:** `session_id: Optional[UUID] = Field(default=None, validation_alias="sessionId")`
**Fix:** Change to `session_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("session_id", "sessionId"))`

This aligns with the `agent_id` pattern which already uses `AliasChoices`. Both fields in the same module should use the same strategy. AliasChoices is already imported.

## Fix 2 (F-01): Exclude `embedding` from default serialization

**File:** `contexter-server/src/contexter_server/models/memory.py`

**Current:** `embedding: Optional[list[float]] = None`
**Fix:** Add a `model_serializer` that strips `embedding` from JSON serialization output. This prevents embedding vectors from being exposed in API responses while preserving the field in the Pydantic model for internal use.

Add import: `from pydantic import model_serializer` (or if already imported from `BaseModel, Field, AliasChoices, ConfigDict`, update the import)

Add method to Memory class:
```python
@model_serializer(mode='wrap')
def _serialize_without_embedding(self, handler):
    data = handler(self)
    data.pop('embedding', None)
    return data
```

## Fix 3 (F-02): Coerce timezone-naive datetimes to UTC

**File:** `contexter-server/src/contexter_server/models/memory.py` + `contexter-server/src/contexter_server/models/session.py`

Add a `model_validator` or per-field `field_validator` that converts timezone-naive datetimes to UTC-aware ones.

In both models, add:
```python
from pydantic import field_validator

@field_validator('created_at', 'updated_at', 'started_at', 'last_active', 'completed_at', mode='before')
@classmethod
def ensure_utc(cls, v):
    if isinstance(v, datetime) and v.tzinfo is None:
        return v.replace(tzinfo=timezone.utc)
    return v
```

Note: only apply the validator to fields that exist in each model. Memory has `created_at`, `updated_at`. Session has `started_at`, `updated_at`, `last_active`, `completed_at`.

## Fix 4 (F-03): Normalize session status values

**File:** `contexter-server/src/contexter_server/models/session.py`

Add a `field_validator` for `status` that normalizes status values:
```python
@field_validator('status', mode='before')
@classmethod
def normalize_status(cls, v):
    if v == 'done':
        return 'completed'
    return v
```
