# Design Compliance Review Report

# MCP Server Live-Functionality Repair (mcp-live-fix) — Auto Bug Loop Iteration 1

> Design compliance audit: approved design preview (`plan/preview/preview-mcp-live-fix-approved.md`, v1.0.0, FROZEN) + 18 approved bug-contract previews (`bugs/2026-08-01-*/plan/preview/*.md`) vs implementation on `feature/mcp-live-fix` (HEAD 27e031d + working tree).

**Verdict:** PASS (class: ALL DEVIATIONS RESOLVED — 2 HIGH, 1 MEDIUM, 2 MINOR, 1 INFO from baseline re-verified; 3 new informational observations)

2026-08-01 · 19/19 design previews verified · Design Compliance Validator · Iteration 1

---

## 01 · Design Preview Sections Covered

| Section | Covered | Notes |
|---|---|---|
| Architecture (Mermaid, parent) | ✅ | Client → launcher → FastMCP → handlers → auth → services → bridge → Rust engine |
| API Contract — Tools (8) | ✅ | Schema vs handler signature vs frozen table |
| API Contract — Resources (4) | ✅ | URI + `{?_api_key}` RFC-6570 + handler + require_api_key |
| Success / Error JSON shapes | ✅ | `result.content[0].text`; -32602 / isError structured |
| 18 bug-contract previews (Mermaid each) | ✅ | Every bug approach diagram mapped to implementation + tests |
| UI Wireframe | N/A | Backend repair contract — no wireframe in preview; frozen component table is the structural contract |

---

## 02 · Architecture Compliance

Design (parent preview L43-71): `OC[OpenCode MCP Client] --stdio JSON-RPC--> RL[run_mcp.py] --> MCP[create_mcp_server FastMCP] --> H[handlers.py 8 tools + 4 resources] --> A[auth.py require_api_key]; H --> SVC[6 services]; SVC --> BR[bridge.py StorageEngine]; BR --asyncio.to_thread--> ENG[Rust Engine]; ENG --> DB[Engine store]`.

| Check | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| Launcher wiring | `run_mcp.py` — real services wired to engine; stdio; clean stderr | `run_mcp.py` L102-117 `build_services()` constructs all 6 real services on `StorageEngine`; L145 `mcp.run(transport="stdio")`; engine path from `CONTEXTER_PATH` (L122) | ✅ MATCH |
| FastMCP factory | `mcp_server.py` registers 8 tools + 4 resources; schemas match handler signatures | `mcp_server.py` L85-192: 8 `@mcp.tool()`; L198-242: 4 `@mcp.resource()`; every wrapper forwards exactly the handler kwargs (incl. `type=`, `_api_key=`); ImportError guard returns None (L59-63) | ✅ MATCH |
| Handler layer | `handlers.py` — real data; structured errors | `handlers.py` — 8 tool handlers (L131-415) + 4 resource handlers (L421-526); every handler calls `require_api_key()`; `HandlerError` raise-only contract; "not connected to storage" guard | ✅ MATCH |
| Auth layer | `auth.py` — `require_api_key()`, `_api_key` kw-only, constant-time compare | `auth.py` — `CONTEXTER_API_KEY` (canonical, INFO-1 resolved); `hmac.compare_digest`; `MCPAuthError(ValueError)`; all handlers/resources call it | ✅ MATCH |
| Services layer | 6 services on the bridge; domain layer translation boundaries | `MemoryService`, `SessionService`, `AgentService`, `SkillService`, `AnalyticsService`, `ExportService` all exist; agent/skill/analytics/session services now contain the documented translation/anti-corruption layers | ✅ MATCH |
| Bridge | `_SYNC_ENGINE_CLASS` validation; no MagicMock in dispatch; bytes path ≥102400; bounded thread pool | `bridge.py` L29 `_SYNC_ENGINE_CLASS`; L149-171 class+instance Mock rejection; L31 `_LARGE_CONTENT_THRESHOLD=102_400`; L231-243 / L263-271 bytes path (single encode); L126-138 pool from `CONTEXTER_BRIDGE_POOL_SIZE` (default 8, invalid/≤0 → 8) | ✅ MATCH |
| Rust engine / store | `contexter_core` real extension → engine store | `test_engine_real.py` asserts compiled `.so` extension; real round-trips; engine persists to RocksDB store | ✅ MATCH |
| Dispatch mechanism | `asyncio.to_thread` (diagram L69, step 5) | `bridge.py` L179: `loop.run_in_executor(self._pool, fn, *args)` over a bounded `ThreadPoolExecutor` — **DEV-4 formally resolved via doc-notes**: the accepted decision ("Bounded pool preferred over bare asyncio.to_thread() — unbounded thread growth under load") is now recorded in `docs/design/specs/2026-07-23-contexter-system-architecture.md` §bridge, and `CONTEXTER_BRIDGE_POOL_SIZE` is documented in README env table | ✅ MATCH (design formally revised by approved doc-notes contract) |

