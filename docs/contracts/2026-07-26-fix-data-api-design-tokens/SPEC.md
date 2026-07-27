---
title: Fix Data API + Align Design Tokens
version: 1.0
date_created: 2026-07-26
tags: pydantic, api, design-tokens, css
---

# Specification: Fix Data API + Align Design Tokens

## 1. Purpose & Scope

This specification addresses two independent defects:

1. **Data API returns empty responses**: The FastAPI endpoints `/api/v1/memories` and `/api/v1/sessions` return `[]` because Pydantic `Memory` and `Session` models cannot validate data returned by the Rust `contexter_core` engine. The Rust engine serialises fields in camelCase (e.g. `sessionId`, `agentId`, `memoryType`) while Pydantic expects snake_case (`session_id`, `agent_id`). Additionally, the Pydantic models require fields (`role`, `name`, `updated_at`, `completed_at`) that the Rust engine does not return, and are missing Rust fields (`embedding`, `tags`, `version`, `updatedAt`, `turnCount`, `durationMs`, `efficiencyScore`, `lastActive`).

2. **Design tokens do not match the approved V2-DEEP design system**: The CSS custom properties in `contexter-web/src/styles/tokens.css` have wrong hex values (e.g. `#1e1d1c` instead of `#1F1E1D`), missing tokens (shadows, gradients, chart colors, motion, layout dimensions), and use different naming conventions than the approved `V2-DEEP-design-system.md`.

## 2. Definitions

| Term | Definition |
|------|------------|
| `validation_alias` | Pydantic v2 mechanism to accept alternate field names during model validation without changing the Python attribute name |
| V2-DEEP | The approved dark design system for Contexter (documented at `docs/design/V2-DEEP-design-system.md`) |
| Token | A CSS custom property (`--*`) defining a design value (color, spacing, shadow, etc.) |
| Bridge | The `StorageEngine` class in `contexter-server/src/contexter_server/core/bridge.py` that wraps Rust Engine calls via `asyncio.to_thread` |
| RFC 3339 | The datetime format used by Rust's `chrono::DateTime<Utc>` serialization |

## 3. Requirements

### REQ-001 — Memory model accepts Rust output
The Pydantic `Memory` model SHALL accept all fields returned by the Rust engine. Fields with camelCase names in the Rust JSON SHALL use Pydantic v2 `validation_alias` to map to Python snake_case attribute names.

### REQ-002 — Missing Rust fields added to Memory model
The `Memory` model SHALL include: `embedding` (optional `list[float]`), `tags` (default `[]`), `version` (default `1`), `updated_at` (RFC 3339 datetime).

### REQ-003 — Role field made optional
The `role` field SHALL be optional with default `"system"`, since the Rust engine does not emit a `role` field.

### REQ-004 — Orphan fields preserved for backward compatibility
Existing Pydantic fields not emitted by Rust (`tokens`, `tokenizer`, `model`, `metadata`) SHALL remain with optional defaults so existing callers are not broken.

### REQ-005 — Session model accepts Rust output
The Pydantic `Session` model SHALL accept all fields returned by the Rust engine. This includes adding `turn_count` (aliased from `turnCount`), `duration_ms` (aliased from `durationMs`), `efficiency_score` (aliased from `efficiencyScore`), and `last_active` (aliased from `lastActive`).

### REQ-006 — Session status alignment
The `status` field SHALL accept the Rust enum values `"active"`, `"completed"`, `"error"` as valid strings.

### REQ-007 — Session model removes incompatible fields
The `name` and `completed_at` fields SHALL be removed from the Session model (or made optional with `None` default) since the Rust engine does not provide them.

### REQ-008 — Session creation fields preserved
`started_at` SHALL map to Rust's `createdAt` via `validation_alias`. Fields needed for `SessionCreate` and `SessionPatch` input models SHALL remain unchanged.

### REQ-009 — Design tokens match V2-DEEP spec exactly
Every color, spacing, border-radius, shadow, gradient, chart color, motion, and layout dimension token in `tokens.css` SHALL match the values defined in `V2-DEEP-design-system.md`.

