# Performance Review Report

# MCP Live-Functionality Repair — Scrutiny: Performance Review (Auto Bug Loop Iteration 3)

> Performance re-review of the ENTIRE feature scope in the working tree (HEAD 27e031d, uncommitted): the two NEW iter-3 bug contracts — count-sessions-fast-path (REQ-CS-001..004: unfiltered count_sessions O(1) estimate fast path) and fastmcp-framework-logging (REQ-FL-001..005: bounded failure stderr, perf-neutral success path) — plus re-verification of every prior finding (PF-01..PF-10) against the REBUILT wheel (maturin build --release 08:38, installed .so mtime == wheel mtime). Baseline reports review-mcp-live-fix-scrutiny-performance-review.md, ...-iter-1.md, ...-iter-2.md are immutable references.

**Verdict:** CONDITIONAL PASS — PF-10 (count_sessions fast path) and the fastmcp-logging perf-neutrality requirement are RESOLVED and verified with live measurements against the rebuilt wheel; 1 NEW LOW finding (PF-11: estimate-num-keys inflation is user-visible through get_overview/CLI status after session mutations, and flush does not correct it) (class: SCRUTINY/PERFORMANCE — static analysis + live-engine and live-MCP runtime verification against rebuilt wheel)

2026-08-02 · 9 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| count_sessions growth (direct engine, median 300) | 0.0015 ms (0) → 0.0013 ms (250) → 0.0005 ms (500) → 0.0019 ms (1000) → 0.0014 ms (2000) — FLAT, sub-2µs; exact parity on fresh store at every size (est == truth) |
| count_sessions growth (bridge-level, median 50) | 0.2351 ms (0) → 0.1966 ms (250) → 0.1653 ms (2000) — FLAT; vs iter-2 baseline 2.538 ms @ 2000 → ~15–17× improvement; full-scan contrast 8.77 ms |
| Filtered counts (index-prefix scan) | Exact at every size: p-0 filter 36/36, 72/72, 143/143, 286/286 — REQ-CS-002 preserved |
| Semantic inflation probe (estimate vs truth) | 100 creates → 100/100 exact; +100 updates → 200/100 (2×); +50 deletes → 150/50 (3×); mixed → 170/60; after flush() → 170/60 — FLUSH DOES NOT CORRECT |
| get_overview end-to-end after mutations | 100 sessions + turn-count updates → overview reports 210 vs 100 actual (2.1×); CLI `contexter status` prints this number directly (status_commands.py:43) |
| PF-09 re-verification (count_agents/count_skills) | Flat: count_agents 0.0015→0.0012 ms, count_skills 0.0015→0.0012 ms, count_memories 0.0013→0.0011 ms across growth — no regression |
| FastMCP filter overhead (per record) | ~200 ns median (success INFO 211 ns; error-call records 184–211 ns) — negligible, only on fastmcp loggers; success-path stderr 0 bytes |
| Filter idempotency & survival | Exactly 1 filter per logger (flag-guarded); survives fastmcp configure_logging (handlers removed, filters kept) |
| Test suites | 881 Python passed (baseline 867 + new) · Rust agent_skill 16 passed · session 9 passed · framework EFS 32 passed incl. success-path + diagnostics |

> **Analysis Scope**
> Full working-tree review (HEAD 27e031d, all changes uncommitted). Files: contexter-core/src/storage/rocksdb.rs (count_sessions estimate fast path at :715-731), contexter-server/src/contexter_server/fastmcp_logging.py (NEW), __init__.py (:48-54), cli/status_commands.py (:43), services/analytics_service.py (:91-118), core/bridge.py (count_sessions :296-298), tests (agent_skill_test.rs, session_test.rs, test_bridge_live_coverage.py, test_framework_efs_stderr.py). Contracts: count-sessions-fast-path (REQ-CS-001..004, AC-CS-001..006, EC-CS-001..007), fastmcp-framework-logging (REQ-FL-001..005, AC-FL-001..006, EC-FL-001..007). Verification: live Rust-engine benches (0/250/500/1000/2000 sessions, 300-sample medians), bridge-level benches, semantic inflation probes, get_overview end-to-end, live FastMCP success-path stderr capture, filter micro-benchmark (20k samples), full test suites. All benches run from /tmp/opencode; stores deleted after use.

---

## 02 · Benchmark Results

