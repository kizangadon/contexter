# SPEC Compliance Review Report

# MCP Server Live-Functionality Repair (mcp-live-fix)

> SPEC Compliance audit: every REQ-XXX / CON-XXX / interface contract item in `docs/contracts/2026-08-01-mcp-live-fix/SPEC.md` mapped to implementation code, tests, and T6 live-verification evidence. Acceptance criteria AC-001..AC-011 and EDGE_CASES EC-001..EC-021 verified against the working tree on `feature/mcp-live-fix` (no commits; 25 working-tree entries).

**Verdict:** CONDITIONAL PASS (class: PARTIAL — 2 HIGH implementation gaps remain)

2026-08-01 · 12/15 requirements fully matched · SPEC Compliance Validator

---

## 01 · SPEC Requirements Coverage

| ID | Requirement | Verdict | Evidence (file:line / test / T6) |
|---|---|---|---|
| REQ-001 | All 8 MCP tools return **real data** live; no mock/stub/placeholder | 🟡 PARTIAL — **6/8 tools pass live; `get_agent_info` and `list_skills` fail** | `run_mcp.py:31-46` (bridge wiring); `bridge.py:98-175`; `mcp_tools/handlers.py:140-174`; `agent_service.py:16` / `skill_service.py:16` (`model_validate` without engine-schema translation); T6 live matrix rows #5/#6 (pydantic ValidationError) |
| REQ-002 | All 4 MCP resources resolve real data via URIs | 🟡 PARTIAL — **3/4 pass live; `contexter://agent/{id}` fails** (McpError wraps F1) | `mcp_server.py:204,216,228,240` (`{?_api_key}` templates); `handlers.py:222-287`; T6 matrix rows #9-#12 |
| REQ-003 | Registered tool schema matches handler signature exactly | ✅ IMPLEMENTED | `mcp_server.py:109-124,165-174` (`type` forwarded); `handlers.py:68,159` (`type` param restored); `tests/mcp/test_mcp_type_filter_live.py:127-146` (`TestRegisteredSchema` asserts `type` present, `type_filter` absent) |
| REQ-004 | `_api_key` auth pattern preserved (optional, `require_api_key()`, backward compat) | ✅ IMPLEMENTED | `mcp_tools/auth.py:25-58` (hmac.compare_digest, unset→open at L46-47); canonical `CONTEXTER_API_KEY` in `auth.py:45`, `mcp_server.py:68`, `api/deps.py:51`, `main.py:111`; `tests/mcp/test_mcp_auth.py` (`TestEnvVarCanonicalName`); T6 auth matrix (6/6 ✅) |
| REQ-005 | Live stdio server starts cleanly; no tracebacks to stdout | ✅ IMPLEMENTED | `run_mcp.py:58-66` (stderr-only prints); no `print()` in mcp_server/handlers/auth/bridge; T6 stdout purity (all frames parsed JSON-RPC, structlog on stderr) |
| REQ-006 | Existing suite (≥579 incl. 59 MCP) green; new tests per repaired failure mode | ✅ IMPLEMENTED *(1 observation)* | T6: **647 passed / 1 pre-existing failure** (`test_lifespan_shutdown_joins_thread`, SSE/lifespan scope, proven pre-existing via stash); 28 new test functions across 5 new files (see §02) |
| REQ-007 | Error conditions → structured MCP tool errors; no crashes/tracebacks | 🟡 PARTIAL | Auth errors → `MCPAuthError(ValueError)` → JSON-RPC error ✅; missing required param → FastMCP protocol error ✅; engine failure → isError, process survives (T6 16+ calls; `tests/core/test_bridge.py:592-598,649-659`) ❌ **but** handler not-found returns **success-result** `{"error": "not found"}` (handlers.py:114,153,235,271) — not an `isError` result; message ≠ documented `Resource not found: <id>` |
| CON-001 | DDD — MCP layer stays a thin adapter over domain services | ✅ IMPLEMENTED | `run_mcp.py:39-46` (six domain services); `handlers.py` delegates only; no business logic added to handlers |
| CON-002 | Fix via TDD (reproducing tests first) | ✅ IMPLEMENTED | RED: 12 failures on stub → GREEN: new tests pass; new tests fail on pre-fix code by construction (`test_engine_real.py` asserts stub absent; `test_mcp_type_filter_live.py` fails on `type_filter` drift; `test_mcp_resource_auth_live.py` fails without `{?_api_key}`) |
| CON-003 | Observability: logs on entry/success/failure, no sensitive data | ✅ IMPLEMENTED | `bridge.py:166` (`bridge_call_failed`), `bridge.py:169-174` (`bridge_call_end` + duration), `_truncated_args_summary` (bridge.py:56-95) avoids large/sensitive payload logging; `auth.py:50,57` (missing/invalid key); `mcp_server.py:70-74` |
| GUD-001 | Boring, obvious fix — no redesign | ✅ IMPLEMENTED | Stub deleted (`git status`: `D contexter-server/src/contexter_core.py`); bridge reused/hardened (+87 lines); handler params restored; no architectural churn |
| PLT-001 | FastMCP version behavior verified and pinned | ✅ IMPLEMENTED | `contexter-server/pyproject.toml`: `fastmcp~=3.4.0` (was `>=0.3`); schema-introspection regression test `test_mcp_type_filter_live.py:127-146` |
| PLT-002 | `_SYNC_ENGINE_CLASS` validation — never MagicMock dispatch | ✅ IMPLEMENTED | `bridge.py:29` (class capture), `bridge.py:134-156` (mock class/instance rejection, `json.loads` boundary); `tests/core/test_bridge_mock_rejection.py` (3 tests); `tests/core/test_bridge.py:584-587` (`test_invalid_method_raises`) |
| DAT-001 | Live verification against temp engine, not user data | ✅ IMPLEMENTED | T6 temp-engine verification; `test_engine_real.py` / `test_mcp_launcher_wiring.py` all use `tempfile.TemporaryDirectory` |
| EXT-001 | OpenCode stdio subprocess launch works | ✅ IMPLEMENTED | T6 live stdio subprocess: 12-call matrix + auth matrix + discovery (8 tools, 4 templates) |

