# Edge Cases — Fix Data API + Align Design Tokens

## Category 1: Pydantic Model Validation

| ID | Scenario | Likelihood | Impact | Expected Behavior |
|----|----------|------------|--------|-------------------|
| EC-001 | Rust returns `memoryType: "UnknownType"` not in the allowed set | Low | High | Pydantic str field accepts any string; no validation error. Frontend may need to handle unknown types. |
| EC-002 | Rust returns `embedding: [...]` with 1536 floats | Medium | Low | Field accepts `list[float]`; no issue. |
| EC-003 | Rust returns `embedding: null` | High | Low | Optional field accepts `None`; no issue. |
| EC-004 | Rust returns `sessionId: null` (foreign key missing) | Low | High | `UUID` field with `None` → Pydantic validation error. Should be Optional[UUID] if nullable. |
| EC-005 | Rust returns `tags: "not_a_list"` (wrong type) | Low | Medium | `list[str]` rejects string → ValidationError. Acceptable — Rust always returns array. |
| EC-006 | Rust returns datetime without timezone | Low | Medium | Pydantic `datetime` accepts naive ISO and treats as UTC if `datetime.fromisoformat` succeeds. Verify behavior. |
| EC-007 | Rust returns new field not in Pydantic model | Medium | Low | Pydantic v2 ignores extras by default (`model_config` has `extra="ignore"`). No error. |
| EC-008 | Session `status` is `"paused"` (not a Rust enum value) | Low | Medium | Current frontend uses `"paused"` but Rust only emits `"active"\|"completed"\|"error"`. Pydantic accepts any string. |
| EC-009 | Multiple concurrent reads of memory/session lists | Medium | Low | The bridge runs in ThreadPoolExecutor; each call is independent. No shared state. |

## Category 2: Design Token Application

| ID | Scenario | Likelihood | Impact | Expected Behavior |
|----|----------|------------|--------|-------------------|
| EC-010 | Tailwind v4 `@theme` block uses non-standard token names | Medium | Medium | Tailwind v4 `@theme` only generates utilities for `--color-*`, `--spacing-*`, etc. Flat names like `--bg-elevated` need to be used directly via `var(--bg-elevated)` in CSS. |
| EC-011 | Gradient token referenced in CSS but not in `@theme` | High | Low | Gradients can't be Tailwind utilities but work fine as CSS custom properties referenced directly. Document as "not in @theme, use var() directly". |
| EC-012 | Browser does not support `oklch` or modern color syntax | Low | Low | All tokens use hex and `rgba()`. No modern color syntax used. |
| EC-013 | Existing components reference old token names after rename | High | Medium | After renaming `--color-bg-secondary` → `--bg-elevated`, all existing components using the old name will lose styling. Fix: keep old names as aliases OR update all references. |

## Category 3: API Contract Stability

| ID | Scenario | Likelihood | Impact | Expected Behavior |
|----|----------|------------|--------|-------------------|
| EC-014 | Frontend expects `status: "done"` but Rust returns `"completed"` | High | Medium | The Pydantic model does not transform enum values. The frontend `Badge` variant mapping uses `'active'\|'done'\|'error'\|'paused'`. Rust returns `"completed"` not `"done"`. This mismatch persists after the Pydantic fix. Dashboard badges for completed sessions will show no matching variant. |
| EC-015 | Frontend Memory type expects `"conversation"\|"decision"\|"pattern"\|"reference"\|"custom"` but Rust returns `"fact"\|"preference"\|"procedure"\|"context"\|"episode"` | High | Medium | Neither API nor Pydantic model transforms these values. The Memory Explorer page may not display types correctly. This is a separate frontend concern not in scope. |
