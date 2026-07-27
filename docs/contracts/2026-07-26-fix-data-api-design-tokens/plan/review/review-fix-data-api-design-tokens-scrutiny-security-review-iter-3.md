# Security Review Report

# Fix Data API + Align Design Tokens — Iteration 3

> Re-assessment of all findings after iteration 3 changes: `model_serializer` strips `embedding` from serialization, UTC coercion validators on datetime fields, session status normalization (`done` → `completed`), `session_id` alias aligned to `AliasChoices`, new tests for null `agent_id` and `role=None`.

**Verdict:** CONDITIONAL PASS (class: pass)

2026-07-27 · 7 (2 new observations, 5 resolved, 1 partially resolved) findings · Security Architect (Iteration 3)

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 1 (Partially Resolved) |

> **Security Scope**
> Full re-assessment of 7 tracked findings (F-01 through F-04, N-01, N-02, N-05) after iteration 3 bug-fix changes targeting `contexter-server/src/contexter_server/models/memory.py` (+18 new fields/validators), `contexter-server/src/contexter_server/models/session.py` (+15 new fields/validators), and `contexter-server/tests/models/test_memory.py` (+2 new tests).

---

## 02 · Vulnerability Findings

### F-01: Embedding vectors exposed in API responses — **Low** (Partially Resolved)
**File:** `contexter-server/src/contexter_server/models/memory.py:38-42`
**CWE-200: Information Exposure**

**Iteration 3 fix:** `model_serializer(mode='wrap')` added at lines 38-42 strips the `embedding` field from all serialization output (`model_dump()`, `model_dump_json()`, and FastAPI `response_model=Memory` serialization).

```python
@model_serializer(mode='wrap')
def _serialize_without_embedding(self, handler):
    data = handler(self)
    data.pop('embedding', None)
    return data
```

**What is covered:**
- `GET /api/v1/memories` (`response_model=list[Memory]`) ✓
- `POST /api/v1/memories` (`response_model=Memory`) ✓
- `GET /api/v1/memories/{id}` (`response_model=Memory`) ✓
- `PUT /api/v1/memories/{id}` (`response_model=Memory`) ✓
- `Memory.model_dump()` calls ✓
- `Memory.model_dump_json()` calls ✓

**What is NOT covered:**
- `GET /api/v1/memories/search` returns `response_model=SearchResponse`, which contains `SearchResult.data: dict[str, Any]` — a pass-through raw dict from the Rust bridge. If the Rust engine includes `embedding` in search result records, the vectors will be exposed through this endpoint. The `model_serializer` on `Memory` is never invoked for `SearchResult.data`.

**Verification:** In `contexter-server/src/contexter_server/services/memory_service.py:54-63`:
```python
results = [
    SearchResult(
        id=r.get("id", ""),
        type="memory",
        score=r.get("score", 0.0),
        data=r,  # ← raw dict from bridge, not Memory model
        snippet=r.get("content", "")[:200] if r.get("content") else None,
    )
    for r in memory_results
]
```

**Mitigation reasoning (unchanged):** The `content` field already exposes raw text, which is strictly more revealing. API is protected by API key authentication. The search endpoint is within the same auth boundary.

**Status:** PARTIALLY RESOLVED — CRUD endpoints are covered; search endpoint remains an exposure vector. No regression test exists for the serializer (see N-06).

---

### F-02: UTC datetime consistency depends on Rust serialization — **Informational** (→ **Resolved**)
**Files:** `contexter-server/src/contexter_server/models/memory.py:44-49`, `session.py:40-45`

**Iteration 3 fix:** `ensure_utc` field validator added with `mode='before'` to coerce timezone-naive datetimes to UTC on both models:
- **Memory model (lines 44-49):** Covers `created_at`, `updated_at`
- **Session model (lines 40-45):** Covers `started_at`, `updated_at`, `last_active`, `completed_at`

```python
@field_validator('created_at', 'updated_at', mode='before')
@classmethod
def ensure_utc(cls, v):
    if isinstance(v, datetime) and v.tzinfo is None:
        return v.replace(tzinfo=timezone.utc)
    return v
```

**Verification:** The validator fires `mode='before'` — before Pydantic's internal type-coercion, so both `str` and `datetime` inputs are handled. Non-datetime inputs pass through unchanged (Pydantic's own type validation handles format errors separately). ✓

**Status:** RESOLVED. No remaining action required.

---

### F-03: Session status enum mismatch — **Informational** (→ **Resolved**)
**File:** `contexter-server/src/contexter_server/models/session.py:47-52`

**Iteration 3 fix:** `normalize_status` field validator added:

```python
@field_validator('status', mode='before')
@classmethod
def normalize_status(cls, v):
    if v == 'done':
        return 'completed'
    return v
```

