# Performance Review Report

# MCP Live-Functionality Repair — Scrutiny: Performance Review (Auto Bug Loop Iteration 1)

> Performance re-review of the ENTIRE feature scope in the working tree (HEAD 27e031d, uncommitted): session limit pushdown (handler → service → engine), bounded thread pool (default 8, run_in_executor), large-content bytes path (>=102400 B), env-var canonicalization, handler observability, analytics telemetry mapping, launch path. Baseline report review-mcp-live-fix-scrutiny-performance-review.md is the immutable reference; every prior finding (PF-01..PF-08) is re-verified against the current tree and re-stated if not fully resolved.

**Verdict:** CONDITIONAL PASS — 1 NEW MEDIUM finding (analytics overview full-store scans) + re-stated pre-existing LOW/informational items; all three baseline MEDIUM/LOW action items (PF-01, PF-02, PF-03) are RESOLVED (class: SCRUTINY/PERFORMANCE — static analysis + targeted runtime verification)

2026-08-01 · 10 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Dispatch containment (`_run`) | `loop.run_in_executor(self._pool, fn, *args)` — bounded 8-thread pool (env `CONTEXTER_BRIDGE_POOL_SIZE`, invalid/non-positive → 8); event loop never blocks; queue wait included in `duration_ms` |
| PF-01 limit pushdown | COMPLETE: handler clamps (None→None, neg/0→0, >10_000→10_000) → `session_service.list(limit=)` → `bridge.list_sessions(limit=)` → engine. No Python re-slice (service result authoritative) |
| PF-02 bytes path | RESOLVED: `content.encode("utf-8")` executed exactly once, reused for threshold check AND payload (bridge.py:231, :263); no JSON re-encode of large content |
| PF-03 pool env var | RESOLVED: `CONTEXTER_BRIDGE_POOL_SIZE` is the single canonical source; repo-wide grep test forbids `CONtexTER_` prefix |
| Bounded args logging | Empirical: 10 MB string arg → 69-char summary, 0.023 ms, 460 B peak traced allocation (10 MB repr never materialized) |
| Live large-content latency | Real engine: create_memory 1 MiB = 12.0 ms, get_memory 1 MiB = 3.3 ms — no pathological latency |
| `search_memories` | 2 engine calls per search (results + count via gather) — PF-04 pre-existing, unchanged |
| `analytics/overview` | NEW: 6 engine calls gathered; agents/skills counted via FULL-STORE scans (`list_agents/list_skills` limit=1_000_000) — NEW MEDIUM finding PF-09 |
| MCP list tools | Bounded at 100 (bridge default); `list_skills` has no limit param (frozen contract) — informational PF-06 |
| Startup / launch | `Engine.open` once per stdio subprocess; failure path: 1 clean stderr line + server-side log, exit 2; stdout untouched — bounded |

> **Analysis Scope**
> Full working-tree review of feature/mcp-live-fix (HEAD 27e031d, all changes uncommitted). Files: run_mcp.py, core/bridge.py, mcp_tools/handlers.py, mcp_tools/errors.py, mcp_tools/auth.py, mcp_server.py, services/{session,memory,skill,agent,analytics,export}_service.py, models/{agent,skill}.py, cli/status_commands.py, api/deps.py, rate_limiter.py, main.py, README.md, docs/design/specs/2026-07-23-contexter-system-architecture.md. Contract: SPEC.md (REQ-001..007, §10), ACCEPTANCE.md (AC-9..11 non-functional), EDGE_CASES.md (EC large content >=102400 B, limit edges, concurrency), 18 bug contracts (esp. session-limit-pushdown PF-01, handler-limit-passthrough, bridge-double-encode PF-02, env-var-canonicalization PF-03, bridge-log-hygiene, analytics-telemetry-mapping). Verification: 58 targeted tests executed (36 perf-contract + 22 live service/observability) all green; lightweight runtime checks under docs/tests/ (deleted after use). Rust surface checked for count endpoints (only count_sessions/count_memories exist).

---

## 02 · Benchmark Results

**B1 — Blocking-call containment: POSITIVE (unchanged).** Every sync Rust call dispatches via `loop.run_in_executor(self._pool, fn, *args)` on a bounded `ThreadPoolExecutor`. Pool size = `CONTEXTER_BRIDGE_POOL_SIZE` (canonical, PF-03 resolved), default 8, invalid/non-positive → 8. Concurrent calls beyond 8 queue in the executor; the event loop never blocks on RocksDB I/O. `time.monotonic()` starts before submission — `duration_ms` includes queue wait (honest end-to-end latency).