**Architecture findings:** All boxes and arrows of the parent Mermaid diagram map to real code. The no-mock contract is enforced in three layers (stub deletion, import guard, `_SYNC_ENGINE_CLASS`+instance Mock rejection). The bounded pool is now an approved, documented design decision (DEV-4 closed via formal design revision, not silent deferral).

---

## 03 · API Contract Compliance

Frozen tools table (parent preview L153-162) vs registered FastMCP schemas (`mcp_server.py`) vs handler signatures (`handlers.py`):

| Tool | Design Parameters (frozen) | Registered Schema / Handler Signature | Status |
|---|---|---|---|
| `get_system_health` | `_api_key?` | `_api_key?: str|None` (mcp_server.py L171-178; handlers.py L353-360) | ✅ MATCH |
| `list_recent_sessions` | `project?`, `limit?`, `_api_key?` | `limit?`, `project?`, `_api_key?` (mcp_server.py L133-144; handlers.py L261-272) — clamp + pushdown, no re-slice | ✅ MATCH |
| `get_session` | `id` (req), `_api_key?` | `id: str`, `_api_key?` (mcp_server.py L121-130; handlers.py L233-240) | ✅ MATCH |
| `get_agent_info` | `id` (req), `_api_key?` | `id: str`, `_api_key?` — **DEV-1 resolved**: engine payloads validate directly against `Agent` (AliasChoices `capabilities`/`tools`, `createdAt`/`updatedAt`; provider/model resolved from `config` in `agent_service._from_engine`) | ✅ MATCH |
| `list_skills` | `type?`, `_api_key?` | `type: str|None`, `_api_key?` — **DEV-2 resolved**: `Skill` accepts `category`→`type` (AliasChoices), `version` u32→str, `filePath`→`file_path`; `skill_service._translate_filter` maps `type`→`category` AND re-applies the domain filter (silent drop eliminated) | ✅ MATCH |
| `search_memories` | `query` (req), `type?`, `project?`, `limit?`, `_api_key?` | `query: str`, `type?`, `project?`, `limit?`, `_api_key?` — exact; limit clamped to `[1, MAX_SEARCH_LIMIT]` | ✅ MATCH |
| `store_memory` | `session_id` (req), `role` (req), `content` (req), `_api_key?` | `session_id`, `role`, `content`, `_api_key?` — **DEV-5 resolved**: `tokens`/`tokenizer`/`model` removed; `test_store_memory_schema_conformity.py` asserts `EXPECTED_PARAMS={session_id, role, content, _api_key}` and bans legacy extras | ✅ MATCH |
| `export_data` | `format?`, `entities?`, `_api_key?` | `format: str|None`, `entities: list[str]|None`, `_api_key?` — format allowlist enforced | ✅ MATCH |

Resources (parent preview L166-171):