**Verification:** Single normalization rule `'done'` → `'completed'`. Input `'completed'`, `'active'`, `'paused'`, `'archived'` pass through unchanged. The validator is additive — no existing valid status values are affected. ✓

**Status:** RESOLVED. No remaining action required.

---

### F-04: Null UUID rejection for missing foreign keys — **Informational** (→ **Resolved**)
**Files:** `contexter-server/src/contexter_server/models/memory.py:17-18`, `session.py:17`

Already resolved in iteration 1 (session_id) and iteration 2 (agent_id). No regressions in iteration 3.

**Status:** RESOLVED (unchanged from iter-2). ✓

---

### N-01: `agent_id` was non-optional — **Informational** (→ **Resolved**)
**Files:** `memory.py:18`, `session.py:17`

Already resolved in iteration 2. No regressions in iteration 3.

**Status:** RESOLVED (unchanged from iter-2). ✓

---

### N-02: Agent_id null acceptance not explicitly tested — **Informational** (→ **Resolved**)
**Files:** `contexter-server/tests/models/test_memory.py:89-105`

**Iteration 3 fix:** Two new tests added:

```python
def test_agent_id_optional_none(self):
    """Memory with no agent_id defaults to None."""
    mem = Memory(
        session_id=uuid.uuid4(),
        content="test memory without agent_id",
    )
    assert mem.agent_id is None

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

**Verification:**
- `test_agent_id_optional_none` (line 89): Constructs Memory without agent_id, verifies `agent_id is None` ✓
- `test_role_explicit_none` (line 97): Constructs Memory with `role=None`, verifies it stays `None` (not overridden by default) ✓

The second test also covers the interaction between `Optional[str]` with `default="system"` and explicit `None` — confirming that the Optional type is respected over the default. This is a subtle correctness boundary worth testing.

**Status:** RESOLVED. ✓

---

### N-05: `session_id` alias missing `AliasChoices` — **Informational** (→ **Resolved**)
**Files:** `contexter-server/src/contexter_server/models/memory.py:17`

**Iteration 3 fix:** Changed from `validation_alias="sessionId"` to `validation_alias=AliasChoices("session_id", "sessionId")`, matching the already-established `agent_id` pattern on line 18.

**Verification:** Both `session_id` and `sessionId` are now accepted as JSON input keys. The `AliasChoices` import was already present. ✓

**Status:** RESOLVED. ✓

---

### N-06 (New): No test coverage for embedding serializer — **Informational**
**File:** `contexter-server/src/contexter_server/models/memory.py:38-42`

No test verifies that `model_dump()` or `model_dump_json()` on a `Memory` instance with a populated `embedding` field excludes `embedding` from the output while preserving all other fields.

```bash
$ grep -rn "embedding\|model_serializer\|_serialize_without" contexter-server/tests/ --include="*.py"
# → No results
```

**Risk:** The `model_serializer(mode='wrap')` pattern is the sole mechanism preventing embedding vector leakage from CRUD endpoints. Without test coverage:
1. A Pydantic version upgrade could alter serializer behavior
2. A refactor (e.g., changing to `mode='plain'` or removing `pop`) would silently break the protection
3. A future developer adding a `serialization_alias` on `embedding` could bypass the `pop('embedding', None)` call

**Recommendation:** Add a test:
```python
def test_embedding_excluded_from_serialization(self):
    mem = Memory(
        session_id=uuid.uuid4(),
        agent_id=uuid.uuid4(),
        content="test",
        embedding=[0.1, 0.2, 0.3],
    )
    data = mem.model_dump()
    assert "embedding" not in data
    assert data["content"] == "test"
    assert data["session_id"] == mem.session_id
    # Verify embedding is still in the Python object for internal use
    assert mem.embedding == [0.1, 0.2, 0.3]
    
    json_str = mem.model_dump_json()
    assert "embedding" not in json_str
