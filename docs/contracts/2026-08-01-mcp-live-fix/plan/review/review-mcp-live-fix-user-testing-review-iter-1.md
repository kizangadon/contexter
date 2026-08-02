# User-Testing Review Report

# MCP Server Live-Functionality Repair — User-Testing Review (Auto Bug Loop Iteration 1)

> Repairs the Contexter MCP server so all 8 tools + 4 resources return real data from the real Rust engine through a live stdio MCP client. Scope: contexter-server MCP layer only (REST/CLI/Rust core out of scope).

**Verdict:** CONDITIONAL PASS (class: finding-driven (1 LOW contract violation, 2 INFO observations, 2 INFO notes))

2026-08-01 · 21/22 AC passed · User-Testing Validator

---

## 01 · Test Overview

> **Browser & Environment**
> Linux (Python 3.12.3), FastMCP 3.4.0, MCP SDK 1.28 (stdio client). Server: `run_mcp.py` stdio transport, temporary engine dirs via `StorageEngine`/`contexter_core` (no user data touched, per SPEC DAT-001). Verification via live `mcp.client.stdio.ClientSession` + raw JSON-RPC subprocess probes + full pytest suite.

> **Test Summary**
> All 8 tools + 4 resources verified live with real engine data; auth matrix (open mode / wrong key / correct key) verified; structured error shapes verified for 10 failure classes; 102400-B large content, limits, concurrency, liveness after errors; stdout purity 4/4 JSON-RPC frames; launch failure clean (rc=2, no traceback); empty-engine grace; CLI live check rc=0; full suite 794 passed / 0 failed. 1 LOW finding: AC-TH-001 literal violation (bare pytest.raises(Exception) at test_mcp_launcher_wiring.py:218).

---

## 02 · Acceptance Criteria Results

| AC | Phase | Status | Evidence |
|---|---|---|---|
| AC-1: 8 tools real data | API+UI | ✅ PASS | Live stdio: all 8 tools isError:false; list_recent_sessions 12 real sessions (agent_id, turn_count, duration_ms, started_at from engine) |
| AC-2: 4 resources real data | API | ✅ PASS | session/memory/agent reads OK; analytics: 12 sessions, 3 memories, 1 agent, 1 skill, storage 1073152 B (== get_system_health storage_size) |
| AC-3: type filter | API | ✅ PASS | list_skills type=search -> only search skill; search_memories type=memory -> filtered results |
| AC-4: auth preserved | API | ✅ PASS | No key -> open mode OK (health, list_skills); wrong key -> isError 'Invalid API key.'; correct key -> success; resource w/o _api_key -> McpError rejected |
| AC-5: store_memory persists | API | ✅ PASS | store_memory -> search_memories returns snippet 'iter1-roundtrip' (total 3) |
| AC-6: invalid params structured | API | ✅ PASS | nonexistent session/agent -> isError 'Resource not found: <id>'; missing query -> pydantic validation isError; extra param/bad uuid/empty/oversized content/bad format -> isError; no crash, no traceback in response |
| AC-7: empty datasets | API | ✅ PASS | empty engine: sessions [], skills [], search {results:[],total:0}, analytics zeroed, all success |
| AC-8: engine failure contained | API | ✅ PASS | corrupt engine: isError=True, server alive, health OK after errors; launch failure rc=2, 2 clean stderr lines, diagnostics in launch log |
| AC-9: no mocks in live path | Static | ✅ PASS | Live path returns real engine data; mock stub removed (per iter-1 security/code review); bridge mock-import guard |
| AC-10: suite green + regression tests | Suite | ✅ PASS | 794 passed, 0 failures (11.64s) — above ≥647 target; reproduction tests present |
| AC-11: no stdout pollution | API | ✅ PASS | raw JSON-RPC probe: 4/4 frames parse, 0 bad lines; only JSON-RPC on stdout |
| AC-ES-001: not-found error shape | API | ✅ PASS | isError=True, 'Resource not found: <id>', no success frame |
| AC-ES-002: validation error shape | API | ✅ PASS | structured validation isError; never traceback in response |
| AC-ES-003: auth serialization | API | ✅ PASS | MCPAuthError messages unchanged ('API key required...' / 'Invalid API key.') |
| AC-ES-004: error-shape tests | Suite | ✅ PASS | covered by 794-green suite |
| AC-ES-005: survives repeated errors | API | ✅ PASS | 14 error-path calls incl. 102400-B content + concurrency; liveness after all errors |
| AC-BH-001: log hygiene 10KB | API | ✅ PASS | 10KB content never in logs; args_summary capped; max stderr line 337 chars |
| AC-LH-001: launch error clean | API | ✅ PASS | corrupt/locked engine -> rc=2, clean 2-line stderr, NO traceback, structured message; launch log has full diagnostics |
| AC-HO-001: handler observability | API | ✅ PASS | structured logs: call_received, auth_decision, engine_result, handler_error, correlation_id; no secrets |
| AC-TH-001: no pytest.raises(Exception) | Suite | ❌ FAIL | tests/mcp/test_mcp_launcher_wiring.py:218 still uses bare `pytest.raises(Exception)`; engine raises stable `RuntimeError` (verified live), so precise type is catchable |
| AC-EV-002: env canonical src | Static | ✅ PASS | zero `CONtexTER_` in src/ (grep); CONTEXTER_API_KEY/BRIDGE_POOL_SIZE canonical |
| AC-DN-001: env canonical docs | Static | ✅ PASS | README + docs/design clean; remaining strings only in immutable historical contract reports + bug contracts' own descriptions |