| URI | Design | Actual | Status |
|---|---|---|---|
| `contexter://session/{id}` | handler + require_api_key | `mcp_server.py` L198 `"contexter://session/{id}{?_api_key}"` → `handle_session_resource` calls `require_api_key` | ✅ MATCH |
| `contexter://memory/{id}` | handler + require_api_key | L210 + `{?_api_key}` → `handle_memory_resource` calls `require_api_key` | ✅ MATCH |
| `contexter://agent/{id}` | handler + require_api_key | L222 + `{?_api_key}` → `handle_agent_resource` calls `require_api_key` — **DEV-1 resolved**: live round-trip works | ✅ MATCH |
| `contexter://analytics/overview` | handler + require_api_key | L234 + `{?_api_key}` → `handle_analytics_overview_resource` calls `require_api_key` | ✅ MATCH |

Success shape: dict returns serialise to `result.content[0].text` (FastMCP 3.4, pinned); verified in live tests. ✅ MATCH.

Error shape: `HandlerError(ValueError)` subclasses — validation/not_found/storage — raised (never `{"error":...}` success payloads); `not_found_error` → `Resource not found: <id>` convention; `MCPAuthError(ValueError)` → clean JSON-RPC error; FastMCP emits `isError` frames. **DEV-1/DEV-2 pydantic ValidationError paths eliminated** (empirically verified: `Agent.model_validate(engine_payload)` and `Skill.model_validate(engine_payload)` succeed). ✅ MATCH.

**API findings:** 8/8 tools and 4/4 resources now meet the frozen contract. All five baseline API deviations (DEV-1, DEV-2, DEV-5, plus the DEV-3 error-path consequence) are resolved with translation-boundary tests.

---

## 04 · UI Wireframe Compliance

N/A — backend MCP repair contract; no UI wireframe in the design preview. Structural contract (frozen components table, parent preview L75-82) re-verified:

| Component | File | Contract | Status |
|---|---|---|---|
| Launcher | `run_mcp.py` | Real services wired; stdio; clean stderr; defined failure behavior | ✅ MATCH |
| MCP server | `mcp_server.py` | 8 tools + 4 resources; schemas ≡ handler signatures | ✅ MATCH |
| Handlers | `mcp_tools/handlers.py` | Real data; accept all schema params; structured errors; observability | ✅ MATCH |
| Auth | `mcp_tools/auth.py` | `require_api_key()`, `_api_key` kw-only, constant-time compare | ✅ MATCH |
| Bridge | `core/bridge.py` | `_SYNC_ENGINE_CLASS` validation; no mock dispatch; bytes ≥102400; bounded pool | ✅ MATCH |
| Services | `services/*.py` | Domain layer; translation boundaries documented | ✅ MATCH |

---

## 05 · Data Flow Compliance

Parent preview numbered steps (L95-121) vs actual runtime flow:

| Step | Design Spec | Actual Implementation | Status |
|---|---|---|---|
| 1. Client connects and lists tools | `initialize` + `tools/list` return schemas aligned to handler signatures | `run_mcp.py` L145 stdio; `mcp_server.py` 8 tools registered from handler kwargs; `test_mcp_type_filter_live.py` + `test_store_memory_schema_conformity.py` assert schemas | ✅ MATCH |
| 2. Client invokes a tool | Schema-validated args reach handler; no `unexpected keyword argument` | Wrappers forward `type=type` (mcp_server.py L102-118, L159-168); live FastMCP client calls succeed | ✅ MATCH |
| 3. Handler validates auth | `_api_key` via `require_api_key()`; constant-time compare | Every handler L1 call; `hmac.compare_digest`; `test_mcp_auth.py` + `test_mcp_resource_auth_live.py` (correct key ok / missing / wrong rejected; URI templates carry `{?_api_key}`) | ✅ MATCH |
| 4. Handler delegates to real service | Every handler calls its real service — never a mock | `run_mcp.py` builds 6 real services; `test_mcp_launcher_wiring.py` asserts every `service._engine` is a `StorageEngine`, never `Mock` | ✅ MATCH |
| 5. Service → Bridge → Engine | Bridge dispatches sync Rust call off-thread with `_SYNC_ENGINE_CLASS` validation | `bridge.py` L179 `loop.run_in_executor(self._pool, ...)`; mock rejection; real round-trips (launcher wiring, engine live, analytics live, agent/skill live) | ✅ MATCH |
| 6. Real result returned | Engine data marshals as JSON-RPC result; errors structured; stdout clean | Real agent/skill/session/memory/analytics payloads (live tests); `HandlerError` → structured `isError`; empty-engine path returns empty results with success (AC-7) | ✅ MATCH |