### REQ-010 — Missing design tokens added
The following token groups SHALL be added to `tokens.css`, all absent from the current file:
- Shadows: `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-accent`
- Gradients: `--gradient-card`, `--gradient-accent`, `--gradient-accent-glow`
- Chart colors: `--chart-1` through `--chart-8`, `--chart-grid`, `--chart-axis`, `--chart-zero`
- Motion: `--ease-out`, `--ease-in-out`, `--duration-fast`, `--duration-normal`, `--duration-slow`
- Layout: `--max-content-width`, `--sidebar-width`, `--sidebar-collapsed`, `--topbar-height`
- Type scale: `--text-xs` through `--text-3xl`
- Semantic backgrounds: `--bg-status-success`, `--bg-status-warning`, `--bg-status-error`, `--bg-status-info`

### REQ-011 — Surface card tokens added
`--surface-card`, `--surface-card-alt`, `--surface-card-hover`, `--surface-card-accent` SHALL be added per the V2-DEEP spec.

### REQ-012 — Dashboard page token usage updated
`DashboardPage.tsx` SHALL reference the updated token names where applicable. No structural redesign unless the approved design preview requires it.

## 4. Interfaces & Data Contracts

### Memory API Response (after fix)

```json
{
  "id": "uuid",
  "session_id": "uuid",
  "agent_id": "uuid",
  "memory_type": "fact",
  "content": "string",
  "embedding": null,
  "tags": [],
  "version": 1,
  "created_at": "2026-07-26T10:00:00Z",
  "updated_at": "2026-07-26T10:00:00Z",
  "role": "system",
  "tokens": null,
  "tokenizer": null,
  "model": null,
  "metadata": {}
}
```

### Session API Response (after fix)

```json
{
  "id": "uuid",
  "agent_id": "uuid",
  "project": "string",
  "status": "active",
  "turn_count": 5,
  "duration_ms": 12345,
  "efficiency_score": null,
  "metadata": {},
  "started_at": "2026-07-26T10:00:00Z",
  "last_active": "2026-07-26T10:12:00Z"
}
```

## 5. Acceptance Criteria

### AC-001 — Memory list returns data
Given the Rust engine has stored memories, When `GET /api/v1/memories` is called, Then the response SHALL be a non-empty JSON array of memory objects.

### AC-002 — Memory response has correct shape
Given a memory is returned by the API, When inspecting the response, Then each object SHALL contain all fields from the Pydantic `Memory` model with correct types.

### AC-003 — Session list returns data
Given the Rust engine has stored sessions, When `GET /api/v1/sessions` is called, Then the response SHALL be a non-empty JSON array of session objects.

### AC-004 — Session response has correct shape
Given a session is returned by the API, When inspecting the response, Then each object SHALL contain all fields from the Pydantic `Session` model with correct types.

### AC-005 — No validation errors in logs
Given the API endpoints are called, When the server processes the Rust engine output, Then no `ValidationError` exceptions SHALL appear in server logs.

### AC-006 — Token values match V2-DEEP spec
Given the approved V2-DEEP-design-system.md, When comparing against `tokens.css`, Then every color, shadow, gradient, spacing, radius, and motion value SHALL match exactly.

### AC-007 — All required token groups present
Given the V2-DEEP-design-system.md lists shadows, gradients, chart colors, motion, layout, and semantic background tokens, When checking `tokens.css`, Then each group SHALL be present.

## 6. Dependencies

### Affected Files

| File | Package |
|------|---------|
| `contexter-server/src/contexter_server/models/memory.py` | Backend |
| `contexter-server/src/contexter_server/models/session.py` | Backend |
| `contexter-web/src/styles/tokens.css` | Frontend |
| `contexter-web/src/pages/Dashboard/DashboardPage.tsx` | Frontend |

## 7. Validation Criteria

- `curl http://localhost:8051/api/v1/memories` returns non-empty array
- `curl http://localhost:8051/api/v1/sessions` returns non-empty array
- `tokens.css` values match V2-DEEP-design-system.md (spot-check 5 random values)
- `tokens.css` contains all token groups listed in V2-DEEP-design-system.md
