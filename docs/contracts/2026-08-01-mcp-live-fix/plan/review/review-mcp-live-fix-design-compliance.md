# Design Compliance Review Report

# MCP Server Live-Functionality Repair (mcp-live-fix)

> Design compliance audit: approved design preview (`plan/preview/preview-mcp-live-fix-approved.md`, v1.0.0, FROZEN) vs implementation on `feature/mcp-live-fix` (HEAD 27e031d + working tree).

**Verdict:** CONDITIONAL PASS (class: DEVIATIONS — 2 HIGH, 1 MEDIUM, 2 MINOR, 1 INFO)

2026-08-01 · 21/28 design elements verified · Design Compliance Validator

---

## 01 · Design Preview Sections Covered

| Section | Covered | Notes |
|---|---|---|
| Architecture (Mermaid) | ✅ | Client → launcher → FastMCP → handlers → auth → services → bridge → Rust engine |
| Frozen Components table | ✅ | 6 component files + contracts |
| Data Flow Sequence (6 steps) | ✅ | initialize/tools-list → call → auth → service → bridge → result |
| API Contract — Tools (8) | ✅ | Schema vs handler signature vs frozen table |
| API Contract — Resources (4) | ✅ | URI + handler + require_api_key |
| Success / Error JSON shapes | ✅ | result.content[0].text; -32602 structured |
| UI Wireframe | N/A | Backend repair contract — no wireframe in preview; frozen component table serves as structural contract |

---

## 02 · Architecture Compliance

Design (Mermaid, preview L43-71): `OC[OpenCode MCP Client] --stdio JSON-RPC--> RL[run_mcp.py] --> MCP[create_mcp_server FastMCP] --> H[handlers.py 8 tools + 4 resources] --> A[auth.py require_api_key]; H --> SVC[6 services]; SVC --> BR[bridge.py StorageEngine]; BR --asyncio.to_thread--> ENG[Rust Engine]; ENG --> DB[Engine store]`.

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Launcher wiring | `run_mcp.py` — real services wired to engine; stdio transport; clean stderr only | `run_mcp.py` L31-46 `build_services()` constructs all 6 real services on `StorageEngine(engine_path)`; L69 `mcp.run(transport="stdio")`; SSE only prints to stderr (L66); engine path from `CONTEXTER_PATH` (L51). Old raw `Engine.open()` wiring removed (diff). | ✅ MATCH |
| FastMCP factory | `mcp_server.py` registers 8 tools + 4 resources; schemas match handler signatures | `mcp_server.py` L85-198: 8 `@mcp.tool()`; L204-248: 4 `@mcp.resource()`; every wrapper forwards exactly the handler kwargs (incl. `type=`, `_api_key=`); ImportError guard returns None (L59-63). | ✅ MATCH |
| Handler layer | `handlers.py` — real data; structured errors | `handlers.py` — 8 tool handlers (L20-216) + 4 resource handlers (L222-287); every handler calls `require_api_key()` first; delegates to service kwargs; "not connected to storage" guard. | ✅ MATCH |
| Auth layer | `auth.py` — `require_api_key()`, `_api_key` kw-only (BUG-019/028/029) | `auth.py` L25-58: `require_api_key` — env `CONTEXTER_API_KEY` (L45, renamed from typo `CONtexTER_API_KEY`); `hmac.compare_digest` constant-time (L56); `MCPAuthError(ValueError)` → FastMCP serialises as clean JSON-RPC error. All handlers/resource handlers call it. | ✅ MATCH (see INFO-1 for rename) |
| Services layer | 6 services; domain layer modified only on proven defect | `MemoryService`, `SessionService`, `AgentService`, `SkillService`, `AnalyticsService`, `ExportService` all exist in `services/*.py`; only `memory_service.py` modified (+14) — query-vocabulary translation (`query→keywords`, `type→memory_type`) proven by T2 schema-drift investigation. | ✅ MATCH |
| Bridge | `_SYNC_ENGINE_CLASS` validation; no MagicMock in dispatch; bytes path ≥102400 | `bridge.py` L29 `_SYNC_ENGINE_CLASS = _SyncEngine` (captured before patching); L134-156 class+instance Mock rejection raises `TypeError`; L31 `_LARGE_CONTENT_THRESHOLD = 102_400`; L214-220 `create_memory_bytes` / L238-245 `update_memory_bytes` bytes path. Stub `contexter-server/src/contexter_core.py` deleted (staged); ImportError guard L18-25 refuses to run on mocks. | ✅ MATCH |
| Rust engine / store | `contexter_core` real extension → engine store | `test_engine_real.py` asserts compiled `.so` extension, no Mock attrs; `Engine.open` real. Engine persists to RocksDB store. | ✅ MATCH |
| Dispatch mechanism | `asyncio.to_thread` (diagram L69, step 5) | `bridge.py` L163-164: `loop.run_in_executor(self._pool, fn, *args)` with `ThreadPoolExecutor` — functionally identical thread offload, not `asyncio.to_thread` by name. | 🟡 DEVIATION (MINOR) — see DEV-4 |

