# SPEC Compliance Review Report

# Fix Data API + Align Design Tokens

> Two independent work packages: (1) Pydantic model fixes to accept Rust camelCase output via `validation_alias`, (2) CSS design token alignment with V2-DEEP design system.

**Verdict:** FAIL (class: fail)

2026-07-26 · 11/12 requirements matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

| REQ | Description | Status |
|-----|-------------|--------|
| REQ-001 | Memory model accepts Rust output via `validation_alias` | ✅ MATCHED |
| REQ-002 | New fields on Memory (`embedding`, `tags`, `version`, `updated_at`) | ✅ MATCHED |
| REQ-003 | `role` field made optional with default `"system"` | ✅ MATCHED |
| REQ-004 | Orphan fields preserved (`tokens`, `tokenizer`, `model`, `metadata`) | ✅ MATCHED |
| REQ-005 | Session model accepts Rust output (`turn_count`, `duration_ms`, `efficiency_score`, `last_active`) | ✅ MATCHED |
| REQ-006 | Session `status` accepts Rust values (plain `str` field) | ✅ MATCHED |
| REQ-007 | Session incompatible fields made optional (`name`, `completed_at`) | ✅ MATCHED |
| REQ-008 | `started_at` maps from `createdAt` via `validation_alias` | ✅ MATCHED |
| REQ-009 | Design tokens match V2-DEEP spec exactly | ⚠️ PARTIAL |
| REQ-010 | Missing token groups added (shadows, gradients, chart, motion, layout, type scale, semantic bg) | ✅ MATCHED |
| REQ-011 | Surface card tokens added | ✅ MATCHED |
| REQ-012 | Dashboard page updated to use new token names | ❌ UNMATCHED |

**Summary:** 11/12 matched, 1 unmatched, 1 partial

---

## 02 · Implementation Mapping

### REQ-001 — Memory model accepts Rust output via validation_alias

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `model_config = ConfigDict(populate_by_name=True)` | `models/memory.py` | 12 | Enables both alias and name-based population |
| `session_id: UUID = Field(validation_alias="sessionId")` | `models/memory.py` | 14 | Aliases Rust `sessionId` |
| `agent_id: UUID = Field(validation_alias="agentId")` | `models/memory.py` | 15 | Aliases Rust `agentId` |
| `memory_type: str = Field(validation_alias="memoryType")` | `models/memory.py` | 16 | Aliases Rust `memoryType` |
| `created_at: datetime = Field(validation_alias="createdAt")` | `models/memory.py` | 34-37 | Aliases Rust `createdAt` |
| `updated_at: datetime = Field(validation_alias="updatedAt")` | `models/memory.py` | 38-41 | Aliases Rust `updatedAt` |

### REQ-002 — New fields on Memory

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `embedding: Optional[list[float]] = None` | `models/memory.py` | 19 | Present |
| `tags: list[str] = Field(default_factory=list)` | `models/memory.py` | 20 | Present |
| `version: int = Field(default=1)` | `models/memory.py` | 21 | Present |
| `updated_at: datetime = Field(validation_alias="updatedAt")` | `models/memory.py` | 38-41 | Present |

### REQ-003 — Role field made optional

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `role: Optional[str] = Field(default="system")` | `models/memory.py` | 17 | Present |

### REQ-004 — Orphan fields preserved

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `tokens: Optional[int] = None` | `models/memory.py` | 22 | Present |
| `tokenizer: Optional[str] = None` | `models/memory.py` | 23 | Present |
| `model: Optional[str] = None` | `models/memory.py` | 24 | Present |
| `metadata: dict = Field(default_factory=dict)` | `models/memory.py` | 42 | Present |

### REQ-005 — Session model accepts Rust output

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `agent_id: UUID = Field(validation_alias="agentId")` | `models/session.py` | 15 | Aliases Rust `agentId` |
| `turn_count: int = Field(validation_alias="turnCount")` | `models/session.py` | 19 | Aliases Rust `turnCount` |
| `duration_ms: int = Field(validation_alias="durationMs")` | `models/session.py` | 20 | Aliases Rust `durationMs` |
| `efficiency_score: Optional[float] = Field(validation_alias="efficiencyScore")` | `models/session.py` | 21-22 | Aliases Rust `efficiencyScore` |
| `last_active: datetime = Field(validation_alias="lastActive")` | `models/session.py` | 31-33 | Aliases Rust `lastActive` |
| `model_config = ConfigDict(populate_by_name=True)` | `models/session.py` | 12 | `populate_by_name` enabling alias support |

