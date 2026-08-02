# SPEC — Unfiltered `count_sessions` O(1) Fast Path

> Parent contract: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Source finding: **PF-10** (Performance Benchmarker, `plan/review/review-mcp-live-fix-scrutiny-performance-review-iter-2.md` §03)
> Verdict: CONDITIONAL PASS — 1 NEW LOW finding

## Problem

`count_sessions` in `contexter-core/src/storage/rocksdb.rs:691-727` has an index-prefix fast path ONLY for `project` filters. The unfiltered call that `get_overview` makes (`count_sessions({})`) falls through to a **full scan of the sessions CF with serde deserialization per row**.

Empirical (live engine): 0.078 ms empty → 2.538 ms @ 2,000 sessions (linear scaling; ~125 ms projected at 100k sessions). It is ~70% of the 3.63 ms `get_overview` call and the **last O(store) element** on the analytics-overview path after the PF-09 fix.

The sessions CF holds only session keys (index entries live in the separate `session_index` CF, rocksdb.rs:538-540, :655-683), so the same `rocksdb.estimate-num-keys` O(1) fast path already added for `count_agents` (rocksdb.rs:1156) and `count_skills` (rocksdb.rs:1335) is directly applicable.

## Requirements

### REQ-CS-001 — Unfiltered fast path
Unfiltered `count_sessions` (no `project` filter) SHALL use the `rocksdb.estimate-num-keys` CF-property fast path on the sessions CF, mirroring the `count_agents`/`count_skills` implementation at rocksdb.rs:1156/:1335. The full scan SHALL remain only as the fallback when the property is unavailable (same semantics as the existing fast paths).

### REQ-CS-002 — Filtered path unchanged
`count_sessions` WITH a `project` filter SHALL keep its existing index-prefix scan semantics exactly — no behavior change for filtered counts.

### REQ-CS-003 — API surface unchanged
The engine method signature, Python bridge wrapper (`core/bridge.py` count_sessions), and `AnalyticsService.get_overview` call SHALL be unchanged. No new parameters, no new dispatch path. The documented estimate-error semantics of `estimate-num-keys` (matching REQ-ACE-001 pattern and `test_bridge_live_coverage.py` docstring) apply.

### REQ-CS-004 — Tests
Rust correctness tests SHALL verify unfiltered count parity (count matches seeded sessions, empty → 0, filtered counts still exact). Python-side regression tests SHALL verify `get_overview` session counts remain correct (e.g., 12-session store → 12). Performance validation SHALL demonstrate the unfiltered count is flat across store growth (no per-row deserialization).

## Non-Goals

- No change to `count_agents`/`count_skills`/`count_memories` (already O(1) or unchanged).
- No pagination, no secondary index for filtered counts, no schema changes.
- No Python service restructuring.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-count-sessions-fast-path/`
- References: `plan/review/review-mcp-live-fix-scrutiny-performance-review-iter-2.md` (B1/B3, PF-10), `contexter-core/src/storage/rocksdb.rs` (:691-727, :1156, :1335)