**Architecture findings:** All boxes and arrows of the Mermaid diagram map to real code. One mechanism-level deviation (run_in_executor vs asyncio.to_thread — equivalent semantics, pre-existing code, not changed by this feature). The no-mock contract (REQ-001/AC-007) is enforced in three layers: stub deletion, import guard, `_SYNC_ENGINE_CLASS`+instance Mock rejection in `_run`.

---

## 03 · API Contract Compliance

Frozen tools table (preview L153-162) vs registered FastMCP schemas (`mcp_server.py`) vs handler signatures (`handlers.py`):

| Tool | Design Parameters (frozen) | Registered Schema / Handler Signature | Status |
|---|---|---|---|
| `get_system_health` | `_api_key?` | `_api_key?: str|None` (mcp_server.py L177-184; handlers.py L177-181) | ✅ MATCH |
| `list_recent_sessions` | `project?`, `limit?`, `_api_key?` | `limit?`, `project?`, `_api_key?` (L139-150; handlers.py L119-125) — same param set | ✅ MATCH |
| `get_session` | `id` (req), `_api_key?` | `id: str`, `_api_key?` (L127-136; handlers.py L101-106) | ✅ MATCH |
| `get_agent_info` | `id` (req), `_api_key?` | `id: str`, `_api_key?` (L153-162; handlers.py L140-145) — schema OK, **result path fails on real engine data (DEV-1)** | 🟡 PARTIAL |
| `list_skills` | `type?`, `_api_key?` | `type: str|None`, `_api_key?` (L165-174; handlers.py L158-163) — schema OK, **result path fails on real engine data + filter dropped (DEV-2)** | 🟡 PARTIAL |
| `search_memories` | `query` (req), `type?`, `project?`, `limit?`, `_api_key?` | `query: str`, `type?`, `project?`, `limit?`, `_api_key?` (L109-124; handlers.py L66-74) — exact | ✅ MATCH |
| `store_memory` | `session_id` (req), `role` (req), `content` (req), `_api_key?` | `session_id`, `role`, `content`, **+ `tokens?`, `tokenizer?`, `model?`** (L86-106; handlers.py L20-31) | 🟡 MINOR DEVIATION — DEV-5 (extra additive params) |
| `export_data` | `format?`, `entities?`, `_api_key?` | `format: str|None`, `entities: list[str]|None`, `_api_key?` (L187-198; handlers.py L196-202) | ✅ MATCH |

Resources (preview L166-171):

