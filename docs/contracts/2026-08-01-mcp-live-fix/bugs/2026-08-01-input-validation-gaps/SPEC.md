# SPEC — Input Validation Gaps Repair

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-input-validation-gaps

## Problem
Live validation gaps vs the frozen contract:
- `store_memory` accepts and persists **empty content** (EC-006 violated).
- `export_data` accepts unsupported format → returns `completed` (EC-012 violated).
- `limit` edge values (0, negative, huge) not clamped (EC-009 violated).
- No size caps on `store_memory` content / `search_memories` query; error paths echo unbounded client input (Security finding).

## Requirements
- REQ-IV-001: `store_memory` rejects empty/whitespace-only `content` with a structured error.
- REQ-IV-002: `export_data` validates `format` against supported set; unsupported → structured error.
- REQ-IV-003: `limit` clamped to sane bounds (≥0, ≤max, default preserved).
- REQ-IV-004: Size caps on content/query with structured errors when exceeded.
- REQ-IV-005: Error messages do not echo unbounded client input.
- REQ-IV-006: TDD reproduction tests for each; full suite green (≥647/1 pre-existing).

## Constraints
Auth unchanged. DDD applies. Frozen contract parameter names unchanged.