Launch-failure data flow (bug contract `launch-error-handling`, Mermaid `open_failure -->[client] structured error -->[server log] full diagnostics -->exit 2`):

| Step | Design | Actual | Status |
|---|---|---|---|
| Engine open fails (LOCK/corrupt/unwritable) | ONE clean structured stderr line; full diagnostics to launch log; exit code 2 | `run_mcp.py` L83-99 `_fail_engine_open`: single `contexter: engine_open_failed: ...` line to stderr; L58-80 `_write_launch_failure_log` appends timestamped record + full traceback to `~/.contexter/logs/mcp-launch.log` (override `CONTEXTER_LOG_FILE`); `sys.exit(ENGINE_OPEN_EXIT_CODE=2)`; exit 1 reserved for missing fastmcp | ✅ MATCH |
| Direct `build_services` callers | Keep receiving the raw exception | `test_mcp_launcher_wiring.py` L208-221 `test_build_services_still_raises_raw_on_engine_open_failure` (deliberate `pytest.raises(Exception)` with documented raw-contract rationale — not a hardening gap) | ✅ MATCH |

Session-limit pushdown flow (bug contract `session-limit-pushdown`, `handler-limit-passthrough`): handler `_clamp_session_list_limit` (handlers.py L54-63; None passes through, ≤0 → 0, >MAX → MAX) → `session_service.list` clamps and pushes limit to engine (session_service.py L42-48; engine performs slicing, no Python re-slice) → `test_handler_limit_passthrough.py` spy asserts exact call `(filter=None, limit=5)`; `test_session_service_live.py` asserts engine receives requested limit, returns exactly N, ordering identical to full page first N. ✅ MATCH.

---

## 06 · Bug-Contract Compliance Matrix (18 previews → implementation + tests)