### REQ-006 — Session status accepts Rust values

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `status: str = Field(default="active")` | `models/session.py` | 18 | Plain `str` — accepts any string including `"active"`, `"completed"`, `"error"` |

### REQ-007 — Session incompatible fields made optional

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `name: Optional[str] = Field(None, max_length=512)` | `models/session.py` | 17 | Already Optional with None default |
| `completed_at: Optional[datetime] = None` | `models/session.py` | 34 | Already Optional with None default |

### REQ-008 — started_at maps from createdAt

| Field | File | Line | Evidence |
|-------|------|------|----------|
| `started_at: datetime = Field(validation_alias="createdAt")` | `models/session.py` | 26-29 | Maps to Rust `createdAt` |

### REQ-009 — Design tokens match V2-DEEP spec

| Token | Expected (V2-DEEP) | Actual (tokens.css) | Match |
|-------|-------------------|---------------------|-------|
| `--color-bg-base` / `--bg-base` | `#181716` | `#181716` | ✅ |
| `--color-bg-elevated` / `--bg-elevated` | `#1F1E1D` | `#1F1E1D` | ✅ |
| `--color-accent` / `--accent` | `#7C5CFC` | `#7C5CFC` | ✅ |
| `--color-text-primary` / `--text-primary` | `#F2F0EE` | `#F2F0EE` | ✅ |
| `--color-text-secondary` / `--text-secondary` | `#A09E9B` | `#A09E9B` | ✅ |
| `--color-text-tertiary` / `--text-tertiary` | `#6F6D6B` | `#6F6D6B` | ✅ |
| `--color-bg-hover` / `--bg-hover` | `#252423` | `#252423` | ✅ |
| `--color-bg-active` / `--bg-active` | `#2A2928` | `#2A2928` | ✅ |
| `--color-bg-inset` / `--bg-inset` | `#131211` | `#131211` | ✅ |
| `--color-accent-hover` / `--accent-hover` | `#6A4DE0` | `#6A4DE0` | ✅ |
| `--color-status-success` / `--status-success` | `#4CAF50` | `#4CAF50` | ✅ |
| `--color-status-warning` / `--status-warning` | `#FF9800` | `#FF9800` | ✅ |
| `--color-status-error` / `--status-error` | `#F44336` | `#F44336` | ✅ |
| `--color-status-info` / `--status-info` | `#42A5F5` | `#42A5F5` | ✅ |
| `--shadow-sm` | `0 1px 2px rgba(0,0,0,0.3)` | `0 1px 2px rgba(0, 0, 0, 0.3)` | ⚠️ (format) |
| `--shadow-md` | `0 4px 12px rgba(0,0,0,0.4)` | `0 4px 12px rgba(0, 0, 0, 0.4)` | ⚠️ (format) |
| `--shadow-lg` | `0 8px 30px rgba(0,0,0,0.5)` | `0 8px 30px rgba(0, 0, 0, 0.5)` | ⚠️ (format) |

**Hex values: all match exactly.**
**Shadow rgba values: functionally identical (CSS ignores whitespace inside rgba), but V2-DEEP spec has no spaces after commas while tokens.css has spaces. Marked as PARTIAL due to formatting divergence from literal spec.**

### REQ-010 — Missing token groups added

Located in `tokens.css` `:root` block (lines 109-205):

| Group | Tokens | Lines | Status |
|-------|--------|-------|--------|
| Shadows | `--shadow-sm`, `--shadow-md`, `--shadow-lg`, `--shadow-accent` | 166-169 | ✅ |
| Gradients | `--gradient-card`, `--gradient-accent`, `--gradient-accent-glow` | 172-174 | ✅ |
| Chart colors | `--chart-1` through `--chart-8`, `--chart-grid`, `--chart-axis`, `--chart-zero` | 177-187 | ✅ |
| Motion | `--ease-out`, `--ease-in-out`, `--duration-fast`, `--duration-normal`, `--duration-slow` | 190-194 | ✅ |
| Layout | `--max-content-width`, `--sidebar-width`, `--sidebar-collapsed`, `--topbar-height` | 197-200 | ✅ |
| Type scale | `--text-xs` through `--text-3xl` (+ line-height, font-weight) | 68-108 | ✅ |
| Semantic bg | `--bg-status-success`, `--bg-status-warning`, `--bg-status-error`, `--bg-status-info` | 157-160 | ✅ |