**B2 — Dispatch guard overhead: NEGLIGIBLE (unchanged).** Per call: class attr lookup + Mock isinstance, instance attr lookup + Mock isinstance, truncated summary, monotonic. All O(1); no hot-loop impact. Mock-rejection TypeError guards are defense-in-depth and free.

**B3 — `_truncated_args_summary` bounded: VERIFIED EMPIRICALLY.** Scratch harness (deleted): 10 MB string arg → summary length 69 chars, 0.023 ms, peak traced allocation 460 B. Content args capped at 64 chars/bytes (`_ARG_SUMMARY_CAP`); large reprs are never materialized. Error-path logging cannot balloon memory.

**B4 — Camelization: POSITIVE (no quadratic behavior).** Single top-level dict comprehension; `_snake_to_camel` is O(key length). Nested `metadata`/embeddings pass through untouched. Constant-factor only.

**B5 — PF-01 session limit pushdown: RESOLVED.** Handler (`handlers.py:286-293`) passes `_clamp_session_list_limit(limit)` (None→None, ≤0→0, >MAX_SESSION_LIST_LIMIT(10_000)→10_000) to `session_service.list()`; service clamps again and forwards to `bridge.list_sessions(filter, limit=)`; bridge injects limit/offset into the engine filter. Service result is authoritative — no `sessions[:limit]` re-slice remains (grep verified). `limit=5` now fetches 5, not 100; `limit=10**9` is clamped to 10_000. Tests: `tests/mcp/test_handler_limit_passthrough.py` (8 tests) prove the limit reaches the service boundary.

**B6 — PF-02 double-encode on bytes path: RESOLVED.** `create_memory`/`update_memory` encode content once (`content.encode("utf-8")`) and reuse the bytes for the threshold check and the `create_memory_bytes`/`update_memory_bytes` payload (bridge.py:231, :263). Static verification: exactly 1 encode call per path. Runtime verification: `tests/core/test_bridge_large_content_roundtrip.py` — byte-identical round-trips at exactly 102400 B, 102399 B, 1 MiB, 102402 B multi-byte, search path, and update path (6 tests, all green).

**B7 — PF-03 pool env canonicalization: RESOLVED.** `bridge.py:127` reads only `CONTEXTER_BRIDGE_POOL_SIZE`; `tests/core/test_env_canonicalization.py` greps ALL production sources for the misspelled `CONtexTER_` prefix (zero allowed) and asserts the pool honors the canonical var (4 workers). REST-layer vars (`CONTEXTER_API_KEY`, `CONTEXTER_RATE_LIMIT*`, `CONTEXTER_ENABLE_DOCS`) canonicalized in deps.py/main.py/rate_limiter.py; README Configuration table documents the canonical set.

**B8 — PF-04 search = 2 engine calls: RE-STATED (pre-existing, unchanged).** `memory_service.search` gathers `search_memories` + `count_memories`; the count is a full-scan count. `return_exceptions=True` silently degrades to `total=0` on count failure. Not introduced by this fix (diff shows only query-vocabulary translation added). Informational: `SearchResult.data` copies full content (minus embedding) per result — bounded by MAX_SEARCH_LIMIT=100 and the 1 MB content cap; the handler serializes only id/type/score/snippet, so the copy is discarded at the adapter boundary.

**B9 — PF-09 (NEW) analytics overview full-store scans: MEDIUM.** `analytics_service.get_overview` (new telemetry-mapping code in this fix) gathers `list_agents({}, 1_000_000, 0)` and `list_skills({}, 1_000_000, 0)` — the ENTIRE agents and skills tables are JSON-serialized by the engine, decoded by the bridge, and materialized as Python dicts solely to take `len()`. The Rust engine exposes no `count_agents`/`count_skills` (verified: only `count_sessions`/`count_memories` in bridge.rs), hence the scan. Bounded at 1M records but O(store size) on every `contexter://analytics/overview` resource read and every `contexter status` CLI run (get_overview is called there too). At 100k+ agents/skills this is 100k+ dicts per call (tens of MB); the two scans run concurrently in the 8-thread pool and can saturate all workers during the call.

**B10 — Launch path: POSITIVE (unchanged, bounded).** Each stdio subprocess imports the Rust wheel and opens the engine once; failure path prints ONE clean stderr line (no traceback), appends full diagnostics to `~/.contexter/logs/mcp-launch.log` (best-effort, never raises), exits 2 (fastmcp-missing uses 1). stdout carries only MCP frames. CLI `status` now issues ~14 engine calls (5 sequential service calls incl. version read) — bounded, one-shot CLI; no finding.