**B1 — count_sessions growth flatness (direct engine, rebuilt wheel): RESOLVED.** Median of 300 calls per size: 0 sessions → 0.0015 ms; 250 → 0.0013 ms; 500 → 0.0005 ms; 1000 → 0.0019 ms; 2000 → 0.0014 ms. Latency is FLAT (sub-2µs) — no store-size scaling, confirming the `rocksdb.estimate-num-keys` property path is taken (rocksdb.rs:715-731). Exact parity on fresh stores at every size (est == truth == 0/250/500/1000/2000) with 50 agents + 50 skills interleaved, proving CF isolation. Filtered counts (project filter, index-prefix scan) stay exact: p-0 returns 36/36, 72/72, 143/143, 286/286 vs full-scan ground truth (REQ-CS-002 / AC-CS-003). AC-CS-001, AC-CS-002, AC-CS-004 all verified.

**B2 — Bridge-level (get_overview path): RESOLVED.** Median of 50 calls through `StorageEngine.count_sessions` (thread pool): 0.2351 ms (empty) → 0.1966 ms (250) → 0.1653 ms (2000). Flat, with the ~0.15–0.24 ms floor set by bridge dispatch overhead, not the count itself. vs iter-2 baseline 2.538 ms @ 2000 sessions → ~15–17× improvement; the eliminated full scan costs 8.77 ms at 2000 sessions (list_sessions ground truth, 5-call median). AC-CS-004's sub-millisecond target met.

**B3 — Fallback preserved (AC-CS-006).** The `.ok().flatten()` → `parse::<u64>()` → fall-through-to-scan pattern is byte-identical to count_agents/count_skills (rocksdb.rs:1156/:1335). Property unavailable or unparseable → exact full scan. Never panics, never silently returns a wrong type.