### REQ-011 — Surface card tokens added

| Token | File | Line | Evidence |
|-------|------|------|----------|
| `--color-surface-card` / `--surface-card` | `tokens.css` | `@theme` line 52, `:root` line 163 | ✅ |
| `--color-surface-card-alt` / `--surface-card-alt` | `tokens.css` | `@theme` line 53, `:root` line 164 | ✅ |
| `--color-surface-card-hover` / `--surface-card-hover` | `tokens.css` | `@theme` line 54, `:root` line 165 | ✅ |
| `--color-surface-card-accent` / `--surface-card-accent` | `tokens.css` | `@theme` line 55, `:root` line 166 | ✅ |

### REQ-012 — Dashboard page updated

**File:** `contexter-web/src/pages/Dashboard/DashboardPage.tsx`
**Git diff:** No changes — **zero modifications** to this file.

DashboardPage.tsx still uses these token names that no longer exist in the new tokens.css:
- `bg-surface` (line 135, 231) — requires `--color-surface` which was replaced by `--color-surface-card`
- `border-border` (line 135, 188, 231) — requires `--color-border` which was replaced by `--color-border-subtle` and `--color-border-default`
- `bg-accent/10`, `bg-accent/15` (lines 233) — no `--color-accent` exists for opacity utility in `@theme` (accent is now in `@theme` as `--color-accent`, so this might still work)

Additionally, the `statusVariant` map (line 16-21) does not include `"completed"`, so sessions with `status: "completed"` (as returned by Rust) will not have a matching Badge variant. This was noted as out of scope in the design preview (RQ-004) but REQ-012 requires the dashboard to "reference the updated token names."

**Status: ❌ UNMATCHED** — No implementation changes found in DashboardPage.tsx.

---

## 03 · Unmatched Requirements

### ❌ REQ-012 — Dashboard page updated

**Severity:** High

**What SPEC says:** "DashboardPage.tsx SHALL reference the updated token names where applicable."

**What the code does:** DashboardPage.tsx was **not modified**. It still references old token names (`bg-surface` for `--color-surface`, `border-border` for `--color-border`) that are **no longer defined** in the new `tokens.css`. The old `--color-surface` was replaced by `--color-surface-card` and `--color-border` was replaced by `--color-border-subtle` and `--color-border-default`.

**Impact:** Components using `bg-surface`, `border-border` will lose their styling because the underlying CSS custom properties no longer exist in the `@theme` block. The HTML `@layer base` block was updated to use new flat aliases (`--bg-base`, `--text-primary`, `--border-default`), but the DashboardPage.tsx component file was not touched.

**Fix required:** Update DashboardPage.tsx to use `bg-surface-card` instead of `bg-surface`, and `border-border-subtle` or `border-border-default` instead of `border-border`. Alternatively, add backward-compatible aliases in `tokens.css` for the old names.

---

## 04 · Partially Matched Requirements

### ⚠️ REQ-009 — Design tokens match V2-DEEP spec exactly

**Severity:** Low

**Issue:** Shadow values (`--shadow-sm`, `--shadow-md`, `--shadow-lg`) in `tokens.css` use extra whitespace in `rgba()` parameters compared to the V2-DEEP spec:

| Spec | tokens.css |
|------|-----------|
| `rgba(0,0,0,0.3)` | `rgba(0, 0, 0, 0.3)` |
| `rgba(0,0,0,0.4)` | `rgba(0, 0, 0, 0.4)` |
| `rgba(0,0,0,0.5)` | `rgba(0, 0, 0, 0.5)` |

**Impact:** None functionally — CSS parsers treat `rgba(0,0,0,0.3)` and `rgba(0, 0, 0, 0.3)` identically. This is a formatting difference only.

**All hex values** (14+ spot-checked) match the V2-DEEP spec exactly: `#181716`, `#1F1E1D`, `#252423`, `#2A2928`, `#131211`, `#F2F0EE`, `#A09E9B`, `#6F6D6B`, `#7C5CFC`, `#6A4DE0`, `#4CAF50`, `#FF9800`, `#F44336`, `#42A5F5`, `#9B82FF` — all correct.

