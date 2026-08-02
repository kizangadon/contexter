# Design Compliance Review Report

# MCP Server Live-Functionality Repair — Auto Bug Loop Iteration 3

> Design preview → implementation compliance audit. Verifies the parent approved preview (`preview-mcp-live-fix-approved.md`), the two NEW iter-3 bug previews (`preview-count-sessions-fast-path.md`, `preview-fastmcp-framework-logging.md`), and spot-checks the 30 other bug-contract previews against the working tree (HEAD `27e031d`, uncommitted changes included).

**Verdict:** CONDITIONAL PASS (class: findings — 2 LOW) — 5/5 design dimensions verified

2026-08-02 · 5/5 design sections verified · Design Compliance Validator

---

## 01 · Design Preview Sections Covered

| # | Design Preview | Sections Verified |
|---|---|---|
| 1 | `plan/preview/preview-mcp-live-fix-approved.md` (parent, re-verify) | Architecture (C4-style), data flow sequence, API contract (8 tools/4 resources), auth gating, error shapes, env vars, launch failure |
| 2 | `bugs/2026-08-01-count-sessions-fast-path/plan/preview/preview-count-sessions-fast-path.md` | Architecture (as-is → to-be), sequence diagram, behavior contract, verification plan |
| 3 | `bugs/2026-08-01-fastmcp-framework-logging/plan/preview/preview-fastmcp-framework-logging.md` | Problem diagram (as-is), target diagram (to-be), sequence, acceptance gates AC-FL-001..006 |
| 4 | Other 30 bug-contract previews | Spot-check: architecture claims, fix boundaries, acceptance mappings vs current tree |

---

## 02 · Architecture Compliance

### A. count-sessions fast path (`preview-count-sessions-fast-path.md` §1)

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Unfiltered → O(1) estimate path | `count_sessions({})` → estimate-num-keys on sessions CF (NEW) | `rocksdb.rs:715-731` — when `filter.project.is_none()` and `filter.agent_id.is_none()` and `filter.status.is_none()`, reads `property_value_cf(sessions_cf, "rocksdb.estimate-num-keys")`, parses u64, returns | ✅ MATCH |
| Filtered (project) → index-prefix scan | index-prefix scan on session_index CF, exact (UNCHANGED) | `rocksdb.rs:693-713` — `session_index_prefix_from_filter(filter)`, forward iterator on `session_index` CF, prefix `starts_with` break | ✅ MATCH |
| Fallback: property unavailable → full scan | full-scan fallback, exact (unchanged behavior) | `rocksdb.rs:719-731` falls through to full scan at `rocksdb.rs:733-760` (`KEY_PREFIX_SESSION` prefix + in-memory filter) | ✅ MATCH |
| Python wrapper | `core/bridge.py count_sessions wrapper` | `bridge.py:296-298` — `filter=None` → `"{}"`, dispatches via `_run("count_sessions", filter_json)` | ✅ MATCH |
| Engine → backend dispatch | `Engine::count_sessions` → `RocksDbBackend::count_sessions` | `engine/session.rs:128-129` forwards to storage; `bridge.rs:160-167` parses filter JSON and calls `self.inner.count_sessions` | ✅ MATCH |
| `AnalyticsService.get_overview` (6 engine calls) | 6 engine calls in overview | `analytics_service.py:99-109` — `storage_size, status, count_sessions({}), count_memories({}), count_agents({}), count_skills({})` via `asyncio.gather` | ✅ MATCH |