| # | Bug Contract (preview Mermaid approach) | Implementation Evidence | Tests | Status |
|---|---|---|---|---|
| 1 | agent-skill-schema-drift (translation boundary) | `models/agent.py` AliasChoices + config-blob; `models/skill.py` category/type + version coercion; `agent_service._to_engine/_from_engine`; `skill_service._translate_filter` + re-apply | `test_agent_skill_engine_live.py`, `test_agent.py`, `test_skill.py`, `test_agent_service.py`, `test_skill_service.py` | ✅ MATCH |
| 2 | analytics-telemetry-mapping (explicit key mapping, `_safe_get` logs mismatch) | `analytics_service.py` L35-96 `_safe_get`/`_safe_int`/`_safe_len`/`_safe_cache_entries` log warnings on missing keys (never silent zero); reads real engine keys (`total_ops`, `entries_by_type`, `total`) | `test_analytics_service_live.py` seeds real store → asserts non-zero counters | ✅ MATCH |
| 3 | bridge-double-encode (decode once; byte-identical ≥102400) | `bridge.py` L231-232/L263-264 encode once, reuse for size check + payload; bytes path via `create_memory_bytes`/`update_memory_bytes` | `test_bridge_large_content_roundtrip.py` (≥102400 byte-identical) | ✅ MATCH |
| 4 | bridge-log-hygiene (args summary cap ≤64; no full content/secrets) | `bridge.py` L59-110 `_ARG_SUMMARY_CAP=64`, `_truncated_args_summary` never materialises full repr; used in `_run` L175/181/187 | `test_bridge.py`, `test_bridge_live_coverage.py` | ✅ MATCH |
| 5 | camelization-coverage-tests (all engine methods live harness) | `test_bridge_live_coverage.py` exercises 35/36 methods live (open implicit) with 0 exceptions + wire-shape assertions; `test_engine_real.py` asserts 36-method contract against compiled extension | tests/core/ | ✅ MATCH |
| 6 | cli-status-test-alignment (mock + formatter conform to real engine status shape) | `status_commands.py` reads only real fields (`status`, `version` via `_read_engine_version` L66-85, graceful `"unknown"`); formatters `_format_uptime`/`_format_bytes` | `test_status_format.py` (interpolation, graceful degradation without optional keys, version-missing → unknown, GC exception) | ✅ MATCH |
| 7 | doc-notes (README/ARCHITECTURE: canonical vars, engine dependency, accepted decisions) | README +70 lines: env var table (CONTEXTER_*), "Engine as hard dependency" section; `docs/design/specs/2026-07-23-contexter-system-architecture.md` §bridge: bounded-pool accepted decision, telemetry mapping | (docs-only; no test required by contract) | ✅ MATCH |
| 8 | env-var-canonicalization (CONTEXTER_* canonical, typo removed) | `bridge.py` L127 `CONTEXTER_BRIDGE_POOL_SIZE`; grep across `src/` + `run_mcp.py` → zero `CONtexTER_` offenders | `test_env_canonicalization.py` | ✅ MATCH |
| 9 | error-shape-drift (raise, never `{"error":...}`; `Resource not found: <id>`) | `errors.py` `HandlerError` kinds + `not_found_error`; all handlers raise | `test_error_shape_drift.py` | ✅ MATCH |
| 10 | handler-limit-passthrough (clamp in handler; no re-slice) | `_clamp_session_list_limit` (handlers.py L54-63) + service pushdown | `test_handler_limit_passthrough.py` (spy) | ✅ MATCH |
| 11 | handler-observability (call → auth → result + duration → error; no content/secrets; correlation id) | `_log_bind` correlation_id (L111-113); `call_received`/`auth_decision`/`engine_result`+duration/`handler_error` logs; `_raise_structured_error` logs only kind + duration, never message | `test_handler_observability.py` (caplog, ANSI-stripped) | ✅ MATCH |
| 12 | input-validation-gaps (non-empty content, size caps, format allowlist, bounded errors) | `_validate_content` (MAX_CONTENT_LENGTH=1_000_000), `_validate_query` (MAX_QUERY_LENGTH=10_000), `_validate_export_format` (json/yaml/csv), `_bounded` 64-char error messages | `test_input_validation_gaps.py` (EC-006/012/009, REQ-IV-004/005) | ✅ MATCH |
| 13 | launch-error-handling (clean structured error + full server log + exit 2) | `run_mcp.py` L42/45/58-99; `ENGINE_OPEN_EXIT_CODE=2` | `test_mcp_launcher_wiring.py` L144-221 (locked dir, unwritable dir, corrupt data, raw-contract) | ✅ MATCH |
| 14 | pre-existing-lifespan-test-fix (unique temp data dir per test; no RocksDB LOCK contention) | `test_mcp_server.py` L865-946: every lifespan test uses `tmp_path` (per-test unique dir) | `test_lifespan_shutdown_joins_thread`, `test_lifespan_double_shutdown_is_idempotent`, etc. | ✅ MATCH |
| 15 | scratch-cleanup (delete leftover scratch in both `docs/tests/`) | Top-level `docs/tests/` empty ✅; both dirs gitignored (`git check-ignore` confirms) | (filesystem; see OBS-DC-3) | 🟡 PARTIAL (OBS-DC-3) |
| 16 | session-limit-pushdown (clamp 0..max; engine slicing; most-recent-first) | `session_service.py` L11/42-48; engine receives clamped limit | `test_session_service_live.py` | ✅ MATCH |
| 17 | store-memory-schema-conformity (frozen schema + handler signature exact) | `handlers.py` L131-139 signature `(session_id, role, content, *, _api_key)`; mcp_server.py wrapper identical | `test_store_memory_schema_conformity.py` (EXPECTED_PARAMS, legacy banned) | ✅ MATCH |
| 18 | test-hardening (precise exception types; missing edge tests) | Broad `pytest.raises(Exception)` eliminated across suite except the single documented raw-contract test (L218); edge tests added (empty-engine, empty-content, limit edges, launch failure) | Full suite: **794 passed** | ✅ MATCH |

