# Performance Review Report

# MCP Live-Functionality Repair — Scrutiny: Performance Review (Auto Bug Loop Iteration 2)

> Performance re-review of the ENTIRE feature scope in the working tree (HEAD 27e031d, uncommitted): agent/skill count endpoints (PF-09), search count failure signal (PF-04), DEBUG per-call logging (PF-05), documented accepted decisions (PF-06/07/08), new bridge count wrappers, and the failure-path diagnostics log write. Baseline reports review-mcp-live-fix-scrutiny-performance-review.md and ...-iter-1.md are immutable references; every prior finding is re-verified against the current tree and re-stated if not fully resolved.

**Verdict:** CONDITIONAL PASS — all iter-1 findings (PF-09, PF-04, PF-05, PF-06/07/08) are RESOLVED and verified with live Rust-engine measurements; 1 NEW LOW finding (PF-10: unfiltered count_sessions still full-scans and is now the dominant cost of the get_overview path) (class: SCRUTINY/PERFORMANCE — static analysis + live-engine runtime verification)

2026-08-02 · 8 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| PF-09 overview counts | RESOLVED — `get_overview` uses `count_agents`/`count_skills` (no `list_*` scans); unfiltered counts hit the `rocksdb.estimate-num-keys` O(1) fast path (rocksdb.rs:1156, :1335) |
| Empirical O(1) evidence | count_agents 0.092→0.174 ms, count_skills 0.078→0.104 ms, count_memories 0.081→0.077 ms (empty → 3,000 records) — FLAT; the eliminated scans were 13.8 ms / 10.7 ms at just 3,000 rows |
| `get_overview` end-to-end | 1.41 ms empty → 3.63 ms with 11,000 seeded records (6 gathered engine calls); dominant remaining cost = `count_sessions` full scan (2.54 ms @ 2,000) |
| PF-04 search happy path | EXACTLY 2 engine calls (`search_memories` + `count_memories`, spy-verified), `total=3000` real int, 5.4 ms; count failure → `total=-1` + explicit `search_count_failed` ERROR log (REQ-STF-001); zero happy-path overhead |
| PF-05 per-call logging | `bridge_call_end` (bridge) + `call_received`/`auth_decision`/`engine_result` (handlers) ALL at DEBUG; INFO reserved for lifecycle/error events |
| PF-06/07/08 documentation | All accepted decisions documented in README "Design Decisions" (lines 281-306) + architecture spec §7.5 (lines 953-976) |
| NEW bridge count wrappers | `count_agents`/`count_skills` dispatch via `_run` (bounded 8-thread pool, mock guards, capped args summary) — identical pattern to existing counts |
| Diagnostics log write | Failure-path ONLY (except branch of `_run`), best-effort (never raises), bounded record; success path untouched |

> **Analysis Scope**
> Full working-tree review of feature/mcp-live-fix (HEAD 27e031d, all changes uncommitted). Files: services/analytics_service.py, core/bridge.py, services/memory_service.py, mcp_tools/handlers.py, mcp_server.py, contexter-core/src/{bridge.rs,storage/rocksdb.rs,engine/agent.rs,engine/skill.rs}, README.md, docs/design/specs/2026-07-23-contexter-system-architecture.md (§7.5). Contract: SPEC.md (REQ-001..007, §10), ACCEPTANCE.md, EDGE_CASES.md, bug contracts analytics-count-endpoints (REQ-ACE-001..005) and search-total-failure (REQ-STF-001..004). Verification: static code-path analysis + live Rust-engine harness (11,000-record seeded store under docs/tests/, deleted after use) + 44 targeted tests (analytics service, memory service, env canonicalization, handler limit passthrough) all green.

---

## 02 · Benchmark Results