**Requirement tally:** 15 total · 12 fully implemented · 3 partial (REQ-001, REQ-002, REQ-007) · 0 missing.

---

## 02 · Implementation Mapping

| Implementation artifact | Location | Guards which requirement |
|---|---|---|
| Real Rust wheel resolvable (stub deleted) | `git status` `D src/contexter_core.py`; `test_engine_real.py:65-105` (extension `.so` present, zero Mock attrs on `Engine`); env: `contexter_core` → `site-packages`, `Engine` is `builtins` type | REQ-001, AC-9 |
| Launcher rewired through bridge | `run_mcp.py:31-46`; `test_mcp_launcher_wiring.py:18-44` (six services, each holds `StorageEngine`, engine not a Mock) | REQ-001, AC-9 |
| Bridge hardened (camelize + mock-reject) | `bridge.py:36-53` (`_snake_to_camel`/`_camelize_payload_keys`), `bridge.py:130-175` (`_run` guards); `test_bridge_mock_rejection.py:33-62`; `test_bridge.py` camelCase assertion updates (L138-143, L238-251, L265-268, L509-513) | REQ-001, PLT-002, EC-019 |
| Memory service translation | `services/memory_service.py:21` (`memory_type` default), `:46-49` (`query`→`keywords`, `type`→`memory_type`), `:56-60` (limit/offset) | REQ-001 (search_memories live ✅) |
| `type` param restored | `handlers.py:68,159`; `mcp_server.py:111,166`; `test_handlers_type_filter.py` (rename `type_filter=`→`type=`), `test_mcp_server.py:187,358`, `test_mcp_type_filter_live.py:41-124` | REQ-003, AC-3, EC-004 |
| `CONTEXTER_API_KEY` canonicalized | `auth.py:45`, `mcp_server.py:68,73`, `api/deps.py:51`, `main.py:111`; `test_mcp_auth.py` (`TestEnvVarCanonicalName`), `test_security.py`, `test_mcp_server.py:565,677` | REQ-004, EC-013/014 |
| `{?_api_key}` resource templates | `mcp_server.py:204 (session), 216 (memory), 228 (agent), 240 (analytics)`; `test_mcp_resource_auth_live.py` (9 tests) | REQ-004, AC-2/4, EC-013 |
| fastmcp pin + gitignore | `pyproject.toml` `fastmcp~=3.4.0`; `.gitignore:32-33` `**/docs/tests/` | PLT-001, hygiene |