**B11 — Services N+1/unbounded-fetch sweep: POSITIVE.** Every service op is a single engine call; no N+1 in list/search paths. `memory_service.list` bounded 100; `skill_service.list`/`agent_service.list` at bridge default 100; `export_service` gathers up to 10_000/entity (pre-existing, LRU-cached results, PF-08 informational); `store_memory` does 2 sequential engine calls (get_session → create_memory) — deliberate agent_id derivation, not N+1 (PF-07 informational).

---

## 03 · Performance Bottlenecks

**Findings (every observation cataloged):**

- **[MEDIUM] PF-09 (NEW, iteration 1) — Analytics overview counts via full-store scans.** `analytics_service.py:113-123` (`get_overview`): `list_agents({}, 1_000_000, 0)` + `list_skills({}, 1_000_000, 0)` materialize the complete agent and skill tables in Python per call, only to compute `len()`. New code in this fix (analytics telemetry mapping, REQ-AN-001). Engine lacks `count_agents`/`count_skills` — the correct fix is engine-side O(1) counts mirroring `count_sessions`/`count_memories` (Rust `engine/session.rs:128`, `engine/search.rs:85` already exist). Contract ref: analytics-telemetry-mapping bug contract; SPEC §10 performance validation ("no N+1, bounded engine calls").
- **[LOW] PF-04 (RE-STATED, pre-existing, unchanged) — Search = 2 engine calls per request (count full-scan).** `memory_service.py:56-60` gather(search, count) with `return_exceptions=True` → `total=0` on count failure. Consider making total optional or caching per query window. Contract ref: EDGE_CASES.md EC large-content/search; baseline PF-04.
- **[informational] PF-05 (RE-STATED) — Per-call INFO logging.** `bridge.py:184-189` `bridge_call_end` fires once per engine call with duration. Documented as accepted design (README Design Decisions, architecture spec §7.2). Acceptable at MCP call rates.
- **[informational] PF-06 (RE-STATED) — MCP list tools bounded at 100, no pagination.** `list_skills` (no limit param — frozen contract), `list_recent_sessions` defaults to engine 100. Not a regression; documented.
- **[informational] PF-07 (RE-STATED) — `handle_store_memory` 2 sequential engine calls** (get_session → create_memory). Deliberate; not N+1.
- **[informational] PF-08 (RE-STATED) — `export_data` gathers up to 10,000 records/entity** into memory (pre-existing, bounded; results LRU-cached).
- **[POSITIVE] Baseline PF-01 (MEDIUM) RESOLVED** — limit pushed down to engine; no over-fetch for small limits; no silent cap beyond clamped 10_000; no Python re-slice (handler test suite proves service-boundary propagation).
- **[POSITIVE] Baseline PF-02 (LOW) RESOLVED** — single UTF-8 encode on bytes path; byte-identical round-trips at threshold/1 MiB/multi-byte/search/update.
- **[POSITIVE] Baseline PF-03 (LOW) RESOLVED** — `CONTEXTER_BRIDGE_POOL_SIZE` canonical; no misspelled `CONtexTER_` prefix anywhere in production code.
- **[POSITIVE] No N+1 patterns** in handlers/services; no unbounded fetches; no `sessions[:limit]` re-slice remains; error-path memory bounded (empirically verified).
- **[POSITIVE] Thread-pool containment correct** — bounded pool, `run_in_executor`, honest duration accounting.
- **[POSITIVE] Runtime evidence** — 36 perf-contract tests + 22 live service tests pass; 1 MiB store round-trip 12.0 ms / 3.3 ms.

---

## 04 · Optimization Recommendations

> **High Impact**
> No HIGH-impact issues. Blocking-call containment (bounded 8-thread pool via run_in_executor) is correct; no event-loop stall, no N+1, no unbounded memory, no double-encode, limit pushdown complete.

> **Medium Impact**
> PF-09: Add engine-side `count_agents`/`count_skills` to the Rust bridge (mirror existing `count_sessions`/`count_memories` in contexter-core/src/bridge.rs) and use them in `AnalyticsService.get_overview`; alternatively cap the scan lower and document "≥N" count semantics. Eliminates whole-table materialization (up to 1M records) on every analytics overview read.

> **Quick Wins**
> PF-04 (pre-existing): make the count query optional or cache totals per query window to halve per-search engine work. PF-05: demote bridge per-call `bridge_call_end` to DEBUG if sustained concurrent load is ever expected. PF-06: add explicit pagination/limit to `list_skills` when the frozen contract is next revised.

---

_Generated by Performance Benchmarker · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