**B1 — PF-09 agent/skill counts: RESOLVED (static + live).** `AnalyticsService.get_overview` (analytics_service.py:99-109) gathers 6 engine calls: `storage_size`, `status`, `count_sessions`, `count_memories`, `count_agents`, `count_skills` — the `list_agents({},1_000_000)` / `list_skills({},1_000_000)` full-store scans are GONE (REQ-ACE-003). Engine side: `count_agents` (bridge.rs:323) and `count_skills` (bridge.rs:397) delegate to storage where UNFILTERED counts return the `rocksdb.estimate-num-keys` CF property (rocksdb.rs:1156, :1335) — an O(1) property read; the full scan is only a fallback when the property is unavailable. Filtered counts scan (no secondary index; same semantics as `list_*`). The memory_items CF holds only memory keys (index entries live in the separate memory_index CF, rocksdb.rs:124, :302-348), so the estimate is accurate modulo RocksDB's inherent estimate error — a documented accepted semantics (test_bridge_live_coverage.py docstring; REQ-S-004 pattern). Rust correctness tests exist (agent_skill_test.rs:148-262: unfiltered and filtered counts match store).

**B2 — Empirical O(1) verification (live engine, 11,000-record store).** Empty → 3,000 seeded records: count_agents 0.092 → 0.174 ms; count_skills 0.078 → 0.104 ms; count_memories 0.081 → 0.077 ms. Latency is FLAT — no store-size scaling, confirming the estimate fast path is taken. Contrast — the eliminated approach: list_agents(limit=1M) 13.8 ms and list_skills(limit=1M) 10.7 ms at only 3,000 rows (60-170× slower, excluding the JSON decode + pydantic materialization that used to happen in Python). Counts matched seeded data exactly (3,000/3,000/3,000/2,000) on a fresh store.

**B3 — get_overview end-to-end: 1.41 ms (empty) → 3.63 ms (11,000 records).** Six calls gather concurrently in the bounded thread pool. The remaining dominant cost is count_sessions' unfiltered full scan (2.54 ms @ 2,000 sessions — see PF-10); storage/status/agents/skills/memories together account for ~1.1 ms.

**B4 — PF-04 search count failure signal: RESOLVED.** memory_service.py:76-84: count-call failure → `logger.error("search_count_failed", ...)` + `total=-1` distinguishing signal while results are still returned (REQ-STF-001 option c). Happy path verified live: EXACTLY 2 engine calls (`search_memories`, `count_memories` — method-spy verified), `total=3000` (real int), 5.4 ms end-to-end. No extra calls, no added happy-path work; the failure branch is an O(1) isinstance check + a logger.error that never fires on the happy path (REQ-STF-002).

**B5 — PF-05 per-call logging at DEBUG: RESOLVED.** bridge.py:261-266 logs `bridge_call_end` at DEBUG (method, truncated args summary, duration_ms). All handler per-call events (`call_received`, `auth_decision`, `engine_result`) are at DEBUG across every handler including the analytics overview resource (handlers.py:148-527). INFO is reserved for lifecycle/error events; the failure path logs at ERROR with bounded context. The default INFO level stays quiet under sustained MCP call rates — no hot-path INFO overhead.

**B6 — PF-06/07/08 documented: RESOLVED.** README "Design Decisions → Accepted performance decisions" (lines 281-306) and architecture spec §7.5 (lines 953-976) document all four: list tools bounded at 100 with no pagination (PF-06); store_memory's exactly-two-sequential-calls design, not N+1 (PF-07); export_data's bounded 10,000/entity materialisation with LRU caching (PF-08).

**B7 — NEW bridge count wrappers: POSITIVE.** bridge.py:379-381 (count_agents) and :410-412 (count_skills) mirror count_sessions/count_memories exactly: serialize filter ("{}" when None), dispatch via `self._run` → `loop.run_in_executor(self._pool, ...)` with the bounded pool, mock-rejection guards, capped args summary, and DEBUG per-call event. No new dispatch path, no unbounded work. Live calls measured at 0.1-0.2 ms.

**B8 — Failure-path diagnostics write: POSITIVE (bounded, best-effort, off hot path).** `_write_runtime_failure_diagnostics` (bridge.py:136-162) executes ONLY in the except branch of `_run` (bridge.py:238) — never on the success path. It appends a structured record + traceback to the diagnostics log (CONTEXTER_LOG_FILE / ~/.contexter/logs/mcp-launch.log), never raises (returns None on any write failure), and the stderr line stays bounded (<512 chars; capped path, truncated args). Verified live: a real engine ValueError during seeding produced the diagnostics-file entry plus a single concise `bridge_call_failed` ERROR line. Success-path overhead: zero — no code added between dispatch and return beyond the existing DEBUG event.