| URI | Design | Actual | Status |
|---|---|---|---|
| `contexter://session/{id}` | handler `handle_session_resource`, require_api_key | `mcp_server.py` L204 `"contexter://session/{id}{?_api_key}"` → `handle_session_resource` (handlers.py L222-237) calls `require_api_key` | ✅ MATCH (URI extends with RFC-6570 `{?_api_key}` query block — required to satisfy the Auth column; T3 evidence: `test_mcp_resource_auth_live.py`) |
| `contexter://memory/{id}` | `handle_memory_resource`, require_api_key | L216 + `{?_api_key}` → `handle_memory_resource` (handlers.py L240-255) calls `require_api_key` | ✅ MATCH |
| `contexter://agent/{id}` | `handle_agent_resource`, require_api_key | L228 + `{?_api_key}` → `handle_agent_resource` (handlers.py L258-273) calls `require_api_key` — **result path fails on real engine data (DEV-1)** | 🟡 PARTIAL |
| `contexter://analytics/overview` | `handle_analytics_overview_resource`, require_api_key | L240 + `{?_api_key}` → `handle_analytics_overview_resource` (handlers.py L276-287) calls `require_api_key` | ✅ MATCH |

Success shape (preview L175-181): `result.content[0].text` containing real data — FastMCP 3.4 (`fastmcp~=3.4.0` pinned, pyproject diff) serialises handler dict returns as `text` content; verified in `test_mcp_type_filter_live.py` (`result.content[0].text`). ✅ MATCH.

Error shape (preview L185-191): `{"code": -32602, "message": "Missing required parameter: query"}` — FastMCP generates `InvalidParams` (-32602) for missing required schema params; `MCPAuthError(ValueError)` yields clean JSON-RPC error. Note: the pydantic `ValidationError` surfaced by DEV-1/DEV-2 is NOT the frozen structured shape — it becomes an internal server error class. ❌ for agent/skill paths (consequence of DEV-1/DEV-2).

**API findings:** 6/8 tools exactly match the frozen parameter table; 4/4 resource URIs registered with auth handlers. `store_memory` advertises 3 extra optional params. `get_agent_info`/`list_skills`/agent resource meet schema+auth contract but cannot return real data (DEV-1/DEV-2).

---

## 04 · UI Wireframe Compliance

N/A — this is a backend MCP repair contract; the design preview contains no UI wireframe. The structural contract (frozen components table, preview L75-82) is verified in Section 02. Each frozen file exists and fulfils its contract:

| Component | File | Contract | Status |
|---|---|---|---|
| Launcher | `run_mcp.py` | Real services wired; stdio; clean stderr | ✅ MATCH |
| MCP server | `mcp_server.py` | 8 tools + 4 resources; schemas ≡ handler signatures | ✅ MATCH (DEV-5 minor) |
| Handlers | `mcp_tools/handlers.py` | Real data; accept all schema params; structured errors | 🟡 PARTIAL (DEV-1/2/3) |
| Auth | `mcp_tools/auth.py` | `require_api_key()`, `_api_key` kw-only | ✅ MATCH (INFO-1) |
| Bridge | `core/bridge.py` | `_SYNC_ENGINE_CLASS` validation; no mock dispatch; bytes ≥102400 | ✅ MATCH |
| Services | `services/*.py` | Domain layer | ✅ MATCH (memory_service justified) |

---

