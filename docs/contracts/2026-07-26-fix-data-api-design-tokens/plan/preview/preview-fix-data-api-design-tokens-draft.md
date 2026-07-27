# Fix Data API + Align Design Tokens — Design Draft

> **Status:** `DRAFT — Pending Review` · **Version:** `v1.0-draft`
> **Scope:** 2 Independent Work Packages · 0 Open Questions (design is determined)

---

## Navigation

- [Problem](#problem)
- [Work Package A — Pydantic Model Fixes](#wp-a)
- [Work Package B — Design Token Alignment](#wp-b)
- [Data Flow](#dataflow)
- [API Contract](#api)
- [Design Token Mapping](#tokens)
- [Scope](#scope)
- [AC](#ac)
- [Edge Cases](#edgecases)
- [Summary](#summary)

---

## Why This Fix Exists {#problem}

| The Bug | The Fix |
|---|---|
| `GET /api/v1/memories` returns `[]` despite 194 memories in the Rust engine because `Memory.model_validate(r)` raises `ValidationError` for every record — the Pydantic model expects `session_id` but Rust sends `sessionId`, requires `role` which doesn't exist, and is missing `memoryType`, `embedding`, `tags`, `version`, `updatedAt` | Align Pydantic models with what Rust actually serialises using `validation_alias` for camelCase fields and optional defaults for fields the Rust layer doesn't emit |
| `tokens.css` uses different hex values, missing token groups (shadows, gradients, chart colors, motion, layout), and different naming conventions than the approved `V2-DEEP-design-system.md` | Replace token values with spec-correct values, add all missing token groups, and align naming conventions |

---

## Work Package A — Pydantic Model Fixes {#wp-a}

### Memory Model — Field Mapping

**Rust `Memory` struct** (what the engine emits, camelCase JSON):

```
id            Uuid
sessionId     Uuid
agentId       Uuid
memoryType    "fact"|"preference"|"procedure"|"context"|"episode"
content       String
embedding     Option<Vec<f32>>
tags          Vec<String>     (default [])
version       u32             (default 1)
createdAt     DateTime<Utc>
updatedAt     DateTime<Utc>
```

**Pydantic `Memory` model** (what the API returns, snake_case JSON):

| Field | Type | Changed? | Alias | Default |
|-------|------|----------|-------|---------|
| `id` | UUID | — | — | `uuid4` |
| `session_id` | UUID | ✅ Add alias | `"sessionId"` | — |
| `agent_id` | UUID | ✅ Add alias | `"agentId"` | — |
| `memory_type` | str | ✅ Add alias | `"memoryType"` | `"fact"` |
| `content` | str | — | — | — |
| `embedding` | `Optional[list[float]]` | ✅ **New** | — | `None` |
| `tags` | `list[str]` | ✅ **New** | — | `[]` |
| `version` | int | ✅ **New** | — | `1` |
| `created_at` | datetime | ✅ Add alias | `"createdAt"` | `now()` |
| `updated_at` | datetime | ✅ **New** | `"updatedAt"` | `now()` |
| `role` | `Optional[str]` | ✅ Made optional | — | `"system"` |
| `tokens` | `Optional[int]` | Kept (backward compat) | — | `None` |
| `tokenizer` | `Optional[str]` | Kept (backward compat) | — | `None` |
| `model` | `Optional[str]` | Kept (backward compat) | — | `None` |
| `metadata` | dict | Kept (backward compat) | — | `{}` |

### Session Model — Field Mapping

**Rust `Session` struct** (camelCase JSON):

```
id              Uuid
project         String
agentId         Uuid
status          "active"|"completed"|"error"
turnCount       u32
durationMs      u64
efficiencyScore Option<f64>
metadata        serde_json::Value
createdAt       DateTime<Utc>
lastActive      DateTime<Utc>
```

**Pydantic `Session` model** (what the API returns):

| Field | Type | Changed? | Alias | Default |
|-------|------|----------|-------|---------|
| `id` | UUID | — | — | `uuid4` |
| `agent_id` | UUID | ✅ Add alias | `"agentId"` | — |
| `project` | str | — | — | — |
| `status` | str | — | — | `"active"` |
| `turn_count` | int | ✅ **New** | `"turnCount"` | `0` |
| `duration_ms` | int | ✅ **New** | `"durationMs"` | `0` |
| `efficiency_score` | `Optional[float]` | ✅ **New** | `"efficiencyScore"` | `None` |
| `metadata` | dict | — | — | `{}` |
| `started_at` | datetime | ✅ Changed alias | `"createdAt"` | `now()` |
| `last_active` | datetime | ✅ **New** | `"lastActive"` | `now()` |
| `name` | `Optional[str]` | ✅ Made optional | — | `None` |
| `completed_at` | `Optional[datetime]` | ✅ Made optional | — | `None` |
| `updated_at` | datetime | Kept (backward compat) | — | `now()` |

### Key Design Decisions

| ID | Decision | Choice | Rationale |
|----|----------|--------|-----------|
| D-A1 | Use `validation_alias` not `serialization_alias` | `validation_alias` only | Accept camelCase from Rust; serialize snake_case for API consumers. Frontend `types.ts` already uses snake_case. |
| D-A2 | Keep orphan fields with defaults | Keep, don't delete | `tokens`, `tokenizer`, `model`, `metadata` are used by `MemoryCreate`/`MemoryPatch` input models and existing callers. Making them optional with `None`/`{}` defaults preserves backward compatibility. |
| D-A3 | `role` default `"system"` | Optional with default | Rust doesn't emit `role`. The field exists for user→agent→system role routing. Defaulting to `"system"` is safe for imported rekal data. |

---

## Work Package B — Design Token Alignment {#wp-b}

### Strategy

Replace the current `tokens.css` `@theme` block with one matching V2-DEEP spec values, keeping Tailwind v4 compatibility. Add all missing token groups as CSS custom properties (some inside `@theme` for Tailwind utility generation, some as flat `:root` vars for direct `var()` reference).

### Token Group Summary

| Group | Current State | Action |
|-------|--------------|--------|
| **Background colors** | 6 tokens, wrong hex values | Replace all hex values |
| **Accent colors** | 4 tokens, close values | Replace to match spec |
| **Text colors** | 4 tokens, slightly off | Replace hex values |
| **Borders** | 3 tokens, wrong values + missing | Replace + add `--border-subtle`, `--border-default`, `--border-accent` |
| **Spacing** | 6 tokens, `--spacing-xs` naming | Add `--space-1` through `--space-16` per spec |
| **Border radius** | 4 tokens, wrong values | Replace |
| **Shadows** | **MISSING** | Add `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-accent` |
| **Gradients** | **MISSING** | Add `--gradient-card`, `--gradient-accent`, `--gradient-accent-glow` |
| **Chart colors** | **MISSING** | Add `--chart-1` through `--chart-8`, grid, axis, zero-line |
| **Motion** | **MISSING** | Add easing curves and durations |
| **Layout** | **MISSING** | Add sidebar widths, topbar height, max content width |
| **Type scale** | **MISSING** | Add `--text-xs` through `--text-3xl` |
| **Semantic backgrounds** | **MISSING** | Add `--bg-status-success/warning/error/info` |
| **Surface cards** | **MISSING** | Add `--surface-card` variants |
| **Status colors** | 6 tokens, wrong hex values | Replace hex values to match spec |

### Token Naming Convention

Tokens inside `@theme` use Tailwind's namespaced format (`--color-*`, `--spacing-*`, etc.) for utility generation. Tokens outside `@theme` use the flat V2-DEEP names (`--bg-elevated`, `--shadow-md`, etc.) and are referenced via `var()` in CSS.

```
@theme {
  /* Tailwind-visible tokens */
  --color-bg-base: #181716;
  --color-bg-elevated: #1F1E1D;
  --color-accent: #7C5CFC;
  --spacing-1: 4px;
  --spacing-4: 16px;
  /* etc — all Tailwind utility generators */
}

:root {
  /* Custom tokens for direct var() use */
  --bg-base: var(--color-bg-base);
  --bg-elevated: var(--color-bg-elevated);
  --shadow-sm: 0 1px 2px rgba(0,0,0,0.3);
  --gradient-card: linear-gradient(135deg, #1F1E1D 0%, #1D1C1B 100%);
  --chart-1: #7C5CFC;
  /* etc */
}
```

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

### `GET /api/v1/memories` (after fix)

**Response 200**
```json
[
  {
    "id": "018f8b70-1234-7abc-def0-123456789abc",
    "session_id": "018f8b70-5678-7def-abcd-987654321fed",
    "agent_id": "018f8b70-9abc-7def-0123-456789abcdef",
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

### `GET /api/v1/sessions` (after fix)

**Response 200**
```json
[
  {
    "id": "018f8b70-1234-7abc-def0-123456789abc",
    "agent_id": "018f8b70-5678-7def-abcd-987654321fed",
    "project": "contexter",
    "status": "active",
    "turn_count": 5,
    "duration_ms": 12345,
    "efficiency_score": null,
    "metadata": {},
    "started_at": "2026-07-26T10:00:00Z",
    "last_active": "2026-07-26T10:12:00Z",
    "name": null,
    "completed_at": null,
    "updated_at": "2026-07-26T10:12:00Z"
  }
]
```

---

## Design Token Mapping {#tokens}

### Color Value Corrections (Before → After)

| Token | CSS Name | Old Value | V2-DEEP Value | Status |
|-------|----------|-----------|---------------|--------|
| `--bg-base` | `--color-bg-primary` | `#181716` | `#181716` | ✅ Already correct |
| `--bg-elevated` | `--color-bg-secondary` | `#1e1d1c` | `#1F1E1D` | 🔶 Fix |
| `--bg-hover` | `--color-bg-tertiary` | `#242322` | `#252423` | 🔶 Fix |
| `--bg-active` | `--color-bg-hover` | `#2a2928` | `#2A2928` | ✅ Already correct |
| `--bg-inset` | **MISSING** | — | `#131211` | 🆕 Add |
| `--text-primary` | `--color-text-primary` | `#f5f4f2` | `#F2F0EE` | 🔶 Fix |
| `--text-secondary` | `--color-text-secondary` | `#a09e9c` | `#A09E9B` | 🔶 Fix |
| `--text-tertiary` | `--color-text-tertiary` | `#73716e` | `#6F6D6B` | 🔶 Fix |
| `--accent-muted` | `--color-accent-muted` | `rgba(124,92,252,0.15)` | `#7C5CFC20` | 🔶 Fix |
| `--status-success` | `--color-success` | `#22c55e` | `#4CAF50` | 🔶 Fix |
| `--status-error` | `--color-error` | `#ef4444` | `#F44336` | 🔶 Fix |
| `--status-warning` | `--color-warning` | `#f59e0b` | `#FF9800` | 🔶 Fix |
| `--status-info` | `--color-info` | `#3b82f6` | `#42A5F5` | 🔶 Fix |

---

## Out of Scope {#scope}

| # | Item | Rationale |
|---|---|---|
| 01 | Frontend Memory type enum alignment (Rust: `fact|preference|procedure|context|episode` vs frontend: `conversation|decision|pattern|reference|custom`) | Separate concern. The frontend `types.ts` defines its own enum regardless of API response. A future task should align these. |
| 02 | Frontend Session status mapping (Rust: `completed` vs frontend: `done`) | DashboardPage `statusVariant` map doesn't include `"completed"`. A separate frontend fix. |
| 03 | Full DashboardPage redesign with charts | The approved design preview's dashboard wireframe already matches current implementation structurally. Token fix is sufficient for this task. |
| 04 | Adding new API endpoints | No new endpoints. Only fixing existing ones to return data. |
| 05 | Replacing Rust data with new seed data | Existing data is valid. The fix is in the Pydantic parsing layer only. |

---

## Acceptance Criteria {#ac}

> **Status:** 10 Pending

| ID | Description | Status |
|---|---|---|
| AC-001 | Memory list returns non-empty array `GET /api/v1/memories` | 🔶 Pending |
| AC-002 | Memory response has all required fields | 🔶 Pending |
| AC-003 | Session list returns non-empty array `GET /api/v1/sessions` | 🔶 Pending |
| AC-004 | Session response has all required fields | 🔶 Pending |
| AC-005 | No `ValidationError` in server logs | 🔶 Pending |
| AC-006 | Token hex values match V2-DEEP spec | 🔶 Pending |
| AC-007 | All required token groups present (shadows, gradients, charts, motion, layout, type scale, semantic bg, surface cards) | 🔶 Pending |
| AC-008 | Memory list handles empty engine gracefully → `[]` | 🔶 Pending |
| AC-009 | Session list handles empty engine gracefully → `[]` | 🔶 Pending |
| AC-010 | Unknown Rust fields silently ignored (no crash on unexpected keys) | 🔶 Pending |

---

## Edge Cases {#edgecases}

> **Status:** 14 Identified

| ID | Scenario | Expected Behavior | Priority |
|---|---|---|---|
| EC-001 | `memoryType: "UnknownType"` | Accepted as string; frontend may need to handle | Low |
| EC-002 | `embedding: [float, ...]` 1536-dim array | Accepted as `list[float]` | Medium |
| EC-003 | `embedding: null` | Accepted as `None` | High |
| EC-004 | `sessionId: null` foreign key missing | Would fail UUID validation — should be `Optional[UUID]` | Medium |
| EC-005 | `tags: "not_a_list"` wrong type | Pydantic rejects → `ValidationError`. Acceptable — Rust always returns array. | Low |
| EC-006 | Datetime without timezone | Accepted as naive ISO. Pydantic may or may not treat as UTC. Verify. | Medium |
| EC-007 | New unknown field added by Rust engine | Silently ignored via `extra="ignore"` (Pydantic v2 default) | Medium |
| EC-008 | Session status `"paused"` (not a Rust enum value) | Accepted as string (Pydantic field is `str`). Not a Rust bug — frontend uses `"paused"`. | Low |
| EC-009 | Concurrent reads of memory/session lists | Bridge uses ThreadPoolExecutor; each call independent. No shared state. | Low |
| EC-010 | Tailwind v4 `@theme` only generates utilities for `--color-*`, `--spacing-*` | Flat tokens (`--bg-elevated`) must be used via `var()` in CSS, not Tailwind classes | Medium |
| EC-011 | Gradient tokens can't generate Tailwind utilities | Used as `var(--gradient-card)` in CSS directly | Medium |
| EC-012 | Old token name references in components after rename | Components referencing `--color-bg-secondary` will break if renamed to `--color-bg-elevated`. Approach: keep old names as aliases or update all references. | High |
| EC-013 | Frontend expects `status: "done"` but Rust returns `"completed"` | Dashboard badges won't match. Not fixed in this scope. | Medium |
| EC-014 | Frontend Memory `memory_type` enum values differ from Rust | Memory Explorer page may show wrong type labels. Not fixed in this scope. | Medium |

---

## Design Draft Summary {#summary}

| Metric | Count |
|---|---|
| Acceptance Criteria | 10 |
| Edge Cases | 14 |
| Work Packages | 2 (Backend + Frontend) |
| Files Changed | 3 (memory.py, session.py, tokens.css) |
| Token Groups to Add | 8 (shadows, gradients, charts, motion, layout, type scale, semantic bg, surface cards) |

This draft covers two independent fixes. Both are ready for implementation — no open questions on approach.

---

**Generated · 2026-07-26 · Contexter Fix Data API + Design Tokens · v1.0-draft**