**B9 — Regression sweep.** No `list_agents`/`list_skills` call remains on the overview path (grep-verified across services/handlers: remaining callers are bounded — agent/skill services default 100, onboarding default 100, export 10k LRU-cached per documented PF-08). Thread-pool containment, mock guards, and bounded arg-summary behavior unchanged from iter-1. Targeted tests: 44 passed (test_analytics_service.py, test_memory_service.py, test_env_canonicalization.py, test_handler_limit_passthrough.py).

---

## 03 · Performance Bottlenecks

**Findings (every open observation cataloged):**

- **[LOW] PF-10 (NEW, iteration 2) — Unfiltered `count_sessions` still full-scans; it is now the dominant cost of `get_overview`.** rocksdb.rs:691-727: `count_sessions` has an index-prefix fast path ONLY for `project` filters; the unfiltered call that `get_overview` makes (`count_sessions({})`) falls through to a full scan of the sessions CF with serde deserialization per row. Empirically: 0.078 ms empty → 2.538 ms @ 2,000 sessions (linear scaling; ~125 ms projected at 100k sessions), and it is ~70% of the 3.63 ms overview call. The sessions CF holds only session keys (index entries live in the separate session_index CF, rocksdb.rs:538-540, :655-683), so the same `rocksdb.estimate-num-keys` O(1) fast path just added for count_agents/count_skills (rocksdb.rs:1156, :1335) is directly applicable. This is a PRE-EXISTING engine gap (count_sessions predates this fix and was already called by get_overview in iter-1) — NOT a regression of this iteration — but it is the last O(store) element on the analytics-overview path that the PF-09 contract was created to optimize. Contract ref: analytics-count-endpoints (REQ-ACE-001 pattern), SPEC §10 performance validation.

**Resolution verification (iter-1 findings):**

- **PF-09 (MEDIUM) — RESOLVED.** Count endpoints added (bridge.rs:323/:397), O(1) estimate fast path verified statically and empirically; no `list_*` full-store scan remains on the overview path (B1-B3).
- **PF-04 (LOW) — RESOLVED.** Count failure surfaces `total=-1` + explicit ERROR log; happy path still exactly 2 engine calls with correct total (B4).
- **PF-05 (informational) — RESOLVED.** Per-call events at DEBUG, INFO reserved for lifecycle/errors; documented (B5).
- **PF-06/07/08 (informational) — RESOLVED.** All documented as accepted decisions in README Design Decisions + architecture spec §7.5 (B6).
- **Baseline PF-01/02/03 — remain RESOLVED** (session limit pushdown, single UTF-8 encode on bytes path, canonical `CONTEXTER_BRIDGE_POOL_SIZE`) — unchanged from iter-1, verified.

**Positives:** no N+1 patterns; no unbounded fetches; thread-pool containment correct (bounded 8 workers, run_in_executor, honest duration accounting); error-path memory bounded; new count wrappers consistent with the dispatch pattern; diagnostics write off the hot path.

---

## 04 · Optimization Recommendations

> **High Impact**
> No HIGH-impact issues. The O(store) analytics-overview work from iter-1 (PF-09) is eliminated for agents/skills/memories; blocking-call containment, N+1, and memory-bounding remain correct.

> **Medium Impact**
> None — the iter-1 MEDIUM (PF-09) is fully resolved; no new MEDIUM or HIGH issues found.

> **Quick Wins**
> PF-10: add the `rocksdb.estimate-num-keys` O(1) fast path to unfiltered `count_sessions` in contexter-core/src/storage/rocksdb.rs (mirror count_agents/count_skills at :1156/:1335). The sessions CF holds only session keys, so the estimate is valid; this removes the last O(store) scan from `get_overview` (MCP resource + CLI status) and cuts overview latency from ~2.5 ms to ~0.1 ms at 2,000 sessions, with scaling that stays flat beyond.

---

_Generated by Performance Benchmarker · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
