# SPEC — count_memories Estimate Invariant Comment

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 6 (persisted from iter-5)
> Finding: **Code Reviewer [LOW]** — `review-mcp-live-fix-scrutiny-code-review-iter-5.md`

## Problem

In `contexter-core/src/storage/rocksdb.rs`, the estimate fast paths for `count_sessions`, `count_agents`, and `count_skills` each carry a comment documenting the invariant that makes the estimate valid ONLY because the CF holds exclusively entity keys with index entries living in the companion `*_index` CF. `count_memories` (estimate fast path, ~lines 1029-1047) lacks that same caveat, despite using the identical `estimate_num_keys` mechanism. Documentation asymmetry.

## Requirements

### REQ-IV-001 — Add invariant comment on count_memories fast path
The `memories` estimate fast path SHALL carry the same validity caveat as its sibling count functions: the estimate is exact on a fresh DB, inflated after updates/deletes, and valid ONLY because the `memories` CF holds exclusively entity keys (index entries in the companion `*_index` CF). Copy the sibling phrasing, adapted to `memories`.

### REQ-IV-002 — No behavior change
Comment-only change. No logic, no layout, no other file changes.

### REQ-IV-003 — Consistent sibling parity
The three existing sibling comments remain untouched; the new comment matches their style/terminology.

## Non-Goals
- No change to the estimate mechanism (`estimate_num_key`) or its call sites.
- No change to any other CF comment.

## Artifacts
- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-count-memories-invariant-comment/`
- Reference: `plan/review/review-mcp-live-fix-scrutiny-code-review-iter-5.md`