## 05 · Data Flow Compliance

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. Client connects and lists tools | `initialize` + `tools/list` return schemas aligned to handler signatures | `run_mcp.py` L69 stdio; `mcp_server.py` 8 tools registered from handler kwargs; schema-registration tests (`test_mcp_type_filter_live.py` TestRegisteredSchema) assert `type` (never `type_filter`) in schemas | ✅ MATCH |
| 2. Client invokes a tool | Schema-validated args reach handler; no `TypeError` (`unexpected keyword argument` eliminated) | Wrappers forward `type=type` (mcp_server.py L117-122, L170-173); `test_mcp_type_filter_live.py` proves `call_tool("list_skills", {"type": "mcp"})` and `search_memories {"type": "memory"}` succeed through the real FastMCP client | ✅ MATCH |
| 3. Handler validates auth | `_api_key` via `require_api_key()` when `CONTEXTER_API_KEY` set; open mode when unset | Every handler L1 call `require_api_key(_api_key)`; auth.py L45-58; tests `test_mcp_auth.py` + `test_mcp_resource_auth_live.py` (correct key ok / missing / wrong rejected) | ✅ MATCH |
| 4. Handler delegates to real service | Every handler calls its real service instance — never a mock | `run_mcp.py` L38-46 builds 6 real services on `StorageEngine`; `test_mcp_launcher_wiring.py` asserts every `service._engine` is a `StorageEngine` and never a `unittest.mock.Mock` | ✅ MATCH |
| 5. Service → Bridge → Engine | Bridge dispatches sync Rust call via `asyncio.to_thread` with `_SYNC_ENGINE_CLASS` method-existence validation | `bridge.py` L134-156 class/instance Mock rejection + method existence; dispatch L163-164 `loop.run_in_executor(self._pool, fn, *args)` (thread offload; mechanism differs from design wording — DEV-4); real round-trips in `test_engine_real.py`, `test_mcp_launcher_wiring.py` | 🟡 DEVIATION (MINOR) — DEV-4 |
| 6. Real result returned | Engine data marshals as JSON-RPC result; errors → structured `isError`; process survives; stdout clean | Dict returns serialised to `content[0].text`; `MCPAuthError(ValueError)` → clean error; empty-engine results in tests. **Exception: agent/skill paths raise pydantic `ValidationError` (unstructured internal error) — DEV-1/DEV-2.** stdout: only FastMCP frames (no prints in server path) | 🟡 PARTIAL (DEV-1/2/3) |

---

## 06 · Compliance Matrix — Design Element → Implementation Evidence

| # | Design Element | Evidence (file:line) | Status |
|---|---|---|---|
| 1 | 8 tools + 4 resources registered | `mcp_server.py:85-198, 204-248` | ✅ MATCH |
| 2 | `type` filter on list_skills/search_memories | `mcp_server.py:111, 166`; `handlers.py:68, 159`; `test_mcp_type_filter_live.py:41-146` | ✅ MATCH |
| 3 | `_api_key` kw-only on all tools/resources | `handlers.py` all handlers `*, _api_key` | ✅ MATCH |
| 4 | `require_api_key` enforced everywhere | `handlers.py:37,76,108,127,147,165,183,204,229,247,265,282` | ✅ MATCH |
| 5 | Bridge `_SYNC_ENGINE_CLASS` validation | `bridge.py:29,134-156` | ✅ MATCH |
| 6 | No MagicMock in dispatch | `bridge.py:144-156` TypeError guards; stub `contexter_core.py` deleted; `test_bridge_mock_rejection.py:33-49` | ✅ MATCH |
| 7 | Bytes path ≥102400 | `bridge.py:31 (102_400), 214-220, 238-245` | ✅ MATCH |
| 8 | Launcher wires real services, stdio | `run_mcp.py:38-46, 69`; `test_mcp_launcher_wiring.py:32-44` | ✅ MATCH |
| 9 | 6 real services (no mocks) | `run_mcp.py:21-28` imports; `services/*.py` | ✅ MATCH |
| 10 | Success shape `content[0].text` | FastMCP serialisation; asserted `test_mcp_type_filter_live.py:61,106` | ✅ MATCH |
| 11 | Error shape -32602 structured | FastMCP InvalidParams; `MCPAuthError(ValueError)` `auth.py:15-22` | ✅ MATCH (except DEV-1/2 paths) |
| 12 | get_system_health real payload | `handlers.py:187-193` → `analytics_service.py:47-62` | 🟡 PARTIAL (DEV-3) |
| 13 | get_agent_info real agent config | `handlers.py:151-155` → `agent_service.py:17-19` → `Agent.model_validate` | ❌ DEVIATION (DEV-1, HIGH) |
| 14 | list_skills filtered real list | `handlers.py:173-174` → `skill_service.py:21-23` → `Skill.model_validate` | ❌ DEVIATION (DEV-2, HIGH) |
| 15 | agent resource real record | `handlers.py:269-273` → same Agent path | ❌ DEVIATION (DEV-1, HIGH) |
| 16 | analytics overview real counters | `analytics_service.py:29-45` → engine telemetry | 🟡 DEVIATION (DEV-3, MEDIUM) |
| 17 | store_memory persists; search returns it | `handlers.py:41-63`; `memory_service.py:16-31`; `test_mcp_launcher_wiring.py:47-72` | ✅ MATCH |
| 18 | Frozen component files exist | run_mcp.py / mcp_server.py / handlers.py / auth.py / bridge.py / services/ | ✅ MATCH |