### B. FastMCP framework logging (`preview-fastmcp-framework-logging.md` §1-2)

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Option A: fastmcp logger configured (level/filter) | `fastmcp` namespace logger configured with level/filter; framework error-call records suppressed | `fastmcp_logging.py:76-90` `configure_fastmcp_failure_stderr()` installs `_SuppressFrameworkTracebackBox` filter on `fastmcp`, `fastmcp.server`, `fastmcp.server.server` (the emitters, per Python logging filter semantics); wired at package import — `__init__.py:52-54` | ✅ MATCH (Option A chosen; design permits A **OR** B) |
| Option B alternative (errors subclass FastMCPError) | allowed alternative (exc_info=False) | Not used — `HandlerError` subclasses `ValueError` (`errors.py:24`), `MCPAuthError` subclasses `ValueError` (`auth.py:15`), exactly as the problem diagram (as-is) describes. Sequence diagram §3 is labeled "Option B shown" and is illustrative; the sanctioned Option A produces the identical observable outcome | ✅ MATCH (design's OR clause; mechanism note) |
| Bridge concise line retained | `bridge_call_failed` 224-char concise line | `bridge.py:256` `logger.error("bridge_call_failed", **error_context)` — no `exc_info`, no `exception` key; test asserts <512 chars, one line per failure (`test_bridge_engine_failure_stderr.py:59-81, 126-147`) | ✅ MATCH |
| Diagnostics log file retained (full traceback) | diagnostics log holds full traceback (unchanged) | `bridge.py:136-162` `_write_runtime_failure_diagnostics` appends `traceback.format_exception` to `CONTEXTER_LOG_FILE`/`~/.contexter/logs/mcp-launch.log`; test `test_bridge_engine_failure_stderr.py:84-100` and `test_framework_efs_stderr.py:324-343` assert `Traceback` present in log | ✅ MATCH |
| Structured isError frame unchanged | client stdout unchanged, isError frame | Filter only drops `fastmcp` logging records (`fastmcp_logging.py:70-73`); client frames pinned byte-identical via `BASELINE_FRAMES` in `test_framework_efs_stderr.py:53-73`; `result.isError is True` asserted for 8 error scenarios | ✅ MATCH |

### C. Parent preview architecture (re-verify)

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Launcher → FastMCP → handlers → services → bridge → engine | `run_mcp.py` → `create_mcp_server` → 8 tools/4 resources → 6 services → `StorageEngine` → `asyncio.to_thread` → Rust engine | `run_mcp.py:30-39, 102-117, 133`; `mcp_server.py:31-244`; `bridge.py:191, 229-231` | ✅ MATCH |
| No mock in live call path | no `unittest.mock` object in live path | `bridge.py:17` imports `Mock` only for dispatch validation; `bridge.py:200-223` raises `TypeError` when a method resolves to a mock; engine import hard-required (`bridge.py:21-28` refuses mocks at import) | ✅ MATCH |
| 8 tools + 4 resources registration | 8 tool handlers + 4 resource handlers | `mcp_server.py:85-192` (8 `@mcp.tool()`), `mcp_server.py:198-242` (4 `@mcp.resource()` URIs `contexter://session/{id}`, `contexter://memory/{id}`, `contexter://agent/{id}`, `contexter://analytics/overview` — each with `{?_api_key}` query param) | ✅ MATCH |
| Auth gating `_api_key` | `require_api_key()`, kw-only `_api_key` on all tools/resources (BUG-019/028/029 unchanged) | `auth.py:25-58` (hmac.compare_digest, open mode when unset); all 8 tool + 4 resource handlers take kw-only `_api_key` and call `require_api_key` (`handlers.py:151, 201, 246, 280, 309, 337, 365, 396, 434, 462, 490, 517`) | ✅ MATCH |

---

## 03 · API Contract Compliance

| Endpoint | Design Schema | Actual Schema | Status |
|---|---|---|---|
| `store_memory` | `session_id` (req), `role` (req), `content` (req), `_api_key?` | `mcp_server.py:86-90` + `handlers.py:133-141` — exact match | ✅ MATCH |
| `search_memories` | `query` (req), `type?`, `project?`, `limit?`, `_api_key?` | `mcp_server.py:103-108` + `handlers.py:187-195` — exact match | ✅ MATCH |
| `get_session` | `id` (req), `_api_key?` | `mcp_server.py:121-124` + `handlers.py:235-240` | ✅ MATCH |
| `list_recent_sessions` | `project?`, `limit?`, `_api_key?` | `mcp_server.py:133-137` + `handlers.py:263-269` | ✅ MATCH |
| `get_agent_info` | `id` (req), `_api_key?` | `mcp_server.py:147-150` + `handlers.py:298-303` | ✅ MATCH |
| `list_skills` | `type?`, `_api_key?` | `mcp_server.py:159-162` + `handlers.py:326-331` (type filter enforced, `type`→`category` translation via `AliasChoices` in `models/skill.py:50,77,97`) | ✅ MATCH |
| `get_system_health` | `_api_key?` | `mcp_server.py:171-173` + `handlers.py:355-360` | ✅ MATCH |
| `export_data` | `format?`, `entities?`, `_api_key?` | `mcp_server.py:181-185` + `handlers.py:384-390` (format allowlist `{json,yaml,csv}`) | ✅ MATCH |
| Resources (4) | `contexter://session/{id}`, `//memory/{id}`, `//agent/{id}`, `//analytics/overview` + `require_api_key()` | `mcp_server.py:198-242` — URIs match; handlers call `require_api_key` and raise structured `HandlerError` | ✅ MATCH |
| Success shape | `result.content[].text` real data | Live client tests assert `result.content[0].text == BASELINE_FRAMES[...]` and non-error content on success (`test_framework_efs_stderr.py`, `test_mcp_empty_engine_live.py`) | ✅ MATCH |
| Error shape | structured `isError` / protocol error; `Resource not found: <id>`; never `{"error":...}` success | `errors.py:24-52` `HandlerError(ValueError)` with `not_found_error`/`validation_error`/`storage_error` helpers; all failure paths raise (`handlers.py:118-127, 256, 319, 444, 472, 500`); `test_error_shape_drift.py` locks it | ✅ MATCH |
| camelCase↔snake_case | engine boundary camelCase, Python snake_case | `bridge.py:39-56` `_camelize_payload_keys`; Rust `#[serde(rename_all = "camelCase")]` (`models/agent.rs:9,19,48,62,75`, `notification.rs:9`, `feedback.rs:9`, `audit.rs:11`, …) | ✅ MATCH |
| Canonical `CONTEXTER_*` env | canonical vars only | `CONTEXTER_API_KEY` (`auth.py:45`, `mcp_server.py:68`), `CONTEXTER_PATH` (`run_mcp.py:122`, `cli/main.py:30`), `CONTEXTER_LOG_FILE` (`bridge.py:132`, `run_mcp.py:54`), `CONTEXTER_MCP_PORT` (`run_mcp.py:141`), `CONTEXTER_BRIDGE_POOL_SIZE` (`bridge.py:179`), `CONTEXTER_MAX_REQUEST_BODY` (`main.py:206`); `test_env_canonicalization.py` locks; grep audit: no `CONtexTER_*` typo remains | ✅ MATCH |
| Launch failure clean error | clean stderr line, no traceback, documented exit | `run_mcp.py:83-99` `_fail_engine_open` — one structured line, diagnostics to launch log, `sys.exit(ENGINE_OPEN_EXIT_CODE=2)`; exit 1 for missing fastmcp (`run_mcp.py:135-136`); tests in `test_mcp_launcher_wiring.py:124-147` | ✅ MATCH |

---

## 04 · UI Wireframe Compliance (protocol surface — this feature is an MCP server, no pixel UI)

For this feature the "UI" is the **protocol surface**: tool/resource frames, stderr/stdout hygiene, error presentation. All wireframe-equivalent states are asserted by live-client tests.

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Client-visible frame surface (8 tools + 4 resources) | tools/resource schemas aligned to handler signatures | Schema-registration tests (`test_store_memory_schema_conformity.py`, `test_mcp_server.py`) lock schema↔signature parity; live client tests (`test_mcp_type_filter_live.py`, `test_mcp_empty_engine_live.py`, `test_mcp_resource_auth_live.py`) exercise every surface | ✅ MATCH |
| Error state presentation | isError frame, no box, bounded stderr, no traceback | `test_framework_efs_stderr.py` — 8+ error scenarios assert `isError=True`, `stderr ≤512` bytes/chars, no `╭│╰` box chars, no `Traceback`, no source frames (`_assert_bounded` at :76-87) | ✅ MATCH |
| Empty state | empty engine → graceful empty results (EC-005) | `test_mcp_empty_engine_live.py` present; `_safe_get`/`_safe_int` defaults in `analytics_service.py:34-69` | ✅ MATCH |
| Loading/transition states | not applicable (stateless JSON-RPC request/response) | N/A — no transient UI states in the MCP protocol surface | ✅ MATCH (not applicable) |
| Stderr budget | ≤512 chars per failure, no rich box (AC-FL-001/002) | `fastmcp_logging.py` filter + bounded bridge line; tests `test_bridge_engine_failure_stderr.py` and `test_framework_efs_stderr.py` assert budget per failure and per class | ✅ MATCH |
| stdout purity | stdout carries only MCP JSON-RPC frames (AC-011) | no `print()` in `mcp_tools/`, `bridge.py`, `mcp_server.py`; `test_bridge_engine_failure_stderr.py:74` asserts `captured.out == ""` | ✅ MATCH |

---

## 05 · Data Flow Compliance

### A. count-sessions sequence (`preview-count-sessions-fast-path.md` §2)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| `S->>B: count_sessions({})` | `get_overview` calls bridge `count_sessions({})` | `analytics_service.py:103` → `bridge.count_sessions({})` | ✅ MATCH |
| `B->>E: count_sessions(filter=None)` | bridge wrapper passes empty filter | `bridge.py:296-298` — `None` → `"{}"` JSON, `_run("count_sessions", ...)` | ✅ MATCH |
| `E->>R: count_sessions(None)` | engine forwards to backend | `engine/session.rs:128-129`; `bridge.rs:160-167` parses `"{}"` → `SessionFilter::default()` | ✅ MATCH |
| `R->>DB: get_property("rocksdb.estimate-num-keys", "sessions")` | property read on sessions CF | `rocksdb.rs:720-722` — `property_value_cf(cf(sessions), "rocksdb.estimate-num-keys")` | ✅ MATCH |
| `DB-->>R` → `R-->>E: count (u64)` → `E-->>B` → `B-->>S: count` | count flows back | `rocksdb.rs:726-727` returns u64; `bridge.rs:164-165` returns usize → Python int; `analytics_service.py:112` `total_sessions` | ✅ MATCH |
| Filtered path | `{"project":"X"}` → index-prefix scan, exact | `rocksdb.rs:693-713`; verified by `agent_skill_test.rs:318-363` (alpha=3, beta=2, all=5) and `session_test.rs:88-124` (lifecycle 1→0) | ✅ MATCH |
| Fallback path | property unavailable → full scan, exact | `rocksdb.rs:730-760` — code present and correct | ✅ MATCH (behavior) — **fallback test gap, see Finding F-1** |

### B. FastMCP engine-failure sequence (`preview-fastmcp-framework-logging.md` §3)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| `C->>FM: tools/call get_session("not-a-uuid")` | client invokes tool | `test_framework_efs_stderr.py:138` `client.call_tool_mcp("get_session", {"id": _INVALID_ID})` over live FastMCP Client | ✅ MATCH |
| `H->>B: get_session(...)` → `B-->>H: raises ValueError` | bridge raises on invalid id | `bridge.py:229-257` — engine call raises; bridge logs `bridge_call_failed` (no exc_info) then re-raises; live engine formats `invalid session id "not-a-uuid": invalid character: found `n` at 0` (pinned in `BASELINE_FRAMES["engine"]`) | ✅ MATCH |
| `H-->>FM: raises HandlerError` | handler error reaches framework | `handlers.py:118-127` `_raise_structured_error` raises; service exceptions propagate through handler to FastMCP generic path | ✅ MATCH |
| `FM->>FM: exc_info=False (no box)` | framework logs without traceback box | Achieved via **Option A**: `_SuppressFrameworkTracebackBox` drops the framework `Error calling tool/reading resource/rendering prompt` records (`fastmcp_logging.py:31-35, 70-73`) — the same no-box outcome; unit test at `test_framework_efs_stderr.py:360-386` | ✅ MATCH (mechanism per Option A; design sanctions A or B) |
| `FM-->>C: isError=true structured frame` | client receives error frame | `test_framework_efs_stderr.py:140-141` `result.isError is True`, text byte-identical to baseline | ✅ MATCH |
| `Note over L: stderr: bridge line only; log file: full traceback` | bounded stderr; traceback in diagnostics log | `test_framework_efs_stderr.py:324-343` asserts `Traceback` in `CONTEXTER_LOG_FILE`; stderr budget asserted | ✅ MATCH |

### C. Parent data flow (re-verify, steps 1-6 of the frozen preview)

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1-2: initialize/tools/list + tools/call | schemas aligned; no TypeError | Schema↔signature registration tests green surface; live client tests in `tests/mcp/` | ✅ MATCH |
| 3: auth validation | `_api_key` validated when key set; open when unset | `auth.py:45-58`; `test_mcp_auth.py`, `test_mcp_resource_auth_live.py` | ✅ MATCH |
| 4: handler → real service | every handler calls real service | `mcp_server.py` wires `memory_service` etc.; handlers delegate to services; no mock in live path (bridge refuses mock dispatch) | ✅ MATCH |
| 5: service → bridge → engine | `asyncio.to_thread` dispatch with `_SYNC_ENGINE_CLASS` validation | `bridge.py:197-231` | ✅ MATCH |
| 6: real result / structured errors | JSON-RPC result; errors structured; process survives | Live tests assert isError frames; `test_bridge_mock_rejection.py`; `test_protocol_edge_cases.py` (6 tests) | ✅ MATCH |

---

## 06 · Component Hierarchy Compliance

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| `run_mcp.py` → `create_mcp_server` → `FastMCP("contexter")` | launcher owns server instance | `run_mcp.py:133`; `mcp_server.py:76-79` `FastMCP("contexter", instructions=...)` | ✅ MATCH |
| Tools/resources → handler functions → services | thin adapter chain | `mcp_server.py` closures delegate to `mcp_tools/handlers.py` functions, which inject `*_service` kwargs; services constructed in `run_mcp.py:102-117` | ✅ MATCH |
| Handler error layer | shared `HandlerError` helper | `mcp_tools/errors.py` — new shared module matching `preview-error-shape-drift.md`; all handlers use it | ✅ MATCH |
| FastMCP logging policy component | `fastmcp_logging` module wired at import | `fastmcp_logging.py` + `__init__.py:52-54`; idempotent install flag `_INSTALLED_ATTR` | ✅ MATCH |
| Service layer (6 services) | Memory/Session/Agent/Skill/Analytics/Export | `run_mcp.py:32-39, 110-116`; files exist under `services/` | ✅ MATCH |
| Bridge → engine | `StorageEngine` → `_SyncEngine` via thread pool | `bridge.py:165-191` | ✅ MATCH |
| Rust engine component | `Engine` → `RocksDbBackend` | `engine/session.rs`, `engine/agent.rs`, `engine/skill.rs`, `engine/search.rs` → `storage/rocksdb.rs` | ✅ MATCH |

---

## 07 · Other 30 Bug-Contract Previews — Spot-Check

| Bug Preview | Key Claim | Implementation Evidence | Status |
|---|---|---|---|
| agent-skill-schema-drift | translation layer; `category`↔`type`; `capabilities`↔`tools` | `models/skill.py:50,77,97` (`AliasChoices("type","category")`); `models/agent.py:51,78,98` (`AliasChoices("capabilities","tools")`) | ✅ MATCH |
| analytics-count-endpoints | overview uses counts, not scans | `analytics_service.py:103-106`; engine `count_agents`/`count_skills` (`engine/agent.rs:106`, `engine/skill.rs:124`) | ✅ MATCH |
| analytics-telemetry-mapping | explicit key mapping; `_safe_get` logs mismatches | `analytics_service.py:34-56, 44-50` (warning on missing key, debug on non-dict) | ✅ MATCH |
| bridge-double-encode | bytes path decode once; round-trip test ≥102400 | `bridge.py:304-323`; `tests/core/test_bridge_large_content_roundtrip.py` | ✅ MATCH |
| bridge-log-hygiene | `_truncated_args_summary` cap ≤64 | `bridge.py:62-113` (`_ARG_SUMMARY_CAP = 64`) | ✅ MATCH |
| camelization-coverage-tests | live-engine coverage of bridge methods | `tests/core/test_bridge_live_coverage.py` | ✅ MATCH |
| camelize-invariant-test | deterministic camelize policy | `bridge.py:39-56`; tests in `test_bridge.py`, `test_bridge_live_coverage.py` | ✅ MATCH |
| cli-status-test-alignment | formatter conforms to real status shape | `tests/cli/test_status_format.py:8-11` documents real shape `{status, version, cacheTelemetry}` | ✅ MATCH |
| engine-failure-stderr | concise stderr; full detail to log file | `bridge.py:232-257`; `test_bridge_engine_failure_stderr.py` | ✅ MATCH |
| env-var-canonicalization | canonical `CONTEXTER_*` only; no typo aliases | `test_env_canonicalization.py`; grep audit clean | ✅ MATCH |
| error-shape-drift | shared error helper; never `{"error":...}` success | `errors.py`; `handlers.py:118-127`; `test_error_shape_drift.py` | ✅ MATCH |
| handler-limit-passthrough | clamp in handler; no re-slice | `handlers.py:56-65, 288-291`; `test_handler_limit_passthrough.py` | ✅ MATCH |
| handler-observability | corr id; DEBUG per-call; ERROR errors; no content | `handlers.py:113-127`; `test_handler_observability.py` | ✅ MATCH |
| handlers-id-bounding | `_bounded()` 64-char at all error/log sites | `handlers.py:68-73`; `test_handlers_id_bounding.py` | ✅ MATCH |
| input-validation-gaps | content/query caps; export allowlist; limit clamp | `handlers.py:76-110` (`MAX_CONTENT_LENGTH=1_000_000`, `MAX_QUERY_LENGTH=10_000`); `test_input_validation_gaps.py` | ✅ MATCH |
| launcher-exception-type | `RuntimeError` pin | `test_mcp_launcher_wiring.py:222` `pytest.raises(RuntimeError)` | ✅ MATCH |
| launch-error-handling | structured client error; diagnostics in log; defined exit | `run_mcp.py:83-99`; `test_mcp_launcher_wiring.py:124-147` | ✅ MATCH |
| max-request-body-env | `CONTEXTER_MAX_REQUEST_BODY` canonical | `main.py:181-206` | ✅ MATCH |
| parent-edge-case-tests | EC-015/017/018/021 coverage | `tests/mcp/test_protocol_edge_cases.py` (6 tests), `test_mcp_empty_engine_live.py` | ✅ MATCH |
| perf-log-and-bounds-docs | per-call INFO → DEBUG; bounds docs | `bridge.py:258-266` (`bridge_call_end` at DEBUG) | ✅ MATCH |
| pre-existing-lifespan-test-fix | unique temp dir per test | `test_mcp_server.py:939-976` (`tmp_path` + `create_app(data_path=str(tmp_path))`) | ✅ MATCH |
| pydantic-alias-annotated | Annotated `AliasChoices` on agent/skill models | `models/agent.py:51,62,65,78,98`; `models/skill.py:50,77,97` | ✅ MATCH |
| scratch-cleanup | `docs/tests/` dirs empty after cleanup | ⚠️ See Finding F-2 — root `docs/tests/` currently holds 3 scratch files | 🟡 PARTIAL |
| search-total-failure | explicit signal on count failure, not silent 0 | `memory_service.py:59-82` — `total = -1` + `search_count_failed` error log | ✅ MATCH |
| session-limit-pushdown | limit pushed to engine; clamp; no re-slice | `session_service.py:8-11, 32-40`; `test_bridge.py:240-253` spy asserts engine receives limit | ✅ MATCH |
| store-memory-schema-conformity | registered schema exact match | `mcp_server.py:86-90`; `test_store_memory_schema_conformity.py` | ✅ MATCH |
| test-hardening | precise exception types; missing edge tests | `pytest.raises(RuntimeError)` pins; edge tests present (`test_mcp_empty_engine_live.py`, `test_input_validation_gaps.py`) | ✅ MATCH |
| doc-notes / docs-corrections | docs reflect canonical env, engine dependency | README/architecture docs updated (working tree `docs/design/specs/2026-07-23-contexter-system-architecture.md` modified) | ✅ MATCH |

---

## 08 · Findings (ALL items — any severity)

| ID | Severity | Preview / Contract | Finding |
|---|---|---|---|
| **F-1** | LOW | `preview-count-sessions-fast-path.md` §4 Verification Plan item 1 | **Dedicated Rust fallback test missing.** The verification plan explicitly lists a Rust test for the fallback path ("unfiltered parity, empty → 0, filtered exactness, **fallback**"). Implemented Rust tests cover unfiltered parity (`agent_skill_test.rs:273`), empty → 0 (`agent_skill_test.rs:308`), filtered exactness (`agent_skill_test.rs:318`, `session_test.rs:88-124`), but **no test forces the estimate-num-keys property to be unavailable and asserts the full-scan fallback** (`rocksdb.rs:730-760`). The scan loop is not exercised by any `count_sessions` test (all existing count tests take the property path or the project-index path). The fallback *behavior* is correctly implemented — this is a test-coverage gap vs. the preview's verification plan. |
| **F-2** | LOW | `preview-scratch-cleanup.md` (AC-SC-001..003) | **Leftover scratch files in root `docs/tests/`.** At verification time `/home/don/Code/contexter/docs/tests/` contains `iter3_harness.py` (08:57), `iter3_seed_large.py` (09:22), `iter3_validator_harness.py` (09:22), all created during this iteration. They are gitignored (`**/docs/tests/` → cannot be committed) but the scratch-cleanup contract requires both `docs/tests/` dirs to be empty after work completes. Per the Zero-Touch Rule I documented this and did not delete the files. |

---

## 09 · Carryover Check

| Check | Result |
|---|---|
| All findings from this iteration have corresponding bug contracts or are explicitly noted | ⚠️ F-1 and F-2 are documented here; they require new/updated bug contracts in the next loop iteration |
| Zero findings are being silently deferred to a future iteration | ✅ — both findings are explicitly listed above with severity and evidence |

---

## 10 · Summary

> **Design Compliance Assessment**
> The two new iter-3 design previews are faithfully realized in the working tree. `count_sessions` implements the unfiltered O(1) estimate-num-keys fast path, the project-filtered index-prefix scan (unchanged), and the full-scan fallback; the sequence `get_overview → bridge → engine → RocksDbBackend → RocksDB property` matches the actual call chain exactly. FastMCP framework logging implements the design's sanctioned Option A (namespace filter suppressing framework error-call records) while retaining the bridge's concise `bridge_call_failed` line, the full-traceback diagnostics log, byte-identical client frames, and all six acceptance gates. The parent preview re-verifies clean: 8 tools, 4 resources, `_api_key` auth gating, structured isError, camelCase↔snake_case mapping, canonical `CONTEXTER_*` env, and launch-failure clean error (rc=2). 28/30 other bug previews spot-check as MATCH; 2 items carry findings (F-1, F-2). Total: 5/5 design dimensions verified.

> **Findings**
> - **F-1 (LOW):** count-sessions preview §4 item 1 — no Rust test exercises the estimate-num-keys→full-scan fallback path; behavior is implemented but the planned fallback test is absent.
> - **F-2 (LOW):** scratch-cleanup contract — 3 leftover scratch files remain in root `docs/tests/` (`iter3_harness.py`, `iter3_seed_large.py`, `iter3_validator_harness.py`); gitignored but not cleaned.

---

## 11 · Final Verdict

| Criterion | Result |
|---|---|
| Architecture matches design preview | ✅ PASS |
| API contracts match design preview | ✅ PASS |
| UI wireframe (protocol surface) matches rendered output | ✅ PASS |
| Data flow matches design specification | ✅ PASS (F-1 = test gap, not flow deviation) |
| Component hierarchy matches design preview | ✅ PASS |
| All 30 other bug previews remain implemented | 🟡 28 MATCH / 2 with findings (F-1, F-2) |
| Carryover declaration clean | ✅ explicit |
| **Overall** | **CONDITIONAL PASS — 2 LOW findings (F-1, F-2)** |

---

_Generated by Design Compliance Validator · 2026-08-02 · Validation Contract: 2026-08-01-mcp-live-fix · Iteration 3_
