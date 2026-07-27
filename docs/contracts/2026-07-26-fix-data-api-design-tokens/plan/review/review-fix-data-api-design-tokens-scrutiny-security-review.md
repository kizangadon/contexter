# Security Review Report

# Fix Data API + Align Design Tokens

> Security review of 4 changed files: bridge.py (tilde expansion), memory.py and session.py (Pydantic model alignment), tokens.css (design token replacement).

**Verdict:** PASS (with observations) (class: pass)

2026-07-26 · 4 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 1 |

**Security Scope**

| Area | Files | Risk Profile |
|------|-------|-------------|
| Path expansion | bridge.py:81 (os.path.expanduser) | None — hardcoded path, not user-controlled |
| Pydantic model changes | memory.py, session.py | Low — new embedding field exposes vectors; populate_by_name is additive |
| Thread safety | bridge.py ThreadPoolExecutor | None — model_validate creates new instances only |
| CORS/CSRF | API endpoints | None — endpoint body shape does not affect CORS; auth/host protections remain |
| CSS tokens | tokens.css | None — static file, no user-controlled values |

---

## 02 · Vulnerability Findings


### F-01: Embedding vectors exposed in API responses  —  **Low**  
**File:** `contexter-server/src/contexter_server/models/memory.py:21`  
**CWE-200: Information Exposure**  

The new `embedding: Optional[list[float]]` field exposes the full embedding vector for each memory in API responses (`GET /api/v1/memories`). Embedding vectors encode semantic information about the content and, in advanced scenarios, can be used for:  
- **Embedding inversion attacks** — reconstructing input text from its vector representation  
- **User fingerprinting** — correlating sessions via embedding similarity  
- **Semantic profiling** — determining content topics without reading raw text  

**Mitigation reasoning:** The `content` field already contains the raw text, which is strictly more revealing than the embedding vector. The API is protected by API key authentication (`CONtexTER_API_KEY`). Nevertheless, exposing raw embedding vectors is unusual and should be documented.  
**Recommendation:** Consider exposing only a similarity score (not the raw vector) unless the frontend performs client-side embedding operations. Document the field's privacy implications.

---

### F-02: UTC datetime consistency depends on Rust serialization  —  **Informational**  
**File:** `contexter-server/src/contexter_server/models/memory.py:27-34`, `session.py:25-35`  
**CWE-20: Improper Input Validation**  

Per EC-006 in the edge cases catalog, if the Rust engine returns datetimes without timezone information (e.g., `2026-07-26T10:00:00` instead of `2026-07-26T10:00:00Z`), Pydantic will accept them as timezone-naive datetimes. This does not raise a `ValidationError` but can cause subtle time drift if the server processes and re-serializes these values, or if the frontend interprets them differently from UTC datetimes.  
**Recommendation:** Add a Pydantic `field_validator` to coerce timezone-naive datetimes to UTC on `model_validate`, or verify the Rust engine always emits `Z` / `+00:00` suffixes.

---

### F-03: Session status enum mismatch is a display concern, not a security issue  —  **Informational**  
**File:** `contexter-server/src/contexter_server/models/session.py:19`  

Per EC-014, the Rust engine returns `status: "completed"` but the frontend `statusVariant` map (in `DashboardPage.tsx`) expects `"done"`. Pydantic accepts any string — no validation error occurs. The mismatch means completed sessions will not render with the correct badge variant on the dashboard. This is a data-integrity concern (not a security vulnerability) and is correctly marked out of scope in the design preview.

---

### F-04: Potential null UUID rejection for missing foreign keys  —  **Informational**  
**File:** `contexter-server/src/contexter_server/models/memory.py:16`, `session.py:16`  

Per EC-004, if the Rust engine ever returns `sessionId: null` or `agentId: null`, Pydantic will raise a `ValidationError` because `session_id: UUID` and `agent_id: UUID` are not `Optional[UUID]`. This would cause the affected record to be silently dropped from API responses. While Rust is expected to always provide these fields, a null foreign key is a plausible edge case during data migration or corruption.  
**Recommendation:** Consider making `session_id` and `agent_id` `Optional[UUID]` with a `None` default if null values are theoretically possible from the storage layer.

---

## 03 · Security-Critical Code Highlights

```python
# bridge.py:77-96 — path expansion (safe, hardcoded)
def __init__(self, path: str, max_workers: int | None = None) -> None:
    expanded_path = os.path.expanduser(path)  # ← SAFE: ~ expansion only, path is hardcoded
    ...
    self._engine = _SyncEngine.open(expanded_path)
```

```python
# memory.py:13 — populate_by_name (safe, additive)
model_config = ConfigDict(populate_by_name=True)
# Accepts both: Memory(sessionId=...) AND Memory(session_id=...)
# No security restriction bypass — purely additive flexibility
```

```python
# memory.py:21 — embedding exposure (low risk)
embedding: Optional[list[float]] = None
# Exposes raw embedding vectors in API responses.
# Risk mitigated by: (1) content field already exposed, (2) API key auth
```

---

## 04 · Remediation Recommendations

> **Must Fix**
> None. No critical or high-severity findings identified.

> **Should Fix**
> 1. **Document `embedding` field privacy implications** (F-01): Add a note in the API docs/spec that embedding vectors are exposed in responses and should be handled as sensitive data.
2. **Add datetime timezone validation** (F-02): If the Rust engine can emit timezone-naive datetimes, add a Pydantic `field_validator` to coerce them to UTC.

> **Consider**
> 1. Make `session_id` and `agent_id` `Optional[UUID]` (F-04) if the Rust engine could theoretically return null foreign keys.
2. Consider exposing embedding similarity scores instead of raw vectors if client-side embedding operations are not required.

---

_Generated by Security Architect · 2026-07-26 · Validation Contract: 2026-07-26-fix-data-api-design-tokens_