---

## 07 · Findings (all observations — every item listed)

| ID | Severity | Finding | Evidence |
|---|---|---|---|
| DEV-1 | HIGH | `get_agent_info` and `contexter://agent/{id}` fail on real engine data: pydantic `ValidationError`. Python `Agent` requires `provider` and `model` (models/agent.py:15-16, no validation_alias); the Rust `Agent` serialises `type`, `description`, `capabilities`, `status`, `config`, `version` — no `provider`/`model` (contexter-core/src/models/agent.rs:19-36, `rename_all="camelCase"`). `AgentService.get` → `Agent.model_validate(raw)` raises. Design target "Real agent config" NOT met; the resulting error is an unstructured internal error, not the frozen -32602 shape. | `models/agent.py:10-23`; `contexter-core/src/models/agent.rs:19-36`; `agent_service.py:17-19`; `handlers.py:151-155, 269-273` |
| DEV-2 | HIGH | `list_skills` fails on real engine data: pydantic `ValidationError`. Python `Skill` requires `type` (models/skill.py:16); the Rust `Skill` serialises `category` and has no `type`/`parameters`/`enabled` (contexter-core/src/models/skill.rs:18-34). Additionally the `type` filter is silently dropped at the engine boundary — Rust `SkillFilter` declares `name`/`category`/`limit`/`offset` only (skill.rs:61-73), so even a successful parse could never return a "filtered" list by `type`. Design target "Filtered real skill list" NOT met (both the filter application and the result parse fail). | `models/skill.py:10-21`; `contexter-core/src/models/skill.rs:18-34, 61-73`; `skill_service.py:21-23`; `bridge.py:295-301`; `handlers.py:169-174` |
| DEV-3 | MEDIUM | Analytics overview and `get_system_health` counters are structurally always zero on real data. Engine keys are camelCase and do not match Python lookups: `cache_telemetry()` returns `gets/hits/misses/stores/invalidations/totalOps/entriesByType` (cache/metrics.rs:8-20) — Python reads `total_sessions/total_memories/total_agents/total_skills/cache_entries` (analytics_service.py:39-45,61) → 0; `storage_size()` returns `perCf/walSize/total` (settings.rs:9-16) — Python reads `total_bytes` (analytics_service.py:43,60,87) → 0; `status()` returns `cacheTelemetry` sub-object with no `uptime_seconds`/`memory_usage_mb` (bridge.rs:510-530) → 0. Only `status:"ok"` is real. "Real health payload" is PARTIAL. | `analytics_service.py:16-20, 29-62`; `contexter-core/src/cache/metrics.rs:8-20`; `contexter-core/src/models/settings.rs:9-16`; `contexter-core/src/bridge.rs:510-530` |
| DEV-4 | MINOR | Bridge dispatch mechanism differs from the frozen design wording: design step 5/diagram states `asyncio.to_thread`; implementation uses `loop.run_in_executor(self._pool, fn, *args)` over a `ThreadPoolExecutor` (bridge.py:123, 163-164). Semantically equivalent (sync call offloaded to a worker thread); pre-existing code, not changed by this feature. | `bridge.py:123, 163-164`; preview L69, L110 |
| DEV-5 | MINOR | `store_memory` schema registers 3 optional parameters (`tokens`, `tokenizer`, `model`) beyond the frozen API table (`session_id`, `role`, `content`, `_api_key`). Schema ≡ handler (contract direction satisfied), but the frozen table is a subset; additive, backward-compatible params. | `mcp_server.py:86-106`; `handlers.py:20-31`; preview L161 |
| INFO-1 | INFO | `auth.py` "Unchanged" contract: `CONtexTER_API_KEY` typo renamed to `CONTEXTER_API_KEY` (auth.py:45, deps.py, mcp_server.py:68). Behavior preserved (`require_api_key`/`_api_key` pattern intact); rename aligns code with the design's documented env var name. Not a functional deviation. | `auth.py:45`; diff `mcp_tools/auth.py`, `api/deps.py`, `mcp_server.py:68` |
| OBS-1 | INFO | `analytics_service._safe_get` masks the DEV-3 key mismatches silently (returns defaults for non-dict/exception and missing keys) — the zero counters are invisible to callers; no logging of fallback. | `analytics_service.py:16-20` |

