# User-Testing Review Report

# MCP Server Live-Functionality Repair — User-Testing Review (Auto Bug Loop Iteration 2)

> Repairs the Contexter MCP server so all 8 tools + 4 resources return real data from the real Rust engine through a live stdio MCP client; resolves iter-1 findings (AC-TH-001 bare pytest.raises, bridge stderr tracebacks, pydantic warning, lowercase docs). Scope: contexter-server MCP layer only (REST/CLI/Rust core out of scope).

**Verdict:** CONDITIONAL PASS (class: finding-driven (1 LOW contract violation: AC-EFS-001 letter unmet end-to-end due to unconfigured FastMCP framework logger))

2026-08-02 · 25/26 AC passed · User-Testing Validator

---

## 01 · Test Overview

> **Browser & Environment**
> Linux (Python 3.12.3), FastMCP 3.4.0, MCP SDK 1.28 (stdio client). Server: run_mcp.py stdio transport; temporary engine dirs under /tmp/opencode (12 sessions, 3 memories, 4 agents incl. session-owner, 2 skills) via contexter_core — no user data touched (SPEC DAT-001). Verification via live mcp.client.stdio.ClientSession harness (37/37 checks) + raw JSON-RPC subprocess probes + full pytest suite.

> **Test Summary**
> All iter-2 verification targets exercised live against the real Rust engine: (1) AC-TH-001 test now pins pytest.raises(RuntimeError) at test_mcp_launcher_wiring.py:222 — corrupt dir raises stable RuntimeError 3/3 attempts; (2) bridge engine-failure stderr is ONE concise 224-char structured line (bridge_call_failed ... exception_type=ValueError), full traceback only in diagnostics log file (3046 bytes); (3) 0 UnsupportedFieldAttributeWarning — full suite passes with -W error::pydantic.warnings.UnsupportedFieldAttributeWarning; (4) README documents engine lowercase behavior (REQ-S-003). Full live matrix re-verified: 8 tools + 4 resources, auth matrix, 10 error classes, limit clamping (5/0/-1/1e9), search total present, get_overview counts 3/2, launch failure rc=2 with clean stderr, stdout purity 5/5 JSON-RPC frames. NEW finding: FastMCP's unconfigured framework logger still renders a 2672-char rich traceback box for the same engine failure, making total failure stderr 2897 bytes > 512 (AC-EFS-001 letter).

---

## 02 · Acceptance Criteria Results

