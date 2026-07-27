# Security Review Report

# Fix Data API + Align Design Tokens — Iteration 4

> Re-assessment of all findings after iteration 4 changes: embedding stripped from raw dict in both `memory_service.py:59` and `search_service.py:50` before constructing `SearchResult.data`, plus new tests for embedding serializer, session null agent_id, UTC coercion, and status normalization.

**Verdict:** CLEAR (class: pass)

2026-07-27 · 0 findings (all 7 resolved) · Security Architect (Iteration 4)

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 0 |

> **Security Scope**
> Full re-assessment of all 7 tracked findings (F-01 through F-04, N-01, N-02, N-05, N-06, N-07) after iteration 4 bug-fix changes targeting the search endpoint embedding leak, plus new regression tests.

---

## 02 · Vulnerability Findings

**All previous findings are RESOLVED. No new findings identified.**

### F-01: Embedding vectors exposed in API responses — **Low** (→ **Fully Resolved**)

**Iteration 4 fix:** The remaining gap from iter-3 is now closed. Both `memory_service.py:59` and `search_service.py:50` filter the `embedding` key from raw bridge dicts before constructing `SearchResult.data`.

```python
# memory_service.py:55-63 — NEW: dict comprehension strips embedding
results = [
    SearchResult(
        id=r.get("id", ""),
        type="memory",
        score=r.get("score", 0.0),
        data={k: v for k, v in r.items() if k != "embedding"},
        snippet=r.get("content", "")[:200] if r.get("content") else None,
    )
    for r in memory_results
]
```

```python
# search_service.py:44-53 — NEW: same fix in SearchService
for r in memory_results_list:
    results.append(
        SearchResult(
            id=r.get("id", ""),
            type="memory",
            score=r.get("score", 0.0),
            data={k: v for k, v in r.items() if k != "embedding"},
            snippet=r.get("content", "")[:200] if r.get("content") else None,
        )
    )
```

**What is now covered (complete chain):**
| Endpoint | Protection | Since |
|---|---|---|
| `POST /api/v1/memories` | `model_serializer` on `Memory` strips embedding | Iter-3 |
| `GET /api/v1/memories` | `model_serializer` on `Memory` strips embedding | Iter-3 |
| `GET /api/v1/memories/{id}` | `model_serializer` on `Memory` strips embedding | Iter-3 |
| `PUT /api/v1/memories/{id}` | `model_serializer` on `Memory` strips embedding | Iter-3 |
| `GET /api/v1/memories/search` | Dict comprehension in `memory_service.py:59` strips embedding | **Iter-4** |
| `GET /api/v1/search` | Dict comprehension in `search_service.py:50` strips embedding | **Iter-4** |
| Session results in `GET /api/v1/search` | Session model has **no** `embedding` field — not applicable | N/A |

**Verification:** Session results in `search_service.py:55-64` pass `data=s` (raw session dict) without filtering. The `Session` model has no `embedding` field — it includes `agent_id`, `project`, `name`, `status`, `turn_count`, `duration_ms`, `efficiency_score`, timestamps, and `metadata`. No embedding vector exposure is possible through this path.

**Status:** FULLY RESOLVED. All code paths that could expose embedding vectors are now covered.

---

### F-02: UTC datetime consistency — **Informational** (→ **Resolved**)

No changes in iter-4. Previously resolved in iter-3 with `ensure_utc` field validators on both `Memory` and `Session` models.

**New regression test:** `test_naive_datetime_coerced_to_utc` (test_memory.py:120-129) verifies that a timezone-naive `datetime` passed to `Memory.created_at` is coerced to UTC-aware. Test passes. ✓

**Status:** RESOLVED. ✓

---

### F-03: Session status mismatch — **Informational** (→ **Resolved**)

No changes in iter-4. Previously resolved in iter-3 with `normalize_status` validator.

**New regression test:** `test_status_done_normalized` (test_session.py:99-102) verifies `'done'` → `'completed'` normalization. Test passes. ✓

**Status:** RESOLVED. ✓

---

### F-04: Null UUID rejection for foreign keys — **Informational** (→ **Resolved**)

No changes in iter-4. Previously resolved in iter-1/iter-2.

**New regression test:** `test_session_agent_id_optional` (test_session.py:94-97) verifies `Session` can be constructed without `agent_id`, defaulting to `None`. Test passes. ✓

**Status:** RESOLVED (unchanged from iter-3). ✓

---

### N-01: agent_id non-optional — **Informational** (→ **Resolved**)

Unchanged from iter-2. No regressions.

**Status:** RESOLVED. ✓

---