---

## 08 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | ✅ All 5 deviations + 2 observations documented above with file:line evidence; no silent deferral |
| Zero findings are being silently deferred to a future iteration | ✅ None deferred |

---

## 09 · Summary

> **Design Compliance Assessment**
> The architecture, launcher wiring, bridge hardening (`_SYNC_ENGINE_CLASS` validation, Mock-rejection, ≥102400-byte path), auth flow, resource URI templates, and 6 of 8 tool schemas match the frozen design exactly. The repair delivers on its core promises: no MagicMock in the live path, no `unexpected keyword argument 'type'` TypeError, and real engine round-trips for session/memory/launcher wiring. Two HIGH deviations remain: `get_agent_info` / `contexter://agent/{id}` and `list_skills` cannot return real data because the Python Agent/Skill domain models are shaped for the REST/domain contract (`provider`/`model`, `type`/`parameters`/`enabled`) while the Rust engine serialises a different schema (`type`/`description`/`capabilities`/`status`/`config`, `category`/`filePath`) — the pydantic `ValidationError` that results also violates the frozen structured-error shape. A MEDIUM deviation leaves analytics counters at zero due to camelCase/snake_case key mismatch between engine telemetry and `AnalyticsService`. Two MINOR deviations (dispatch mechanism naming, extra store_memory params) and two informational items complete the list.

> **Findings**
> 1. HIGH DEV-1 — `get_agent_info` + agent resource: pydantic ValidationError vs real engine schema (Agent requires provider/model; engine emits type/description/capabilities/status/config/version). "Real agent config" target unmet.
> 2. HIGH DEV-2 — `list_skills`: pydantic ValidationError (Skill requires `type`; engine emits `category`); `type` filter silently dropped by Rust SkillFilter. "Filtered real skill list" target unmet.
> 3. MEDIUM DEV-3 — analytics overview / get_system_health counters always 0: engine camelCase telemetry (`totalOps`/`entriesByType`/`total`/`perCf`/`walSize`) vs Python snake_case lookups (`total_sessions`/`total_bytes`/`uptime_seconds`).
> 4. MINOR DEV-4 — dispatch uses `loop.run_in_executor(pool)` not `asyncio.to_thread` (equivalent semantics).
> 5. MINOR DEV-5 — `store_memory` registers extra optional `tokens`/`tokenizer`/`model` beyond frozen table.
> 6. INFO-1 — `CONtexTER_API_KEY` typo renamed to `CONTEXTER_API_KEY` (behavior preserved).
> 7. OBS-1 — `_safe_get` silently masks analytics key mismatches (zero counters without logging).

---

## 10 · Final Verdict

| Criterion | Result |
|---|---|
| Architecture matches design preview | ✅ PASS (1 minor mechanism deviation) |
| API contracts match design preview | ❌ FAIL (DEV-1, DEV-2 high; DEV-5 minor; 6/8 tools exact) |
| UI wireframe matches rendered output | N/A (no wireframe in contract) |
| Data flow matches design specification | ❌ FAIL (DEV-1/2/3 break steps 5-6 real-result contract; DEV-4 minor) |
| Carryover declaration clean | ✅ PASS |
| **Overall** | **CONDITIONAL PASS — 2 HIGH, 1 MEDIUM, 2 MINOR, 2 INFO findings; bug contracts required** |

---

_Generated by Design Compliance Validator · 2026-08-01 · Validation Contract: mcp-live-fix_