---

## 07 · Findings (all observations — every item listed)

### Baseline findings re-verified (status after Iteration 1 fixes)

| ID | Baseline Severity | Status | Resolution Evidence |
|---|---|---|---|
| DEV-1 | HIGH | ✅ RESOLVED | `models/agent.py` (AliasChoices, optional provider/model), `agent_service._from_engine` config promotion; empirical `Agent.model_validate(engine_payload)` succeeds; live round-trip test |
| DEV-2 | HIGH | ✅ RESOLVED | `models/skill.py` category→type/version/filePath; `skill_service._translate_filter` + domain re-filter; empirical validate succeeds; live round-trip test |
| DEV-3 | MEDIUM | ✅ RESOLVED | `analytics_service.py` reads real engine keys (`total_ops`, `entries_by_type`, `total`) and logs mismatches; live seeded test asserts non-zero counters |
| DEV-4 | MINOR | ✅ RESOLVED (formal design revision) | Bounded pool documented as accepted decision in architecture doc §bridge + README env table (approved `doc-notes` contract); `CONTEXTER_BRIDGE_POOL_SIZE` canonical |
| DEV-5 | MINOR | ✅ RESOLVED | Extra store_memory params removed; schema-conformity test enforces frozen table |
| INFO-1 | INFO | ✅ RESOLVED | `CONTEXTER_API_KEY` canonical everywhere; zero `CONtexTER_` offenders (grep) |
| OBS-1 | INFO | ✅ RESOLVED | `_safe_get` now logs explicit warnings on key mismatch instead of silent default |

### New observations (Iteration 1)

| ID | Severity | Finding | Evidence |
|---|---|---|---|
| OBS-DC-1 | INFO | `test_agent_skill_engine_live.py` etc. pass, but pydantic emits 5 `UnsupportedFieldAttributeWarning` (validation_alias inside `Field()` for `capabilities`/`tools` and `type`/`category`) during schema build. **Empirically verified the aliases DO work at runtime** (`AgentCreate(name='x', tools=['t1'])` → `capabilities=['t1']`; `SkillCreate(category='cmd')` → `type='cmd'`; engine payloads validate). Warning noise only, no behavioral defect — noted for potential code-quality cleanup (e.g., `Annotated` metadata). | `models/agent.py:43-46,73-76`, `models/skill.py:43,74`; pytest warnings summary |
| OBS-DC-2 | INFO | `analytics-telemetry-mapping` preview diagram and `doc-notes` §7.4 describe engine telemetry as camelCase (`totalOps`, `entriesByType`). The actual Rust `CacheTelemetry` (`contexter-core/src/cache/metrics.rs`, no `rename_all`) emits **snake_case** (`total_ops`, `entries_by_type`); only `StorageSize` (`settings.rs`, `rename_all="camelCase"`) is camelCase (`perCf`, `walSize`, `total`). Implementation correctly reads the real keys; the design doc's key-naming claim is inaccurate for cache telemetry (docs-only correction suggestion). | `metrics.rs:8-22`; `settings.rs`; `analytics_service.py:86-88,130,148,166` |
| OBS-DC-3 | INFO | `contexter-server/docs/tests/` is not empty at report time: `e2e_iter1.py`, `e2e_iter1_err.txt`, `e2e_iter1_out.txt`, `results_iter1.json` — all **actively written during this iteration** (timestamps 06:15:39-45) by the concurrently-running User-Testing/E2E validator. Top-level `docs/tests/` is empty. Not stale leftovers; deletion is pending the parallel validator's completion (per its own cleanup obligation). Scratch-cleanup contract AC-SC-001..003 final state must be re-checked at iteration close. | `ls -la contexter-server/docs/tests/` |
| OBS-DC-4 | INFO | `test_mcp_launcher_wiring.py:218` retains one `pytest.raises(Exception)` — **intentional**: asserts the raw-exception contract of `build_services` (only the launcher entry point converts to a clean error; direct callers keep the original exception), documented in the test docstring and mandated by the launch-error-handling contract. Not a hardening gap. | `test_mcp_launcher_wiring.py:208-221` |