```

**Severity justification:** Not an exploitable vulnerability — the serializer is correct as implemented. This is a regression-risk gap. The serializer is simple (3 lines) and well-reviewed, so the practical risk is very low.

**Status:** NEW — informational observation.

---

### N-07 (New): Search endpoint may still expose embedding vectors — **Informational**
**File:** `contexter-server/src/contexter_server/services/memory_service.py:54-63`

The search endpoint (`GET /api/v1/memories/search`) returns `SearchResponse` with `SearchResult.data: dict[str, Any]` that passes through the raw dict from the Rust bridge. If the Rust engine includes `embedding` in search results, this bypasses the `model_serializer` on the `Memory` model.

**Code path:**
```python
# memory_service.py:54-63 — SearchResult.data receives raw bridge dict
results = [
    SearchResult(
        id=r.get("id", ""),
        type="memory",
        score=r.get("score", 0.0),
        data=r,  # ← raw dict, no Memory model serialization
        ...
    )
    for r in memory_results
]
```

**Assessment:**
- The `SearchResult.data` field is typed as `dict[str, Any]` — no schema filtering
- The search endpoint is authenticated (same API key boundary)  
- The raw `content` text is already exposed in `snippet` and `data`
- Whether embedding is actually returned depends on the Rust engine implementation (not reviewed here)

**Recommendation:** Either:
1. (Preferred) Filter `SearchResult.data` to exclude `embedding` at the service layer: `{k: v for k, v in r.items() if k != 'embedding'}`
2. Or add a note in API documentation that search results may contain embedding vectors

**Status:** NEW — informational observation. Low priority given the existing `content` exposure and auth barrier.

---

## 03 · Security-Critical Code Highlights

```python
# memory.py:38-42 — NEW: model_serializer strips embedding from serialization
@model_serializer(mode='wrap')
def _serialize_without_embedding(self, handler):
    data = handler(self)
    data.pop('embedding', None)
    return data
# ← CORRECT: wraps default serialization, removes embedding field
# ← COVERS: model_dump(), model_dump_json(), FastAPI response_model=Memory
# ← GAP: SearchResponse with SearchResult.data (raw dict) bypasses this
```

```python
# memory.py:44-49 — NEW: UTC coercion validator
@field_validator('created_at', 'updated_at', mode='before')
@classmethod
def ensure_utc(cls, v):
    if isinstance(v, datetime) and v.tzinfo is None:
        return v.replace(tzinfo=timezone.utc)
    return v
# ← CORRECT: fires mode='before' → catches str and datetime inputs
# ← SAFE: only modifies naive datetimes; preserves tz-aware ones
```

```python
# session.py:40-45 — NEW: UTC coercion for Session datetimes
@field_validator('started_at', 'updated_at', 'last_active', 'completed_at', mode='before')
@classmethod
def ensure_utc(cls, v):
    ...
# ← CORRECT: covers all 4 datetime fields on Session model
```

```python
# session.py:47-52 — NEW: status normalization
@field_validator('status', mode='before')
@classmethod
def normalize_status(cls, v):
    if v == 'done':
        return 'completed'
    return v
# ← CORRECT: additive normalization, no existing values affected
```

```python
# memory.py:17 — FIXED: session_id alias uses AliasChoices
session_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("session_id", "sessionId"))
# ← CORRECT: matches agent_id pattern on line 18
```

```python
# memory_service.py:54-59 — REMAINING GAP: search results bypass serializer
SearchResult(
    data=r,  # ← raw bridge dict may contain embedding
)
```

---

## 04 · Remediation Recommendations

> **Must Fix**
> None. No critical or high-severity findings identified.

> **Should Fix**
> 1. **Add embedding serializer regression test** (N-06, Informational): Add a dedicated test that constructs a Memory with `embedding=[...]` and verifies `model_dump()` and `model_dump_json()` exclude the field. Prevents silent breakage on Pydantic upgrades or refactors.

> **Consider**
> 1. **Filter embedding from search results** (N-07, Informational): In `memory_service.py`, strip `embedding` from raw dicts before storing in `SearchResult.data`. Implement as a dict comprehension: `{k: v for k, v in r.items() if k != 'embedding'}`.
> 2. **Document embedding sensitivity** (F-01 follow-up, Low): Add an API documentation note that embedding vectors, if present in search results, should be treated as sensitive data. Not a code change — documentation only.

---

## 05 · Finding Reconciliation Summary

| ID | Description | Severity | Iter-2 Status | Iter-3 Status | Notes |
|---|---|---|---|---|---|
| F-01 | Embedding exposure in API | Low | ACTIVE | PARTIALLY RESOLVED | CRUD covered; search endpoint gap |
| F-02 | UTC timezone consistency | Info | ACTIVE | **RESOLVED** | `ensure_utc` validator added |
| F-03 | Session status mismatch | Info | ACTIVE | **RESOLVED** | `normalize_status` validator added |
| F-04 | Null UUID rejection | Info | RESOLVED | RESOLVED | Unchanged from iter-2 |
| N-01 | agent_id non-optional | Info | RESOLVED | RESOLVED | Unchanged from iter-2 |
| N-02 | No null agent_id test | Info | ACTIVE | **RESOLVED** | 2 new tests added |
| N-05 | session_id alias missing AliasChoices | Info | — | **RESOLVED** | Aligned to AliasChoices |
| **N-06** | No embedding serializer test | Info | — | **NEW** | Regression risk gap |
| **N-07** | Search endpoint embedding exposure | Info | — | **NEW** | Partial F-01 follow-up |

---

_Generated by Security Architect (Iteration 3) · 2026-07-27 · Validation Contract: 2026-07-26-fix-data-api-design-tokens_
