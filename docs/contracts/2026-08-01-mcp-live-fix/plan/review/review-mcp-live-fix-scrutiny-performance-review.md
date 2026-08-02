# Performance Review Report

# MCP Live-Functionality Repair — Scrutiny: Performance Review

> Static performance review of the MCP live-path repair: StorageEngine bridge rewiring (run_mcp.py), bridge hardening (+87 lines: camelization, mock rejection, bytes path), memory_service translation layer, and handler `type` param restoration. Scope: blocking-call containment, per-call serialization/translation cost, query/limit handling, startup cost, error-path memory behavior.

**Verdict:** CONDITIONAL PASS — no correctness-threatening perf regression; 1 MEDIUM over-fetch + 2 LOW items to address (class: SCRUTINY/PERFORMANCE — static analysis)

2026-08-01 · 8 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Dispatch containment (`_run`) | `loop.run_in_executor(self._pool, fn, *args)` — bounded 8-thread pool; event loop never blocks; queue wait included in `duration_ms` |
| Bridge dispatch overhead | ~2 attribute lookups + 2 isinstance checks + monotonic + truncated-args repr per call — sub-µs, negligible |
| Camelization cost | O(top-level keys) per payload; nested maps untouched; no quadratic behavior on large payloads |
| Bytes path (≥102400 B) | Content encoded twice per store (threshold check + payload) — LOW finding; otherwise avoids json double-encode as designed |
| `search_memories` | 2 engine calls per search (results + count) via gather — pre-existing; count is a full-scan count |
| `list_recent_sessions` | Engine always fetches default 100, then handler slices — MEDIUM over-fetch / silent cap |
| `list_skills` | Bounded at bridge default 100; no MCP pagination — informational |
| Startup | `Engine.open` at launch, once per stdio subprocess; Rust wheel import bounded — informational |

> **Analysis Scope**
> Static code-path analysis of the `feature/mcp-live-fix` working-tree diff (no commits). Files reviewed: `run_mcp.py`, `src/contexter_server/core/bridge.py` (+87 lines), `src/contexter_server/services/memory_service.py`, `src/contexter_server/mcp_server.py`, `src/contexter_server/mcp_tools/handlers.py`, `src/contexter_server/api/deps.py`, plus unmodified service layers (session/skill/analytics/export) to assess query/limit behavior of the repaired call paths. Contract: SPEC.md (REQ-001..007, Section 10 validation), EDGE_CASES.md (EC-010/011/013/016/018/019), approved design preview (bridge contract: `asyncio.to_thread` dispatch, bytes path ≥102400 B). No load tests run — static analysis only, within the 5-minute limit.

---

## 02 · Benchmark Results

No load tests executed (static analysis per 5-min constraint). All measurements below are complexity/code-path estimates from direct inspection of `bridge.py`, `run_mcp.py`, `mcp_server.py`, `mcp_tools/handlers.py`, `services/memory_service.py`, `services/session_service.py`, `services/skill_service.py`, `services/analytics_service.py`, `services/export_service.py`.

**B1 — Blocking-call containment (EC-013/EC-018): POSITIVE.** Every sync Rust call goes through `StorageEngine._run` → `loop.run_in_executor(self._pool, fn, *args)` with a bounded `ThreadPoolExecutor(max_workers=8)` (env-tunable). The event loop never blocks on RocksDB I/O. Concurrent tool calls beyond 8 queue in the executor (bounded wait) rather than starving the loop — correct containment for the stdio MCP single-client model. `time.monotonic()` starts before executor submission, so `duration_ms` includes queue wait — honest end-to-end latency metric.

**B2 — Dispatch guard overhead: NEGLIGIBLE.** Per call: `getattr(_SYNC_ENGINE_CLASS, method)`, one Mock isinstance check on class attr, `getattr(self._engine, method)` (bound method alloc), one Mock isinstance check on instance attr. All sub-microsecond; no hot-loop impact.

**B3 — `_truncated_args_summary`: POSITIVE.** Correctly avoids the `str(args)[:200]` anti-pattern: str/bytes args >~94 chars are sliced before repr, so a 100 KB+ content argument never materializes a 100 KB repr string. Bounded at 200 chars on all paths; error-path logging cannot balloon memory.

**B4 — Camelization: POSITIVE (no quadratic behavior).** `_camelize_payload_keys` is a single top-level dict comprehension; `_snake_to_camel` is O(key length) split/join. Nested values (`metadata`, embeddings) pass through untouched by design. Applied to every bridge payload (incl. filter dicts) — constant-factor cost only.

**B5 — Bytes path (EC-010): LOW finding.** `create_memory`/`update_memory` call `content.encode("utf-8")` to measure length, then encode AGAIN for the bytes payload when ≥102400 B (and `json.dumps` internally re-encodes on the small path). For a 1 MB memory this is ~2 MB of UTF-8 encode work per call — small (~ms) but exactly the class of waste the bytes path exists to avoid. Fix: encode once, reuse the bytes object.