| AC | Phase | Status | Evidence |
|---|---|---|---|
| AC-1: 8 tools real data | API | ✅ PASS | Live stdio harness: all 8 tools isError:false; list_recent_sessions returns 12 real sessions (agent_id, turn_count, duration_ms, started_at) |
| AC-2: 4 resources real data | API | ✅ PASS | session/memory/agent/analytics reads OK; overview totals match engine counters (12 sessions, 3 memories, 3 agents, 2 skills, storage bytes == health) |
| AC-3: type filter | API | ✅ PASS | list_skills type=search → only search skill; search_memories type filter honored |
| AC-4: auth preserved | API | ✅ PASS | open mode (no key) OK; wrong key → isError 'Invalid API key.'; correct key → success; resource w/o _api_key → McpError rejected |
| AC-5: store_memory persists | API | ✅ PASS | store_memory → search_memories returns stored snippet; total incremented |
| AC-6: invalid params structured | API | ✅ PASS | not-found/validation/storage → structured isError; no crash; no traceback in response payloads |
| AC-7: empty datasets | API | ✅ PASS | empty engine: sessions [], skills [], search {results:[],total:0}, analytics zeroed |
| AC-8: engine failure contained | API | ✅ PASS | corrupt engine: isError=True, server alive, health OK after errors; launch failure rc=2, 1 clean stderr line, diagnostics in launch log |
| AC-9: no mocks in live path | Static | ✅ PASS | live path returns real engine data; bridge mock-import guard; no stub in live path |
| AC-10: suite green + regression tests | Suite | ✅ PASS | 867 passed, 0 failures (13.77s) — above ≥647 target; EFS + launcher tests present and green |
| AC-11: no stdout pollution | API | ✅ PASS | raw JSON-RPC probe: 5/5 frames parse as JSON (ids [1,2,3,4,5]), 0 bad lines |
| AC-ES-001: not-found error shape | API | ✅ PASS | isError=True, 'Resource not found: <id>', no success frame |
| AC-ES-002: validation error shape | API | ✅ PASS | structured validation isError; never traceback in response |
| AC-ES-003: auth serialization | API | ✅ PASS | MCPAuthError messages unchanged ('API key required...' / 'Invalid API key.') |
| AC-ES-004: error-shape tests | Suite | ✅ PASS | covered by 867-green suite |
| AC-ES-005: survives repeated errors | API | ✅ PASS | error-path calls incl. 102400-B content + concurrency; liveness after all errors |
| AC-BH-001: log hygiene 10KB | API | ✅ PASS | 10KB content never in logs; args_summary capped ≤64 chars; bridge line 224 chars |
| AC-LH-001: launch error clean | API | ✅ PASS | corrupt engine → rc=2, empty stdout, ONE clean stderr line (contexter: engine_open_failed: ... full diagnostics: <log>), NO traceback on stderr; launch log has full traceback (RuntimeError) |
| AC-HO-001: handler observability | API | ✅ PASS | structured logs: call_received (DEBUG), auth_decision (DEBUG), engine_result (DEBUG), handler_error (ERROR), bridge_call_failed (ERROR); INFO lifecycle only; no secrets |
| AC-TH-001: no pytest.raises(Exception) | Suite | ✅ PASS | tests/mcp/test_mcp_launcher_wiring.py:222 pins pytest.raises(RuntimeError) — corrupt engine raises stable RuntimeError (verified live 3/3); bare Exception usage gone |
| AC-EV-002: env canonical src | Static | ✅ PASS | zero CONtexTER_ in src/; CONTEXTER_API_KEY/BRIDGE_POOL_SIZE canonical |
| AC-DN-001: env canonical docs | Static | ✅ PASS | README + docs/design clean; remaining strings only in immutable historical contract reports |
| AC-EFS-001: stderr ≤512 chars/failure, no raw traceback | API | ❌ FAIL | Bridge emits ONE concise 224-char line (verified), but FastMCP framework logger.exception (fastmcp/server/server.py:1297, RichHandler→stderr, propagate=False, unconfigured by feature) renders 2672-char rich traceback box for the same engine failure — total failure section 2897 bytes > 512, with raw traceback + source frames. Feature-controllable: configure fastmcp logger level/filter or raise FastMCPError subclasses (exc_info=False path at server.py:1284-1287) |
| AC-EFS-002: full diagnostics in log file | API | ✅ PASS | diagnostics log contains full traceback (3046 bytes: bridge_call_failed event + Python traceback to RuntimeError/ValueError source) |
| AC-EFS-003: stdout pure | API | ✅ PASS | raw JSON-RPC probe 5/5 frames parse, ids [1,2,3,4,5], 0 bad lines |
| AC-EFS-004: suite green | Suite | ✅ PASS | 867 passed, 0 failures; EFS regression tests present (test_bridge_engine_failure_stderr.py: 13 passed incl. launcher) |

---

## 03 · As-Built End-to-End Data Flow

**Interaction:** MCP client sends JSON-RPC request frame over stdin → FastMCP dispatches to tool/resource handler → handler validates params and auth first → service calls bridge → Rust engine executes on thread pool → response frame over stdout. Engine failure: bridge catches, persists full traceback to diagnostics log file (CONTEXTER_LOG_FILE), emits ONE concise structured stderr line (bridge_call_failed, exception_type, bounded args_summary); FastMCP serialises structured isError result to client; FastMCP framework logger independently renders a rich traceback box on stderr (iter-2 finding).

### Request Track · Forward Flow (1→5)

