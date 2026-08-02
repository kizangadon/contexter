# SPEC — Estimate Fast Path: Document CF Invariant

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Finding: **Code Reviewer F-5 (INFO)** — `review-mcp-live-fix-scrutiny-code-review-iter-3.md`

## Problem

The `count_sessions` estimate fast path has no prefix guard — it relies on the invariant that **the sessions CF holds only session keys** (index entries live in the separate `session_index` CF). This invariant is true today and documented in the bug contract (EC-CS-003), but the code itself carries no comment pointing to it. A future change that stores other data in the sessions CF would silently corrupt unfiltered counts.

## Requirements

### REQ-EIC-001 — Invariant comment
A concise code comment SHALL be added at the count_sessions estimate fast path (and, for consistency, at the count_agents/count_skills estimate paths if they lack one) stating: unfiltered counts via `estimate-num-keys` are valid ONLY because the CF holds exclusively entity keys (index entries live in the companion `*_index` CF); if that invariant breaks, unfiltered counts must not use the estimate.

### REQ-EIC-002 — No behavior change
Comment-only contract. No logic, signature, or test changes.

## Non-Goals

- No runtime guard/assert (documented invariant, matching count_agents/count_skills precedent).
- No changes to other files.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-estimate-invariant-comment/`
- References: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-3.md` (F-5)