---

## 03 · As-Built End-to-End Data Flow

**Interaction:** MCP client sends JSON-RPC request frame over stdin → FastMCP dispatches to tool/resource handler → handler calls service → bridge executes on thread pool → Rust Engine returns → response frame over stdout. Errors: handler raises HandlerError/MCPAuthError → FastMCP serialises structured isError result (auth-first ordering, bounded messages).

### Request Track · Forward Flow (1→5)

| Step | Layer | Action |
|---|---|---|
| 1 | User | MCP client (OpenCode / SDK) emits JSON-RPC `initialize` then `tools/call` / `resources/read` |
| 2 | Frontend | ClientSession writes request frame to server stdin; awaits response frame on stdout |
| 3 | API | run_mcp.py stdio transport: FastMCP parses frame, routes by method name to registered tool/resource handler |
| 4 | Service | Handler validates params (pydantic, limit clamping), requires `_api_key` first, calls service (e.g. analytics mapping, camelization) |
| 5 | Database | Bridge `StorageEngine` runs engine call in thread pool (8 workers); Rust contexter_core executes query |

**Layer Details (Request):**

> **User Layer:** Live MCP client — this E2E validation used mcp SDK 1.28 stdio_client and raw JSON-RPC subprocess probes
>
> **Frontend Layer:** ClientSession; every tool call and resource read observed at frame level (isError flags, payloads)
>
> **API Layer:** FastMCP 3.4.0 stdio transport on `run_mcp.py`; tool schema from handlers; `{?_api_key}` RFC-6570 templates on 4 resources
>
> **Service Layer:** mcp_tools/handlers.py + mcp_tools/auth.py + analytics_service.py; auth-first ordering; error mapping to structured isError
>
> **Database Layer:** core/bridge.py StorageEngine (asyncio.to_thread pool); Rust engine on temp dirs only (DAT-001)

### Response Track · Return Flow (6→10)

| Step | Layer | Action |
|---|---|---|
| 6 | Database | Engine returns typed result (or raises NotFound/ValueError) |
| 7 | Service | Bridge re-raises; handler catches and maps to HandlerError → isError result with bounded message |
| 8 | API | FastMCP serialises JSON-RPC result/error frame onto stdout; stderr carries structured logs only |
| 9 | Frontend | Client parses response; success payloads contain real data (verified field-by-field) |
| 10 | User | Client receives result (isError=false + payload) or structured error (isError=true, message); server stays alive for subsequent calls |

**Layer Details (Response):**

> **Database Layer:** Verified: 12 sessions, 3 memories, 1 agent, 1 skill, storage_size 1073152 B (matches health)
>
> **Service Layer:** Verified: snake_case outbound mapping (agent_id, turn_count, duration_ms, started_at, updated_at); camelCase inbound
>
> **API Layer:** Verified: isError true/false frames, no success smuggling, no traceback in payloads, stdout purity 4/4
>
> **Frontend Layer:** Verified: all 8 tools parsed; analytics overview matches engine counters
>
> **User Layer:** Verified: acceptance criteria AC-1..AC-11 mapped to observable frames

**Trace (Response):** DB: storage_size 1073152 ↔ analytics storage_size_bytes 1073152 → Service: engine camelCase telemetry mapped (cacheTelemetry→overview counts) with _safe_get → API: stdout: JSON-RPC only (0 bad lines); stderr: structured logs, capped args_summary ≤64 chars → Frontend: client-observable: real data, structured errors, liveness after 14 error-path calls

**21/22** AC passed

---

## 04 · Test Steps Executed

### Findings — Iteration 1 (Auto Bug Loop)