---

## 08 · Carryover Check

| Check | Result |
|---|---|
| All baseline findings (DEV-1..5, INFO-1, OBS-1) have corresponding bug contracts and are resolved in this iteration | ✅ All 7 resolved with evidence above |
| Zero findings are being silently deferred to a future iteration | ✅ None deferred; all 4 new observations are documented with evidence and disposition |
| Test suite green | ✅ **794 passed, 0 failed** (11.63s, `contexter-server/tests/`) |

---

## 09 · Summary

> **Design Compliance Assessment**
> All 19 design previews (parent frozen preview + 18 approved bug-contract previews) map to implementation. Every baseline deviation is resolved: the Agent/Skill translation boundaries eliminate the pydantic ValidationError paths (DEV-1/DEV-2), the analytics anti-corruption layer reads real engine telemetry with explicit mismatch logging (DEV-3/OBS-1), the store_memory schema is byte-exact to the frozen table (DEV-5), env vars are canonical (INFO-1), and the bounded-pool dispatch mechanism was formally adopted as an accepted design decision through the approved doc-notes contract (DEV-4). All 8 tools and 4 resources meet the frozen API contract; all data-flow steps trace through live, tested code paths; launch-failure behavior is defined, documented, and tested (exit 2, single stderr line, server-side diagnostics log). 794 tests pass. Four informational observations remain, each with a clear disposition (docs key-naming inaccuracy, pydantic warning noise, in-flight scratch files from the parallel E2E validator, one intentional raw-contract test).

> **Findings**
> 1. OBS-DC-1 (INFO) — pydantic UnsupportedFieldAttributeWarning noise on AliasChoices in `Field()`; behavior empirically correct; candidate for Annotated-style cleanup.
> 2. OBS-DC-2 (INFO) — design doc claims camelCase cache telemetry; real engine emits snake_case; implementation reads real keys correctly; docs-only correction suggested.
> 3. OBS-DC-3 (INFO) — `contexter-server/docs/tests/` contains live files of the concurrently-running E2E validator; deletion pending that validator's completion; re-check at iteration close.
> 4. OBS-DC-4 (INFO) — single `pytest.raises(Exception)` is an intentional, documented raw-contract test; not a gap.

---

## 10 · Final Verdict

| Criterion | Result |
|---|---|
| Architecture matches design preview | ✅ PASS (bounded pool formally adopted via approved design revision) |
| API contracts match design preview | ✅ PASS (8/8 tools, 4/4 resources exact) |
| UI wireframe matches rendered output | N/A (no wireframe in contract) |
| Data flow matches design specification | ✅ PASS (all 6 steps + launch-failure + limit-pushdown flows traced) |
| Carryover declaration clean | ✅ PASS (all baseline findings resolved; 4 informational observations documented) |
| **Overall** | **PASS — 0 HIGH / 0 MEDIUM / 0 MINOR; 4 INFO observations (all non-blocking, evidence-backed)** |

---

_Generated by Design Compliance Validator · 2026-08-01 · Validation Contract: mcp-live-fix · Auto Bug Loop Iteration 1_