**B6 — `search_memories`: LOW (pre-existing, not a regression).** `MemoryService.search` runs `search_memories` + `count_memories` concurrently (2 engine calls per search); the count is a full-scan count over the engine index. Doubling per-search engine work is a deliberate pagination design (total is returned to the client), pre-existing in `search_service.py` too — not introduced by this fix. `return_exceptions=True` silently degrades to `total=0` on count failure (no retry).

**B7 — `list_recent_sessions`: MEDIUM.** `handle_list_recent_sessions` calls `session_service.list()` which calls bridge `list_sessions(filter, limit=100, offset=0)` — the handler's `limit` is applied AFTER the fetch (`sessions[:limit]`). Client `limit=5` still pulls, JSON-deserializes, and pydantic-validates up to 100 sessions. Client `limit=200` is silently capped at 100. The limit must be pushed down to the engine query.

**B8 — Startup / per-client cost: INFORMATIONAL.** Each stdio client spawns a fresh subprocess: `import contexter_core` (Rust wheel) + `Engine.open(path)` (RocksDB open) + 8-thread pool creation happen once per process. Bounded, matches `main.py` pattern; no lazy-init added. `export_data` may fetch up to 10,000 records/entity into memory (pre-existing, bounded). `handle_store_memory` performs 2 sequential engine calls (get_session for agent_id + create_memory) — deliberate domain flow, not N+1.

---

## 03 · Performance Bottlenecks

**Findings (every observation cataloged):**

- **[MEDIUM] PF-01 — `list_recent_sessions` limit not pushed down to engine.** `handlers.py:131-135`: `sessions = await session_service.list(filter=filter_obj)` (bridge default limit=100) then `sessions[:limit]`. Over-fetch for small limits (e.g. limit=5 → 100 records fetched, serialized, validated), silent cap for limits >100. Fix: thread `limit` through `SessionService.list` → `bridge.list_sessions(limit=limit)`.
- **[LOW] PF-02 — Double UTF-8 encode on large-content bytes path.** `bridge.py:214` and `:238`: `len(content.encode("utf-8")) >= _LARGE_CONTENT_THRESHOLD` then `content.encode("utf-8")` again for the bytes payload. Encode once and reuse.
- **[LOW] PF-03 — `CONtexTER_BRIDGE_POOL_SIZE` env var not canonicalized.** `bridge.py:112` retains the old mixed-case `CONtexTER` prefix while this same fix canonicalized `CONtexTER_API_KEY` → `CONTEXTER_API_KEY` everywhere else. A user setting the canonical `CONTEXTER_BRIDGE_POOL_SIZE` would be silently ignored; pool sizing config is a footgun. Rename for consistency (documented env var, not functional today).
- **[LOW] PF-04 — Search = 2 engine calls per request (count full-scan).** `memory_service.py:56-60` gather(search, count). Pre-existing behavior; doubles per-search engine work. Consider caching count for identical query+page windows or making total optional.
- **[informational] PF-05 — Per-call INFO logging.** `bridge.py:169` `logger.info("bridge_call_end", args_summary=..., duration_ms=...)` fires once per engine call. Acceptable at MCP call rates; consider DEBUG if the server ever handles sustained concurrent load.
- **[informational] PF-06 — MCP list tools bounded at 100 with no pagination.** `list_skills` (no limit param in frozen contract), `list_recent_sessions` capped at bridge default 100. Not a regression; flag for future capacity work.
- **[informational] PF-07 — `handle_store_memory` makes 2 sequential engine calls** (get_session → create_memory). Deliberate agent_id derivation; not N+1.
- **[informational] PF-08 — `export_data` gathers up to 10,000 records per entity into memory** (pre-existing, bounded by `limit=10_000`); fine for single-client stdio model.
- **[POSITIVE] No N+1 patterns found** in list/search paths — each service op is a single engine call.
- **[POSITIVE] No unbounded memory in error paths** — `_truncated_args_summary` bounds all logged argument reprs; `logger.exception` bounded.
- **[POSITIVE] Mock-rejection guards are free** — isinstance checks are O(1); dispatch validation does not measurably affect the hot path.

---

## 04 · Optimization Recommendations

> **High Impact**
> No HIGH-impact performance issues found. Blocking-call containment via bounded thread pool is correct; no event-loop stall, no N+1, no unbounded memory.

> **Medium Impact**
> PF-01: Push `limit` down to the engine in `list_recent_sessions` (eliminates up to 20x over-fetch and removes the silent 100-cap).

> **Quick Wins**
> PF-02: Single UTF-8 encode on the bytes path (encode once, reuse).
PF-03: Canonicalize `CONtexTER_BRIDGE_POOL_SIZE` → `CONTEXTER_BRIDGE_POOL_SIZE` (config consistency).
PF-04: Make the count query optional or cache it per query window.

---

_Generated by Performance Benchmarker · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