| Step | Layer | Action |
|---|---|---|
| 1 | User | Live MCP client (SDK 1.28 stdio) emits JSON-RPC initialize then tools/call or resources/read |
| 2 | Frontend | ClientSession writes request frame to server stdin; awaits response frame on stdout |
| 3 | API | run_mcp.py stdio transport: FastMCP parses frame, routes to registered tool/resource handler |
| 4 | Service | Handler validates params (pydantic, limit clamping), requires _api_key first, calls service (analytics mapping, camelization) |
| 5 | Database | Bridge StorageEngine runs engine call in thread pool (8 workers); Rust contexter_core executes query |

**Layer Details (Request):**

> **User Layer:** Live MCP client — this E2E validation used mcp SDK 1.28 stdio_client and raw JSON-RPC subprocess probes
>
> **Frontend Layer:** ClientSession; every tool call and resource read observed at frame level (isError flags, payloads)
>
> **API Layer:** FastMCP 3.4.0 stdio transport on run_mcp.py; tool schema from handlers; {?_api_key} RFC-6570 templates on 4 resources
>
> **Service Layer:** mcp_tools/handlers.py + mcp_tools/auth.py + analytics_service.py; auth-first ordering; error mapping to structured isError
>
> **Database Layer:** core/bridge.py StorageEngine (asyncio.to_thread pool); Rust engine on temp dirs only (DAT-001)

### Response Track · Return Flow (6→10)

| Step | Layer | Action |
|---|---|---|
| 6 | Database | Engine returns typed result (or raises NotFound/ValueError/RuntimeError) |
| 7 | Service | Bridge re-raises; handler catches and maps to HandlerError → isError result with bounded message; bridge persists full diagnostics to launch log |
| 8 | API | FastMCP serialises JSON-RPC result/error frame onto stdout; stderr carries bridge's single concise line + (framework-level box — finding) |
| 9 | Frontend | Client parses response; success payloads contain real data (verified field-by-field) |
| 10 | User | Client receives result (isError=false + payload) or structured error (isError=true, message); server stays alive for subsequent calls |

**Layer Details (Response):**

> **Database Layer:** Verified: 12 sessions, 3 memories, 4 agents (3 visible), 2 skills; storage_size matches health
>
> **Service Layer:** Verified: snake_case outbound mapping (agent_id, turn_count, duration_ms, started_at); camelCase inbound
>
> **API Layer:** Verified: isError true/false frames, no success smuggling, no traceback in payloads, stdout purity 5/5
>
> **Frontend Layer:** Verified: all 8 tools parsed; analytics overview matches engine counters; search total present
>
> **User Layer:** Verified: acceptance criteria mapped to observable frames; error-path liveness confirmed

**Trace (Response):** DB: storage_size ↔ analytics storage_size_bytes → engine camelCase telemetry mapped (_safe_get) → Service: engine error → bridge diagnostics to log file + one concise stderr line (exception_type, method, bounded args_summary) → API: stdout: JSON-RPC only (5/5 frames, 0 bad lines); stderr: bridge line + FastMCP framework box (finding) → Frontend: client-observable: real data, structured errors, liveness after error paths

**25/26** AC passed

---

## 04 · Test Steps Executed

### Findings — Iteration 2 (Auto Bug Loop)

1. **[LOW] AC-EFS-001 letter violation at the framework level** — `fastmcp/server/server.py:1297` `logger.exception(f"Error calling tool {name!r}")` runs for every tool error (generic `except Exception` path; HandlerError and MCPAuthError are ValueError subclasses, not FastMCPError). FastMCP's logger namespace (`fastmcp.*`, `propagate=False`, `RichHandler(console=Console(stderr=True))`) is NOT configured by the feature (`contexter_server/__init__.py` only sets root stdlib INFO; run_mcp.py sets no fastmcp log level). Measured: bridge line 224 chars (fixed, verified) + FastMCP box 2672 chars = 2897 bytes for one engine failure — exceeds AC-EFS-001's ≤512-char total and contains a raw traceback with source frames. 9 framework boxes observed across error classes (validation, auth, engine). No content leak, stdout pure, diagnostics channel works — but the AC's letter is about total stderr for the failure, which the server does not yet control. Controllable: configure fastmcp logger level/filter, or make HandlerError/MCPAuthError subclass FastMCPError with a low log_level (server.py:1284-1287 logs FastMCPError with exc_info=False).
2. **[INFO] Probe artifact note (not a defect)** — raw JSON-RPC probes using char-by-char reads under-captured frames; line-based reader-thread probe confirmed 5/5 frames (ids 1-5) parse as JSON. Stdout purity holds.