1. **[LOW] AC-TH-001 violation — bare `pytest.raises(Exception)` at `tests/mcp/test_mcp_launcher_wiring.py:218`** (`test_build_services_still_raises_raw_on_engine_open_failure`). The test's documented intent (raw-exception contract of `build_services`) is legitimate, but the engine raises a stable precise type — verified live: `StorageEngine.open` on corrupt dir → `builtins.RuntimeError: Storage error: Corruption: CURRENT file does not end with newline`. Catching `RuntimeError` preserves intent and satisfies AC-TH-001's letter. One occurrence in the whole suite.
2. **[INFO] stderr rich tracebacks for engine-level failures** — `bridge.py:181 logger.exception('bridge_call_failed')` renders rich box-drawing tracebacks (with locals panels) on stderr for engine errors (not-found, invalid variant). Bounded: args_summary ≤64 chars, rich locals truncated, max stderr line 337 chars, content never leaked, stdout pure JSON-RPC. Contradicts the strictest reading of iter-1 security report's 'no traceback' claim for mid-call failure, but no contract requires zero stderr diagnostics for handled engine errors (launch-failure path is traceback-free). Consider logging.exception → logger.error with structured payload.
3. **[INFO] pydantic 2.13 UnsupportedFieldAttributeWarning** — `validation_alias=AliasChoices(...)` in `Field()` in models/agent.py + models/skill.py (5 warnings in suite run). Functional behavior verified intact (tools→capabilities, category→type, engine camelCase→snake_case all validate correctly); warning is a usage-pattern deprecation (Annotated/assignment preferred).
4. **[INFO] Engine pre-lowercases memory content** — 102400-char mixed-case roundtrip returns lowercased content (`'L'*102400` → `'l'*102400`, byte_identical=false); lowercase content is byte-identical. Verified at raw `StorageEngine` level. This is Rust-core REQ-S-003 behavior (out of this feature's scope); bridge bytes-path is sound. Documented for downstream awareness.
5. **[NOTE, out of scope] `contexter status` without CONTEXTER_PATH hit `~/.contexter` LOCK** — caused by another process holding the lock (running OpenCode MCP session), not a product defect; live CLI against a temp engine exits rc=0 with clean status output.

### Test Steps Executed (live, against temporary engines — no user data touched)

1. Started `run_mcp.py` via stdio with `CONTEXTER_PATH`→temp engine, `CONTEXTER_API_KEY=test-key-123`, launch log set.
2. Live `ClientSession` (mcp SDK 1.28, stdio): initialize → server `contexter`, protocol 2025-11-25; tools/list = 8 tools, 4 resource templates with `{?_api_key}`.
3. Called all 8 tools with valid params — all real data (12 seeded sessions, skills, memories, agent, health, export).
4. Read all 4 resources — real payloads; analytics totals match engine health counters.
5. Auth matrix: no-key server (open mode) OK; wrong key → `Invalid API key.`; correct key → OK; resource without `_api_key` → McpError.
6. Error paths: nonexistent session/agent, missing query, extra param, bad UUID, empty content, oversized content (>1,000,000 → rejected), bad format, store_memory extra param, store to nonexistent session → all structured isError, server alive.
7. Limits: limit=5→5, 0→0, −1→0, 1e9→12, None→12 (clamped).
8. Large content: 102400 B store OK, search hits (len 597), 2 concurrent calls OK.
9. Raw JSON-RPC probe: 4/4 stdout frames parse (health, auth-fail, not-found, corrupt-engine), 0 bad lines; stderr captured for hygiene check.
10. Launch failure probe: corrupt data dir → rc=2, 2 clean stderr lines, no traceback, launch log contains diagnostics.
11. Empty engine probe: all list tools return empty collections, analytics zeroed, success frames.
12. CLI live check: `contexter status` against temp engine → rc=0 clean output.
13. Full suite: `pytest tests/` → **794 passed, 0 failures** in 11.64s.
14. Grep audits: `CONtexTER_` in src/ → 0; README + docs/design → 0; `pytest.raises(Exception)` → 1 (finding #1).
15. Resource error shape: valid session read with `_api_key` OK; missing `_api_key` → McpError (auth-first ordering per design).

---

## 05 · Expected vs Actual

| | Description |
|---|---|
| **Expected** | All 8 tools + 4 resources return real engine data over live stdio; auth preserved; structured errors; empty-engine grace; engine-failure containment; no mocks; suite ≥647 green; stdout pure JSON-RPC; no test uses bare pytest.raises(Exception). |
| **Actual** | All of the above verified live — 794 passed / 0 failed. One deviation: AC-TH-001 letter violated by a single deliberate contract test using `pytest.raises(Exception)` (precise type `RuntimeError` is stable and catchable — LOW finding #1). |

**Wireframe comparison:** Design compliance pre-verified by Design Compliance Validator. Quick visual sanity check performed against `plan/preview/preview-mcp-live-fix-approved.md` — MCP surface (8 tools, 4 resources, `_api_key` auth gating, structured isError error shape, camelCase↔snake_case mapping, canonical `CONTEXTER_*` env names, launch-failure clean error) matches the approved preview. No layout/protocol deviations observed (no rendered UI — protocol + dataflow surface compared).

---

_Generated by User-Testing Validator · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
