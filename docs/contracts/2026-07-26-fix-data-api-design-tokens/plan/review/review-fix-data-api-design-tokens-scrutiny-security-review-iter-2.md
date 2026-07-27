# Security Review Report

# Fix Data API + Align Design Tokens — Iteration 2

> Re-assessment of all findings after iteration 2 changes: `agent_id` made Optional[UUID] in Memory and Session models (same pattern as session_id in iter-1), AliasChoices for dual camelCase/snake_case input, bridge expanduser hardening, two new regression tests (expanduser, role default).

**Verdict:** PASS (class: pass)

2026-07-26 · 4 (0 new, 3 unchanged, 1 resolved) findings · Security Architect (Iteration 2)

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 1 |

> **Security Scope**
> Re-assessment of all 5 previous findings (F-01 through F-04, N-01) after iteration 2 bug-fix changes. Scope covers 6 changed files: memory.py (agent_id Optional + AliasChoices), session.py (agent_id Optional + AliasChoices + new fields), bridge.py (expanduser resolution), test_bridge.py (expanduser test), test_memory.py (role default test), tokens.css (V2-DEEP token redesign + backward-compat aliases).

---

## 02 · Vulnerability Findings


### F-01: Embedding vectors exposed in API responses — **Low** (Re-affirmed, unchanged)
**File:** `contexter-server/src/contexter_server/models/memory.py:22`
**CWE-200: Information Exposure**

The `embedding: Optional[list[float]]` field remains exposed in API responses. No changes were applied to mitigate this finding. The risk profile is unchanged:
- Embedding inversion attacks — reconstructing input text from its vector
- User fingerprinting — correlating sessions via embedding similarity
- Semantic profiling — determining content topics without reading raw text

**Mitigation reasoning:** The `content` field already exposes raw text, which is strictly more revealing. API is protected by API key authentication. Low severity — no urgent action required.
**Status:** ACTIVE — no bug contract addressed this finding.

---

### F-02: UTC datetime consistency depends on Rust serialization — **Informational** (Re-affirmed, unchanged)
**Files:** `contexter-server/src/contexter_server/models/memory.py:28-35`, `session.py:26-36`
**CWE-20: Improper Input Validation**

No `field_validator` was added to coerce timezone-naive datetimes to UTC. If the Rust engine emits datetimes without `Z`/`+00:00` suffixes, Pydantic will accept them as timezone-naive, which can cause subtle time drift in display/timestamp calculations.

**Status:** ACTIVE — no bug contract addressed this finding.

---

### F-03: Session status enum mismatch — **Informational** (Re-affirmed, unchanged)
**File:** `contexter-server/src/contexter_server/models/session.py:20`

No changes to the session `status` field. Rust returns `"completed"`, frontend expects `"done"`. The status field is `str` typed and accepts any string value. This is a data-integrity / display concern, not a security vulnerability.

**Status:** ACTIVE — documented as out of scope in design preview.

---

### F-04: Null UUID rejection for missing foreign keys — **→ RESOLVED (Informational)**
**Files:** `contexter-server/src/contexter_server/models/memory.py:17-18`, `session.py:17`

**Resolution:** Both `session_id` (iter-1) and `agent_id` (iter-2) are now `Optional[UUID] = Field(default=None, ...)`. If the Rust engine returns a null foreign key for either field, the model will gracefully default to `None` instead of raising a `ValidationError` that drops the record.

- Memory model: `session_id: Optional[UUID]` (line 17, iter-1) + `agent_id: Optional[UUID]` (line 18, iter-2)
- Session model: `agent_id: Optional[UUID]` (line 17, iter-2)
- Input models (`MemoryCreate`, `SessionCreate`) still enforce `agent_id: UUID` — API endpoints require non-null for new records.

**Note:** The `AliasChoices("agent_id", "agentId")` addition is purely additive input flexibility — no security bypass vector.

---

### N-01: `agent_id` was non-optional (from iter-1) — **→ RESOLVED (Informational)**
**Files:** `memory.py:18`, `session.py:17`

The finding from iteration 1 noted that `agent_id` remained non-optional while `session_id` was fixed. The agent-id-optional bug contract in iteration 2 has resolved this — both models now have `agent_id: Optional[UUID]`.

---

### N-02 (New): Agent_id null acceptance not explicitly tested — **Informational**
**Files:** `tests/models/test_memory.py`, `tests/models/test_session.py`

No test explicitly validates that `Memory()` or `Session()` constructs successfully with `agent_id=None` (or with agent_id omitted entirely). All existing tests provide `agent_id=uuid.uuid4()`. While the change is structurally identical to the well-tested `session_id` Optional change from iteration 1, the null-acceptance path lacks dedicated coverage.

**Recommendation:** Add a test that constructs `Memory(content="test", session_id=uuid.uuid4())` without `agent_id` and asserts `agent_id is None`. Same for `Session(project="test")`.

**Severity justification:** Not a security vulnerability — the Pydantic type system provides defense at the model level. This is a regression-risk gap, not an exploit path.
**Status:** NEW — informational finding.

---

## 03 · Security-Critical Code Highlights

```python
# memory.py:18 — RESOLVED: agent_id now Optional[UUID] (was required UUID)
agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("agent_id", "agentId"))
# ← Defensive: if Rust omits agentId, field becomes None instead of raising ValidationError
# ← AliasChoices accepts both snake_case and camelCase input (safe, additive)
```

```python
# session.py:17 — RESOLVED: agent_id now Optional[UUID] (was required UUID)
agent_id: Optional[UUID] = Field(default=None, validation_alias=AliasChoices("agent_id", "agentId"))
# ← Same defensive pattern as Memory model
```

```python
# memory.py:42 — UNCHANGED: MemoryCreate still requires agent_id
class MemoryCreate(BaseModel):
    session_id: UUID
    agent_id: UUID  # ← Still required for API creates — correct
```

```python
# bridge.py:78-83 — NEW: os.path.expanduser call (safe, hardcoded path)
expanded_path = os.path.expanduser(path)  # ← SAFE: ~ expansion only, path is not user-controlled
```

```python
# memory.py:38-45 — NEW: role default test (no security concern)
def test_role_default_is_system(self):
    mem = Memory(session_id=..., agent_id=..., content="...")
    assert mem.role == "system"
```

---

## 04 · Remediation Recommendations

> **Must Fix**
> None. No critical or high-severity findings identified.

> **Should Fix**
> 1. **Add null agent_id acceptance tests** (N-02, Informational): Add test cases that construct Memory and Session models without `agent_id` and verify `agent_id is None`. Prevents regression if the Optional pattern is accidentally reverted.
2. **Document `embedding` field privacy implications** (F-01, Low): Add a note in API docs that embedding vectors are exposed and should be handled as sensitive data.

> **Consider**
> 1. **Add datetime timezone validation** (F-02, Informational): If the Rust engine can emit timezone-naive datetimes, add a Pydantic `field_validator` to coerce them to UTC.
2. **Frontend session status mapping** (F-03, Informational): The `"completed"` → `"done"` mismatch is documented as out-of-scope but should be tracked.

---

_Generated by Security Architect (Iteration 2) · 2026-07-26 · Validation Contract: 2026-07-26-fix-data-api-design-tokens_
