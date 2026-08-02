# SPEC — Count Endpoints: Document Estimate Semantics

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Finding: **PF-11** (Performance Benchmarker, `review-mcp-live-fix-scrutiny-performance-review-iter-3.md`)

## Problem

The `rocksdb.estimate-num-keys` fast path (added for `count_sessions` in iter-3, and already used by `count_agents`/`count_skills` since PF-09) counts memtable **update history**: exact on freshly seeded stores, but INFLATED after updates/deletes until compaction. Live measurements: 100 creates → 100; +100 updates → 200 (2×); +50 deletes → 150 (3×); after `flush()` → still 170 vs 60 actual (no correction). End-to-end: `get_overview` after 100 creates + turn-count updates → `total_sessions = 210` vs 100 actual (2.1×); `contexter status` prints this directly.

Sessions are the highest-mutation-frequency entity (every conversation turn updates the row; cleanup deletes sessions), so inflation is far more likely to be observed than for agents/skills. **Zero documentation** currently mentions this caveat (README Design Decisions has 0 mentions).

## Requirements

### REQ-ED-001 — README documentation
The README "Design Decisions → Accepted performance decisions" section SHALL document the `estimate-num-keys` semantics for the count endpoints (`count_sessions`, `count_agents`, `count_skills`): exact on freshly seeded stores; inflates after updates/deletes (memtable history) until compaction; `flush()` does NOT correct it; exactness remains available via filtered counts or `list_sessions`/`list_agents`/`list_skills` (bounded at 100 — note the tradeoff).

### REQ-ED-002 — Architecture spec documentation
`docs/design/specs/2026-07-23-contexter-system-architecture.md` §7.5 (or the count-endpoints section) SHALL carry the same caveat, consistent with REQ-ED-001.

### REQ-ED-003 — Concrete numbers
The documentation SHALL include the measured inflation behavior (e.g., 100 creates + 100 updates → count 200 vs 100 actual) so readers can judge magnitude.

### REQ-ED-004 — No behavior change
Documentation-only contract: no code, no engine, no test behavior changes (existing EC-CS-003 semantics stand).

## Non-Goals

- No compaction trigger implementation (documented as a follow-up option, not required here).
- No change to the estimate fast path.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-count-estimate-docs/`
- References: `plan/review/review-mcp-live-fix-scrutiny-performance-review-iter-3.md` (PF-11, measurements)