**B4 — Semantic inflation probe (the estimate's accepted behavior, EC-CS-003).** Fresh store, 100 creates → estimate 100 / truth 100 (exact). +100 update_session calls → estimate 200 / truth 100 (2× — estimate counts memtable update history). +50 deletes → estimate 150 / truth 50 (3×). Mixed → estimate 170 / truth 60. Critically: `engine.flush()` does NOT correct the estimate (170/60 after flush — the memtable is written to an SST that still holds multiple versions of the same keys until compaction; no compaction trigger is exposed in the engine API, only flush/checkpoint). Agents/skills precedent identical: 1 update → count 2 vs truth 1 (same semantics accepted in PF-09).

**B5 — get_overview / CLI status exposure (the assessed concern).** End-to-end `AnalyticsService.get_overview` after 100 session creates + 100 turn-count updates: `total_sessions = 210` vs 100 actual (2.1× inflation). The CLI `contexter status` prints `Sessions: {overview.total_sessions}` (status_commands.py:43) — the inflated number is user-visible. Sessions are the highest-mutation-frequency entity (turnCount/lastActive/durationMs updated on every conversation turn; deletes on cleanup), so the inflation is far more likely to be observed than agents/skills. Documented as accepted semantics (EC-CS-003) and consistent with the PF-09 precedent — but the CLI/overview surface gives users no indication the number is an estimate.

**B6 — FastMCP filter perf-neutrality (REQ-FL-004): RESOLVED.** Micro-benchmark (20k samples): the `_SuppressFrameworkTracebackBox.filter` costs ~200 ns median per record (success INFO 211 ns; error-call records 184–211 ns) — a single `getMessage().startswith()` check against 3 prefixes. The filter runs ONLY on the fastmcp emitter loggers (`fastmcp`, `fastmcp.server`, `fastmcp.server.server`) and only when a record is emitted; the MCP tool-call path itself has zero added work. Success-path stderr measured at **0 bytes** through the live FastMCP client path (3 successful calls, `capfd` capture) — no error records, no box, no Traceback. AC-FL-005's perf-neutrality and stderr-purity requirements met.

**B7 — Filter idempotency and survival: POSITIVE.** Exactly 1 filter per logger (attribute-guarded), verified across all three emitter loggers. Filter survives `fastmcp.utilities.logging.configure_logging` (which removes handlers only) — verified by re-running configure_logging after installation. Failure-path tests (32 in test_framework_efs_stderr.py incl. concurrent failures) all pass with ≤512-char bounded stderr and no box.

**B8 — get_overview correctness (AC-CS-005): RESOLVED.** test_bridge_live_coverage.py: 12-session store → `count_sessions()` == 12, `count_sessions({})` == 12, filtered == 4, `get_overview().total_sessions == 12` (fresh store, estimate exact). Full suite green: 881 Python + 16 agent_skill + 9 session Rust tests.

**B9 — Prior findings re-verification (PF-01..PF-09): no regressions.** PF-01 session limit pushdown intact (list_sessions filtered path applies filter.limit at :585-586). PF-02 single UTF-8 encode bytes path present (bridge `_LARGE_CONTENT_THRESHOLD`). PF-03 `CONTEXTER_BRIDGE_POOL_SIZE` canonicalization present. PF-05 per-call logging at DEBUG present (`bridge_call_end` at DEBUG). PF-06/07/08 documented in README Design Decisions + architecture spec §7.5. PF-09 count_agents/count_skills re-measured flat with exact parity (B1 tables) — no regression from the new count_sessions code. PF-04 search failure signal untouched by iter-3 changes (memory_service path unchanged).

---

## 03 · Performance Bottlenecks

**Findings (every open observation cataloged):**

- **[LOW] PF-11 (NEW, iteration 3) — estimate-num-keys inflation is user-visible through get_overview/CLI status after session mutations, and flush does not correct it.** Measured: 100 creates → 100/100 (exact); +100 updates → 200/100 (2×); +50 deletes → 150/50 (3×); mixed → 170/60; **after `flush()` → 170/60 (no correction)**. End-to-end: `get_overview.total_sessions = 210` vs 100 actual after turn-count updates; CLI `contexter status` (status_commands.py:43) surfaces this directly. This is documented accepted semantics (EC-CS-003) and mirrors the count_agents/count_skills precedent accepted in PF-09 — however sessions are the highest-mutation-frequency entity in the system (every conversation turn updates the session row; cleanup deletes sessions), so the user-visible inflation is far more likely to be hit in practice than for agents/skills. Before this fix the unfiltered count was an exact full scan; the fast path traded exactness for O(1) latency per the explicit contract (REQ-CS-001, AC-CS-004), and the ACs only assert fresh-store parity — but no compaction trigger is exposed anywhere (engine API has flush/checkpoint only), so once the memtable holds update/delete history the CLI count stays inflated indefinitely until RocksDB's background compaction happens to merge those keys. Severity: LOW (documented, contract-compliant, precedent-accepted) but a genuine user-visible correctness note.

**Resolution verification (prior findings):**

- **PF-10 (LOW, iter-2) — RESOLVED.** Unfiltered count_sessions now uses the `rocksdb.estimate-num-keys` O(1) fast path (rocksdb.rs:715-731), empirically flat at 0.0005–0.0019 ms direct / 0.17–0.24 ms bridge across 0→2000 sessions vs 2.538 ms before (~15–17× at the bridge, ≥1000× at the engine). Full-scan fallback preserved (AC-CS-006).
- **PF-09 (MEDIUM) — RESOLVED (re-verified).** count_agents/count_skills flat and exact; no `list_*` scan on the overview path.
- **PF-04 (LOW) — RESOLVED (unchanged, re-verified).** Search count failure signal intact; happy path exactly 2 engine calls.
- **PF-05 (informational) — RESOLVED (re-verified).** Per-call events at DEBUG; INFO reserved for lifecycle/errors.
- **PF-06/07/08 (informational) — RESOLVED (re-verified).** Documented in README Design Decisions + architecture spec §7.5.
- **PF-01/02/03 (baseline) — RESOLVED (re-verified, unchanged).** Session limit pushdown, single UTF-8 encode on bytes path, canonical pool env.

**Positives:** no new hot-path work in the MCP call path from the logging filter (0 bytes success stderr; ~200 ns only when a record is emitted on fastmcp loggers); filter idempotent and survives framework logging re-configuration; count fast path mirrors the established precedent exactly; fallback semantics identical; error-path memory bounded; test coverage now includes the framework-level stderr path the iter-2 gap identified (EC-FL-007).

---

## 04 · Optimization Recommendations

> **High Impact**
> No HIGH-impact issues. The last O(store) element on the analytics-overview path (PF-10) is eliminated; fastmcp failure-stderr bounding adds no success-path overhead (0 bytes stderr, ~200 ns/record only on fastmcp loggers).

> **Medium Impact**
> None — no MEDIUM or HIGH findings in iteration 3.

> **Quick Wins**
> PF-11 (LOW, informational): consider documenting the estimate caveat in the CLI status output or README Design Decisions (README currently has 0 mentions of estimate semantics; only test docstrings and the implementation report note it), and/or expose a compaction trigger (`contexter gc` currently runs flush + checkpoint, which does NOT correct the estimate — measured 170/60 after flush). Exactness remains available via filtered counts and list_sessions at any time.

---

_Generated by Performance Benchmarker · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
