# Fix Data API + Align Design Tokens — Approved Contract

> **Status:** `APPROVED — Contract Frozen` · **Version:** `v1.0-approved`
> **Scope:** 2 Independent Work Packages · 0 Open Questions

---

## Navigation

- [Context](#context)
- [Work Package A — Pydantic Model Fixes](#wp-a)
- [Work Package B — Design Token Alignment](#wp-b)
- [Data Flow](#dataflow)
- [API Contract](#api)
- [Design Token Mapping](#tokens)
- [Resolved Decisions](#decisions)
- [Out of Scope](#scope)
- [Acceptance Criteria](#ac)
- [Edge Cases](#edgecases)
- [Validation Contract Artifacts](#contract)
- [Summary](#summary)

---

## Why This Fix Exists {#context}

| The Pain | The Fix |
|---|---|
| `GET /api/v1/memories` returns `[]` despite 194 memories in the Rust engine. `Memory.model_validate(r)` raises `ValidationError` for every record — Pydantic expects `session_id` but Rust sends `sessionId`, requires `role` which doesn't exist in Rust output, and is missing `memoryType`, `embedding`, `tags`, `version`, `updatedAt` fields. | ✅ Align Pydantic models with Rust's actual serialisation using `validation_alias` for camelCase fields and optional defaults for fields the Rust layer doesn't emit. |
| `tokens.css` uses different hex values, missing 8+ token groups (shadows, gradients, chart colors, motion, layout dimensions, type scale, semantic backgrounds, surface cards), and different naming conventions than the approved `V2-DEEP-design-system.md`. | ✅ Replace token values with spec-correct values, add all missing token groups, and align naming. |

---

## Work Package A — Pydantic Model Fixes {#wp-a}

> **Status:** `APPROVED`

### Memory Model

```python
class Memory(BaseModel):
    id: UUID = Field(default_factory=uuid4)
    session_id: Optional[UUID] = Field(default=None, validation_alias="sessionId")
    agent_id: Optional[UUID] = Field(default=None, validation_alias="agentId")
    memory_type: str = Field(default="fact", validation_alias="memoryType")
    content: str
    embedding: Optional[list[float]] = None
    tags: list[str] = Field(default_factory=list)
    version: int = Field(default=1)
    role: Optional[str] = Field(default="system")
    tokens: Optional[int] = None
    tokenizer: Optional[str] = None
    model: Optional[str] = None
    metadata: dict = Field(default_factory=dict)
    created_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="createdAt",
    )
    updated_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="updatedAt",
    )
```

### Session Model

```python
class Session(BaseModel):
    id: UUID = Field(default_factory=uuid4)
    agent_id: Optional[UUID] = Field(default=None, validation_alias="agentId")
    project: str = Field(..., min_length=1, max_length=256)
    status: str = Field(default="active")
    turn_count: int = Field(default=0, validation_alias="turnCount")
    duration_ms: int = Field(default=0, validation_alias="durationMs")
    efficiency_score: Optional[float] = Field(default=None, validation_alias="efficiencyScore")
    metadata: dict = Field(default_factory=dict)
    started_at: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="createdAt",
    )
    last_active: datetime = Field(
        default_factory=lambda: datetime.now(timezone.utc),
        validation_alias="lastActive",
    )
    name: Optional[str] = Field(default=None, max_length=512)
    completed_at: Optional[datetime] = None
    updated_at: datetime = Field(default_factory=lambda: datetime.now(timezone.utc))
```

### Decision Log

| ID | Decision | Rationale |
|----|----------|-----------|
| **D-A1** | Use `validation_alias` input only; serialize snake_case | Accept camelCase from Rust; output snake_case for API consumers. Frontend `types.ts` uses snake_case. |
| **D-A2** | Keep orphan fields with `None`/`{}` defaults | `tokens`, `tokenizer`, `model`, `metadata` are used by `MemoryCreate`/`MemoryPatch`. Removing them would break input models. |
| **D-A3** | `role` defaults to `"system"` | Rust doesn't emit `role`. Default is safe for imported rekal data. |
| **D-A4** | `started_at` aliases from `createdAt` | Rust has `createdAt` (when session was created), which maps to `started_at` in the domain model. |
| **D-A5** | `session_id` and `agent_id` are `Optional[UUID]` with `default=None` | Defensive measure against null values from Rust engine. Prevents `ValidationError` when Rust emits `null` for these foreign-key fields. |

---

## Work Package B — Design Token Alignment {#wp-b}

> **Status:** `APPROVED`

### Strategy

Inside `@theme`: Tailwind-namespaced tokens (`--color-*`, `--spacing-*`) for utility generation.
In `:root`: flat V2-DEEP names (`--bg-elevated`, `--shadow-md`, etc.) referencing the theme values.

### Token Changes

| Token | Old Value | New Value |
|-------|-----------|-----------|
| `--color-bg-secondary` → `--color-bg-elevated` | `#1e1d1c` | `#1F1E1D` |
| `--color-bg-tertiary` → `--color-bg-hover` | `#242322` | `#252423` |
| `--color-bg-hover` → `--color-bg-active` | `#2a2928` | `#2A2928` |
| New: `--color-bg-inset` | — | `#131211` |
| `--color-text-primary` | `#f5f4f2` | `#F2F0EE` |
| `--color-text-secondary` | `#a09e9c` | `#A09E9B` |
| `--color-text-tertiary` | `#73716e` | `#6F6D6B` |
| `--color-success` | `#22c55e` | `#4CAF50` |
| `--color-error` | `#ef4444` | `#F44336` |
| `--color-warning` | `#f59e0b` | `#FF9800` |
| `--color-info` | `#3b82f6` | `#42A5F5` |

### Token Groups Added

| Group | Tokens |
|-------|--------|
| ✅ Shadows | `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-accent` |
| ✅ Gradients | `--gradient-card`, `--gradient-accent`, `--gradient-accent-glow` |
| ✅ Chart colors | `--chart-1` through `--chart-8`, `--chart-grid`, `--chart-axis`, `--chart-zero` |
| ✅ Motion | `--ease-out`, `--ease-in-out`, `--duration-fast`, `--duration-normal`, `--duration-slow` |
| ✅ Layout | `--max-content-width`, `--sidebar-width`, `--sidebar-collapsed`, `--topbar-height` |
| ✅ Type scale | `--text-xs` through `--text-3xl` |
| ✅ Semantic bg | `--bg-status-success`, `--bg-status-warning`, `--bg-status-error`, `--bg-status-info` |
| ✅ Surface cards | `--surface-card`, `--surface-card-alt`, `--surface-card-hover`, `--surface-card-accent` |
| ✅ Border tokens | `--border-subtle`, `--border-default`, `--border-accent` |
| ✅ Spacing (spec naming) | `--space-1` through `--space-16` |

---

## Data Flow {#dataflow}

### Before Fix (Broken)

```
Rust engine → JSON (camelCase) → Pydantic Memory.model_validate(r)
  → ValidationError: "session_id field required" (no alias for "sessionId")
  → ValidationError: "role field required" (no "role" in Rust output)
  → Exception silently caught → memory skipped → returns []
```

### After Fix

```
Rust engine → JSON (camelCase) → Pydantic Memory.model_validate(r)
  → validation_alias="sessionId" maps to session_id → OK
  → role defaults to "system" → OK
  → embedding/tags/version default to None/[]/1 → OK
  → unknown fields silently ignored (extra="ignore") → OK
  → Memory object created → appended to list → returns [Memory, Memory, ...]
```

---

## API Contract {#api}

> **Status:** `FINAL`

### `GET /api/v1/memories`

**Response 200**
```json
[
  {
    "id": "018f8b70-1234-7abc-def0-123456789abc",
    "session_id": "018f8b70-5678-7def-abcd-987654321fed",
    "agent_id": null,
    "memory_type": "fact",
    "content": "User prefers Ruff over Black for formatting",
    "embedding": null,
    "tags": ["python", "formatting"],
    "version": 1,
    "role": "system",
    "tokens": null,
    "tokenizer": null,
    "model": null,
    "metadata": {},
    "created_at": "2026-07-26T10:00:00Z",
    "updated_at": "2026-07-26T10:00:00Z"
  }
]
```

### `GET /api/v1/sessions`

**Response 200**
```json
[
  {
    "id": "018f8b70-1234-7abc-def0-123456789abc",
    "agent_id": null,
    "project": "contexter",
    "status": "active",
    "turn_count": 5,
    "duration_ms": 12345,
    "efficiency_score": null,
    "metadata": {},
    "started_at": "2026-07-26T10:00:00Z",
    "last_active": "2026-07-26T12:00:00Z",
    "name": null,
    "completed_at": null,
    "updated_at": "2026-07-26T12:00:00Z"
  }
]
```

---

## Resolved Decisions {#decisions}

| ID | Question | Resolution |
|----|----------|------------|
| RQ-001 | Should we delete orphan fields (`tokens`, `tokenizer`, `model`)? | **Keep with `None` defaults.** These are used by `MemoryCreate`/`MemoryPatch` input models. Removing them would be a breaking change. |
| RQ-002 | Should `name` and `completed_at` be removed from Session? | **Make optional with `None` default.** Keeps backward compat while allowing Rust data to validate. |
| RQ-003 | One `@theme` block or split into `@theme` + `:root`? | **Both.** `@theme` for Tailwind utility generation; `:root` for flat V2-DEEP names. The flat names reference the theme values via `var()`. |
| RQ-004 | Should frontend session status `"completed"` → `"done"` mapping be fixed? | **Out of scope.** The `statusVariant` map in `DashboardPage.tsx` lacks `"completed"`. A separate frontend-only fix. |

---

## Out of Scope {#scope}

| # | Item | Rationale |
|---|---|---|
| 01 | Frontend Memory type enum alignment (Rust: `fact\|preference\|procedure\|context\|episode` vs frontend: `conversation\|decision\|pattern\|reference\|custom`) | Separate concern. Frontend `types.ts` defines its own enum. Future task. |
| 02 | Frontend Session status mapping (Rust: `"completed"` vs frontend: `"done"`) | `DashboardPage.tsx` `statusVariant` map doesn't include `"completed"`. A separate frontend-only fix. Will cause badge rendering gaps for completed sessions. |
| 03 | Full DashboardPage redesign with charts | The approved design preview's dashboard wireframe matches current implementation structurally. Token fix is sufficient. |
| 04 | Adding new API endpoints | No new endpoints. Only fixing existing ones to return data. |
| 05 | Replacing Rust data with new seed data | Existing data is valid. Fix is in the Pydantic parsing layer only. |

---

## Acceptance Criteria {#ac}

> **Status:** 10 Approved

| ID | Description | Status |
|---|---|---|
| AC-001 | `GET /api/v1/memories` returns non-empty array (194 memories) | ✅ |
| AC-002 | Each memory object has all required fields: `id`, `session_id`, `agent_id`, `memory_type`, `content`, `tags`, `version`, `created_at`, `updated_at` | ✅ |
| AC-003 | `GET /api/v1/sessions` returns non-empty array (1 session) | ✅ |
| AC-004 | Each session object has: `id`, `agent_id`, `project`, `status`, `turn_count`, `duration_ms`, `started_at`, `last_active` | ✅ |
| AC-005 | No `ValidationError` in server logs when calling endpoints | ✅ |
| AC-006 | Token hex values match V2-DEEP spec exactly (spot-check 5) | ✅ |
| AC-007 | All 8 missing token groups present in `tokens.css` | ✅ |
| AC-008 | Empty engine returns `[]` not error | ✅ |
| AC-009 | Unknown Rust fields silently ignored | ✅ |
| AC-010 | `status: "completed"` accepted as valid string | ✅ |

---

## Edge Cases {#edgecases}

> **Status:** 14 Identified · 3 Marked Out of Scope

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-001 | `memoryType: "UnknownType"` | Accepted as string | Low |
| EC-002 | `embedding: [float, ...]` 1536-dim array | Accepted as `list[float]` | Medium |
| EC-003 | `embedding: null` | Accepted as `None` | High |
| EC-004 | `sessionId: null` (foreign key) | UUID validation fails — suggested: `Optional[UUID]` | Medium |
| EC-005 | `tags: "not_a_list"` (wrong type) | Rejected → `ValidationError`. Rust always returns array. | Low |
| EC-006 | Datetime without timezone | Accepted as naive ISO. Monitor behavior. | Medium |
| EC-007 | New unknown field added by Rust | Silently ignored (`extra="ignore"`) | Medium |
| EC-008 | Status `"paused"` (non-Rust value) | Accepted as string (Pydantic field is `str`) | Low |
| EC-009 | Concurrent reads | Bridge uses ThreadPoolExecutor — independent calls | Low |
| EC-010 | Flat tokens vs Tailwind `@theme` | Flat names need `var()` in CSS, not Tailwind classes | Medium |
| EC-011 | Gradient tokens not in `@theme` | Used via `var()` — documented | Medium |
| EC-012 | Old token name references in components | Keep old `--color-*` names as aliases alongside new flat names | High |
| EC-013 | Frontend status `"done"` vs Rust `"completed"` | **Out of scope** (see #02) | Medium |
| EC-014 | Frontend `memory_type` enum mismatch | **Out of scope** (see #01) | Medium |

---

## Validation Contract Artifacts {#contract}

| Artifact | Path |
|----------|------|
| SPEC.md | `docs/contracts/2026-07-26-fix-data-api-design-tokens/SPEC.md` |
| ACCEPTANCE.md | `docs/contracts/2026-07-26-fix-data-api-design-tokens/ACCEPTANCE.md` |
| EDGE_CASES.md | `docs/contracts/2026-07-26-fix-data-api-design-tokens/EDGE_CASES.md` |
| Design Preview | `docs/contracts/2026-07-26-fix-data-api-design-tokens/plan/preview/preview-fix-data-api-design-tokens-approved.md` |

---

## Summary {#summary}

| Metric | Count |
|---|---|
| Acceptance Criteria | 10 |
| Edge Cases | 14 |
| Work Packages | 2 (Backend + Frontend) |
| Files Changed | 3 (memory.py, session.py, tokens.css) |
| Token Groups Added | 8 (shadows, gradients, charts, motion, layout, type scale, semantic bg, surface cards) |
| Color Values Corrected | 10 |
| Out of Scope Items | 5 |

---

**Generated · 2026-07-26 · Contexter Fix Data API + Design Tokens · v1.0-approved**

**APPROVED — All design decisions frozen. Proceed to BUILD.**