---

## 05 · Constraint Violations

No CON-XXX constraints were defined in SPEC.md. No constraint violations to report.

---

## 06 · Edge Case Verification

| Edge Case | Covered by Implementation | Covered by Tests | Notes |
|-----------|--------------------------|-----------------|-------|
| EC-001: `memoryType: "UnknownType"` | ✅ `memory_type` is plain `str` with `default="fact"` | ❌ No test feeds camelCase `"memoryType": "UnknownType"` to validate alias works | Field accepts any string |
| EC-002: `embedding: [...]` (1536 floats) | ✅ `embedding: Optional[list[float]]` | ❌ No test validates list[float] from alias input | Structurally OK |
| EC-003: `embedding: null` | ✅ `Optional[list[float]] = None` | ❌ No test validates None from alias input | OK |
| EC-004: `sessionId: null` | ⚠️ `session_id: UUID` (not Optional) | ❌ No test | Will raise ValidationError — per SPEC, this is expected behavior |
| EC-005: `tags: "not_a_list"` | ✅ `tags: list[str]` — will reject | ❌ No test | Acceptable — Rust always returns array |
| EC-006: Datetime without timezone | ⚠️ `datetime` field — behavior depends on Pydantic | ❌ No test | Should be verified |
| EC-007: Unknown fields ignored | ✅ `model_config` default is `extra="ignore"` in Pydantic v2 | ❌ No test | OK |
| EC-008: `status: "paused"` | ✅ `status: str` — accepts any string | ✅ `test_session_with_all_fields` uses `"paused"` | OK |
| EC-013: Old token name references | ❌ DashboardPage.tsx still uses old names (REQ-012) | N/A | Unmatched |
| EC-014: Frontend session `"done"` vs `"completed"` | ⚠️ Noted as out of scope in design preview | ❌ No fix applied | Documented carryover |

**Key gap:** No test files (neither `test_memory.py` nor `test_session.py`) test the `validation_alias` behavior with camelCase JSON input. The existing tests construct models with snake_case keyword arguments, which work via `populate_by_name=True` but do **not** verify the primary bug fix — that camelCase Rust JSON is accepted. This is a test coverage gap, not a SPEC gap per se, but it means the fix is not verified by tests.

---

## 07 · Carryover Check

| Check | Result |
|-------|--------|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | NO |
| Zero findings are being silently deferred to a future iteration | NO |

**Explanation:** REQ-012 (DashboardPage.tsx not updated) is an unmatched requirement. It has not been given a bug contract. The out-of-scope items (frontend memory type enum alignment, session status `"completed"`→`"done"` mapping) are documented in the design preview as deferred but that was a planning decision, not a bug contract. The carryover declaration requires explicit bug contracts for every finding — none have been created yet.

---

## 08 · Summary

> **SPEC Compliance Assessment**
> The implementation covers 11 of 12 SPEC requirements. All Pydantic model changes (REQ-001 through REQ-008) are correctly implemented with proper `validation_alias` usage and `ConfigDict(populate_by_name=True)`. All design token groups from V2-DEEP (REQ-009 through REQ-011) are present with correct hex values. However, DashboardPage.tsx (REQ-012) was not updated, leaving old token name references (`bg-surface`, `border-border`) that no longer resolve in the new tokens.css.

> **Findings**
> 1. **REQ-012 UNMATCHED (High):** DashboardPage.tsx was not modified; still uses old token names `bg-surface` and `border-border` that are no longer defined in `tokens.css`. Components using these classes will not style correctly.
> 2. **REQ-009 PARTIAL (Low):** Shadow `rgba()` values have minor whitespace formatting differences from the V2-DEEP spec (spaces after commas). Functionally equivalent but not a literal match.

---

## 09 · Final Verdict

| Criterion | Result |
|-----------|--------|
| All REQ-XXX matched with implementation code | ❌ (11/12) |
| All CON-XXX constraints respected | ✅ (no CON-XXX defined) |
| All EDGE_CASES covered by implementation or tests | ⚠️ (EC-013 uncovered, no test for validation_alias) |
| Carryover declaration clean | ❌ |
| **Overall** | **FAIL** |

---

_Generated by SPEC Compliance Validator · 2026-07-26 · Validation Contract: fix-data-api-design-tokens_