### N-02: No null agent_id test — **Informational** (→ **Resolved**)

**Iteration 3 fix:** `test_agent_id_optional_none` and `test_role_explicit_none` added. No regressions in iter-4.

**Status:** RESOLVED. ✓

---

### N-05: session_id alias missing AliasChoices — **Informational** (→ **Resolved**)

Unchanged from iter-3. No regressions.

**Status:** RESOLVED. ✓

---

### N-06: No embedding serializer test — **Informational** (→ **Resolved**)

**Iteration 4 fix:** `test_embedding_excluded_from_serialization` added at test_memory.py:107-118:

```python
def test_embedding_excluded_from_serialization(self):
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
```

**Verification:**
- `model_dump()` excludes `embedding` from dict output ✓
- `model_dump_json()` excludes `embedding` from JSON string output ✓
- Python object still has `mem.embedding == [0.1, 0.2, 0.3]` (field is retained for internal use) ✓

This test guards against: Pydantic version upgrade altering serializer behavior, refactors changing `mode` from `wrap` to `plain`, and future `serialization_alias` additions bypassing the `pop`.

**Status:** RESOLVED. ✓

---

### N-07: Search endpoint may still expose embedding vectors — **Informational** (→ **Resolved**)

**Iteration 4 fix:** Both `memory_service.py:59` and `search_service.py:50` now use `{k: v for k, v in r.items() if k != "embedding"}` instead of passing the raw dict through as `data=r`.

**Verification:** The fix covers both search endpoints — the dedicated memory search (`GET /api/v1/memories/search`) and the cross-entity search (`GET /api/v1/search`). The session results path (`data=s`) is unaffected because the `Session` model has no `embedding` field.

**Status:** RESOLVED. ✓

---

## 03 · Security-Critical Code Highlights

```python
# memory_service.py:55-63 — FIXED: embedding filtered from search results
SearchResult(
    data={k: v for k, v in r.items() if k != "embedding"},  # ← was data=r
    ...
)
```

```python
# search_service.py:44-53 — FIXED: same fix for cross-entity search
SearchResult(
    data={k: v for k, v in r.items() if k != "embedding"},  # ← was data=r
    ...
)
```

```python
# test_memory.py:107-118 — NEW: regression test for embedding serializer
def test_embedding_excluded_from_serialization(self):
    mem = Memory(..., embedding=[0.1, 0.2, 0.3])
    data = mem.model_dump()
    assert 'embedding' not in data
    json_str = mem.model_dump_json()
    assert 'embedding' not in json_str
```

```python
# test_session.py:94-97 — NEW: regression test for null agent_id
def test_session_agent_id_optional(self):
    session = Session(project="test-project")
    assert session.agent_id is None
```

```python
# test_session.py:99-102 — NEW: regression test for status normalization
def test_status_done_normalized(self):
    session = Session(agent_id=..., project="test", status="done")
    assert session.status == "completed"
```

```python
# test_memory.py:120-129 — NEW: regression test for UTC coercion
def test_naive_datetime_coerced_to_utc(self):
    mem = Memory(..., created_at=datetime(2024, 1, 1, 12, 0, 0))
    assert mem.created_at.tzinfo is not None
```

---

## 04 · Remediation Recommendations

> **Must Fix**
> None. All critical, high, and medium findings resolved.

> **Should Fix**
> None. All findings closed.

> **Consider**
> None. No new security considerations identified in iteration 4.

---

## 05 · Final Finding Reconciliation

| ID | Description | Severity | Iter-3 Status | Iter-4 Status |
|---|---|---|---|---|
| F-01 | Embedding exposure in API | Low | PARTIALLY RESOLVED | **FULLY RESOLVED** |
| F-02 | UTC timezone consistency | Info | RESOLVED | RESOLVED |
| F-03 | Session status mismatch | Info | RESOLVED | RESOLVED |
| F-04 | Null UUID rejection | Info | RESOLVED | RESOLVED |
| N-01 | agent_id non-optional | Info | RESOLVED | RESOLVED |
| N-02 | No null agent_id test | Info | RESOLVED | RESOLVED |
| N-05 | session_id alias missing AliasChoices | Info | RESOLVED | RESOLVED |
| N-06 | No embedding serializer test | Info | NEW | **RESOLVED** |
| N-07 | Search endpoint embedding exposure | Info | NEW | **RESOLVED** |

**All 9 findings across all iterations: 9 RESOLVED, 0 open.**

---

_Generated by Security Architect (Iteration 4) · 2026-07-27 · Validation Contract: 2026-07-26-fix-data-api-design-tokens_