### Test Steps Executed (live, against temporary engines — no user data touched)

1. Verified iter-1 findings fixed: AC-TH-001 test at test_mcp_launcher_wiring.py:222 pins `pytest.raises(RuntimeError)` (corrupt engine raises stable RuntimeError — verified live 3/3 attempts, rc=2 launch failure).
2. Bridge engine-failure stderr: single concise 224-char line (`bridge_call_failed ... args_summary="('not-a-uuid',)" diagnostics_log=... exception_type=ValueError method=get_session`); full traceback only in launch log (3046 bytes); stdout pure.
3. Full suite with `-W error::pydantic.warnings.UnsupportedFieldAttributeWarning`: 867 passed, 0 failures (13.77s) → 0 UnsupportedFieldAttributeWarning.
4. README.md:248-256 documents engine lowercase behavior (REQ-S-003) — verified present.
5. Live stdio harness (mcp SDK 1.28): initialize → tools/list = 8 tools, 4 resource templates; all 8 tools with real data; all 4 resources; store_memory roundtrip; auth matrix (open/wrong/correct key); 10 error classes; limit clamping (5→5, 0→0, −1→0, 1e9→12); search total present (total=4); get_overview counts (3 agents, 2 skills); liveness after all error paths — 37/37 checks.
6. Launch failure probe: corrupt dir → rc=2, empty stdout, ONE clean stderr line with diagnostics path, full RuntimeError traceback in launch log.
7. stdout purity (line-based reader thread): 5/5 JSON-RPC frames, ids [1,2,3,4,5], 0 bad lines.
8. Observability: at default (DEBUG-off) stderr shows INFO lifecycle + ERROR events only; per-call logs (call_received/auth_decision/engine_result) at DEBUG — zero DEBUG events at default, ERROR for handler_error/bridge_call_failed; no secrets in logs.
9. Grep audits: `pytest.raises(Exception)` → 0 in tests/; `CONtexTER_` in src/ → 0.
10. EFS + launcher regression tests: 13 passed in 0.99s (incl. pinned RuntimeError test and bounded-stderr tests).

---

## 05 · Expected vs Actual

| | Description |
|---|---|
| **Expected** | All iter-1 findings resolved: AC-TH-001 pinned to RuntimeError; bridge engine-failure stderr ≤512 chars/no raw traceback end-to-end with full diagnostics in log file; stdout pure; 0 UnsupportedFieldAttributeWarning; README documents lowercase; full live matrix green; suite ≥647 passing. |
| **Actual** | Bridge-level fix verified correct (single 224-char line, diagnostics in log, stdout pure 5/5); AC-TH-001 fixed; 0 pydantic warnings; 867 passed / 0 failed. One deviation: FastMCP's unconfigured framework logger still renders a 2672-char rich traceback box for the same engine failure, so total failure stderr is 2897 bytes with a raw traceback — AC-EFS-001's letter (≤512 chars, no raw traceback) is unmet end-to-end. 25/26 ACs pass. |

**Wireframe comparison:** Design compliance pre-verified by Design Compliance Validator (iter-2 report). Quick visual sanity check performed against plan/preview/preview-mcp-live-fix-approved.md — MCP surface (8 tools, 4 resources, `_api_key` auth gating, structured isError error shape, camelCase↔snake_case mapping, canonical CONTEXTER_* env names, launch-failure clean error) matches the approved preview. No layout/protocol deviations observed (no rendered UI — protocol + dataflow surface compared).

---

_Generated by User-Testing Validator · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix_