**New test files (28 test functions):** `tests/core/test_bridge_mock_rejection.py` (3) · `tests/core/test_engine_real.py` (8) · `tests/mcp/test_mcp_launcher_wiring.py` (3) · `tests/mcp/test_mcp_resource_auth_live.py` (9) · `tests/mcp/test_mcp_type_filter_live.py` (5).

---

## 03 · Unmatched Requirements

None — every REQ/CON/GUD/PLT/DAT/EXT has implementation code and at least one test or live-verification evidence. Zero MISSING (🔴) items.

---

## 04 · Partially Matched Requirements

| ID | Gap | Evidence |
|---|---|---|
| **REQ-001 / AC-1** | `get_agent_info` + `list_skills` return pydantic `ValidationError` on **real** engine data. Engine agent payload `{id, name, type, description, capabilities, status, config, version, createdAt, updatedAt}` lacks `provider`/`model` required by pydantic `Agent` (`models/agent.py:19-20`); engine skill payload uses `category` + int `version`, but `Skill` requires `type: str`, `version: Optional[str]` (`models/skill.py:13-14`). `AgentService.get/list` and `SkillService.list` call `model_validate` with **no translation layer** (`agent_service.py:16,24`; `skill_service.py:16,24`) — contrast with `memory_service.py` which has one. | T6 live matrix #5 (get_agent_info ❌), #6 (list_skills ❌); T6 Finding 1 & Finding 2 (HIGH) |
| **REQ-002 / AC-2** | `contexter://agent/{id}` fails live (McpError wrapping F1). Template/auth wiring is correct; the failure is the same Agent model↔engine drift. | T6 live matrix #11 ❌ |
| **REQ-007 / AC-6 / EC-001** | Nonexistent-ID handlers return `{"error": "not found"}` / `"X not found"` as a **successful** result payload, not an MCP `isError` result, and the message differs from the documented `Resource not found: <id>` (EDGE_CASES Error Messages table). No crash/traceback (the spec's core intent) is satisfied; the error *shape* clause of REQ-007/AC-6 is not. | `handlers.py:114,153,235,271,298` |
| **AC-3** | Schema-drift half fixed (type accepted — live tests ✅); "return filtered **real** data" half fails for `list_skills` on non-empty engines due to F2. `search_memories` with `type` works live. | `test_mcp_type_filter_live.py:45-62` (passes, mocked service); T6 matrix #2 |
| **AC-7** | Analytics zeroed-overview on live ✅; empty-engine **list tools** have no dedicated automated or live test (code path would return `[]`; `list_skills`/`get_agent_info` cannot be exercised on real non-empty data at all due to F1/F2). | T6 matrix #12 (Observation 3) |

---

## 05 · Constraint Violations

| Constraint | Status |
|---|---|
| CON-001 DDD thin adapter | ✅ No violation — handlers delegate to domain services |
| CON-002 TDD | ✅ RED/GREEN evidence present; new tests fail on unfixed code by construction |
| CON-003 Observability | ✅ Logs on entry/success/failure with truncated args; no secrets |
| Out-of-scope boundaries (REST/CLI/Rust core/UI/auth model) | ✅ No out-of-scope changes (env-var canonicalization in `deps.py`/`main.py` is the documented Fix B hygiene item, in-scope) |

---

## 06 · Edge Case Verification

| EC | Scenario | Verdict | Evidence |
|---|---|---|---|
| EC-001 | Nonexistent session/agent ID → structured error, no crash | 🟡 PARTIAL | Process survives (T6 16+ calls ✅) but error is success-payload `{"error": "not found"}` (handlers.py:114,153), not `isError`; message ≠ `Resource not found: <id>`; **agent resource additionally fails on existing IDs (F1)** |
| EC-002 | `search_memories` without `query` → structured validation error | ✅ IMPLEMENTED | `query` required in wrapper (mcp_server.py:110) → FastMCP protocol error |
| EC-003 | Unknown extra params tolerated/structured — never TypeError traceback | ✅ IMPLEMENTED | FastMCP schema validation rejects; `type` drift class covered by `test_mcp_type_filter_live.py` |
| EC-004 | `type` filter accepted (skills/memories) | ✅ IMPLEMENTED | handlers.py:68,159; mcp_server.py:111,166; 5 live-wrapper tests |
| EC-005 | `limit` beyond data → min(limit, count) | ✅ IMPLEMENTED | `handlers.py:134-135` (`sessions[:limit]`) |
| EC-006 | `store_memory` empty content → validation error, nothing persisted | 🟡 PARTIAL | `MemoryCreate.content: str` has **no min_length** (`models/memory.py:21`) — empty content accepted and persisted; no validation |
| EC-007 | Empty engine → empty lists, zeroed overview, success | 🟡 PARTIAL | Overview zeroed ✅ (T6 #12); list-tools empty path unverified by test |
| EC-008 | Large memory ≥102400 bytes → bytes path | ✅ IMPLEMENTED | `bridge.py:213-223` (`create_memory_bytes`); `test_bridge.py:252-268` |
| EC-009 | `limit=0`/negative → no-limit or clamp | 🟡 PARTIAL | `sessions[:0]` → `[]`; `sessions[:-1]` drops last item — not clamped (P3) |
| EC-010 | Unsupported `export_data` format → structured error/fallback | 🟡 PARTIAL | `ExportRequest.format: str = "json"` plain str, no enum (`models/export.py:17`); unsupported value passes through; no test |
| EC-011 | Engine path unopenable at launch → clear stderr exit, no hang | 🟡 PARTIAL | ImportError guard gives clear message (`bridge.py:20-25`); `Engine.open` failure propagates as raw traceback to stderr (`bridge.py:124`, no catch in `run_mcp.py`) — exits, no hang, but not a curated message; untested |
| EC-012 | Engine op raises mid-call → structured error, process survives | ✅ IMPLEMENTED | `bridge.py:165-167` (log+re-raise) → FastMCP isError; T6 process survival; `test_bridge.py:592-598,649-659` |
| EC-013 | Key set + wrong/missing `_api_key` → reject | ✅ IMPLEMENTED | `auth.py:49-58`; `test_mcp_auth.py`; `test_mcp_resource_auth_live.py`; T6 auth matrix |
| EC-014 | Key unset + no `_api_key` → succeed | ✅ IMPLEMENTED | `auth.py:46-47`; `test_mcp_auth.py:14-18`; T6 auth matrix |
| EC-015 | Wrong JSON-RPC payload → protocol error, alive | ⚠️ UNVERIFIED | No test (P2; FastMCP protocol handling by design) |
| EC-016 | FastMCP missing → clear stderr exit | ✅ IMPLEMENTED | `run_mcp.py:58-60` (stderr message, exit 1); `mcp_server.py:59-63` |
| EC-017 | Concurrent tool calls, no frame corruption | ⚠️ UNVERIFIED | No concurrency test (P2) |
| EC-018 | Concurrent `store_memory` same session | ⚠️ UNVERIFIED | No test (P3; bridge thread-pool serialization design) |
| EC-019 | Bridge/engine method mismatch → structured; never MagicMock await | ✅ IMPLEMENTED | `bridge.py:134-156`; `test_bridge_mock_rejection.py` (3); `test_bridge.py:584-587`; `test_engine_real.py:80-101` |
| EC-020 | FastMCP version behavior → pin/align | ✅ IMPLEMENTED | `fastmcp~=3.4.0` pin + schema test |
| EC-021 | Client disconnects mid-call | ⚠️ UNVERIFIED | No test (P3) |

---

## 07 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | ⚠️ T6 documented F1/F2/F3 for the Auto Bug Loop; not yet contracted at time of this audit |
| Zero findings are being silently deferred to a future iteration | ✅ All gaps listed below are explicit findings in this report |

---

## 08 · Summary

> **SPEC Compliance Assessment**
> Fix A (engine path) and Fix B (schema drift) are real, well-tested, and prove the root-cause repairs: the MagicMock stub is deleted, the real Rust wheel is wired through the hardened `StorageEngine` bridge, `type` is restored in the frozen contract, `CONTEXTER_API_KEY` is canonicalized, resource templates carry `{?_api_key}`, fastmcp is pinned, and 28 new tests cover the repaired failure modes. 12/15 requirements fully met. **However, REQ-001 ("all 8 tools return real data") and REQ-002 ("all 4 resources resolve real data") are not fully satisfied**: `get_agent_info` and `list_skills` (and the agent resource) fail live with pydantic `ValidationError` because the Agent/Skill domain models do not match the engine's payload contract and — unlike MemoryService — AgentService/SkillService have no translation layer. The frozen SPEC's central acceptance clause (all tools + resources live) is therefore only 9/12 (per T6 matrix) and the 2 HIGH findings map directly to spec non-compliance.

> **Findings**
> 1. **HIGH (F1)** — `get_agent_info` + `contexter://agent/{id}` fail live: pydantic `Agent` requires `provider`/`model`; engine returns `type/description/capabilities/...`. Affects REQ-001, REQ-002, AC-1, AC-2, EC-001.
> 2. **HIGH (F2)** — `list_skills` fails live: `Skill` requires `type: str`, engine returns `category` (int `version`). Affects REQ-001, AC-1, AC-3.
> 3. **MEDIUM** — Error-shape deviation: nonexistent-ID returns success-payload `{"error": "not found"}` rather than `isError`; message ≠ documented `Resource not found: <id>`. Affects REQ-007, AC-6, EC-001.
> 4. **LOW (F3)** — `contexter://analytics/overview` returns all-zero counters despite seeded data (T6 observation).
> 5. **LOW** — EC-006: `store_memory` empty content not validated (`MemoryCreate.content` has no min_length).
> 6. **LOW** — EC-009: `limit` 0/negative slicing not clamped.
> 7. **LOW** — EC-011: engine-open failure at launch produces raw traceback (stderr), no curated message; untested.
> 8. **LOW** — AC-7: no dedicated empty-engine list-tool test; REQ-006: suite is 647/648 (1 pre-existing failure, SSE/lifespan, proven pre-existing, out of scope).
> 9. **INFO** — EC-010 (unsupported export format), EC-015 (bad JSON-RPC), EC-017/018 (concurrency), EC-021 (disconnect) untested.

---

## 09 · Final Verdict

| Criterion | Result |
|---|---|
| All REQ-XXX matched with implementation code | ❌ 3 of 7 partial (REQ-001, REQ-002, REQ-007) |
| All CON-XXX constraints respected | ✅ 3/3 |
| All EDGE_CASES covered by implementation or tests | ❌ 13/21 fully; 6 partial; 4 unverified |
| Carryover declaration clean | ⚠️ F1/F2/F3 pending bug contracts |
| **Overall** | **CONDITIONAL PASS — 80% (12/15) requirements fully implemented; 2 HIGH gaps (F1/F2) block REQ-001/REQ-002/AC-1/AC-2 full compliance** |

---

_Generated by SPEC Compliance Validator · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
