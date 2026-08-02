# Design Preview Comparison — MCP Server Live-Functionality Repair

> Annotated comparison of live behavior vs the **approved design preview** (`docs/contracts/2026-08-01-mcp-live-fix/plan/preview/preview-mcp-live-fix-approved.md`, frozen v1.0.0).
> Author: User-Testing Validator · 2026-08-01 · Evidence: live stdio wire captures (`docs/tests/results.json` + tee'd JSON-RPC logs, deleted after reporting).

**Verdict: 3 MISMATCHES (1 high, 2 medium) + 1 low-severity schema drift.** Architecture, data-flow steps, tool/resource inventory, and auth flows match; entity schemas and error shape deviate.

---

## 1. Architecture (Mermaid, `#architecture`)

| Frozen component | Live behavior | Status |
|---|---|---|
| `run_mcp.py` launcher — real services wired to engine, stdio, clean stderr | Confirmed — `build_services()` wires 6 real services; stdio transport; structlog to stderr only | ✅ MATCH |
| `create_mcp_server` — 8 tools + 4 resources; schemas match handlers | Confirmed — 8 tools, 4 resources; all `_api_key`; `type` on skills/memories | ✅ MATCH |
| `handlers.py` — real data; structured errors | ⚠️ Real data for 9/12 paths; agent/skill handlers surface pydantic errors; handler-level errors returned as success-with-`error`-key | ❌ PARTIAL |
| `auth.py` — unchanged, `require_api_key()` | Confirmed — full auth matrix passes (missing/wrong/correct; resource URI; open mode) | ✅ MATCH |
| `bridge.py` — `_SYNC_ENGINE_CLASS` validation, no MagicMock | Confirmed — real Rust engine dispatch, class-level mock refusal, bytes path intact (code) | ✅ MATCH |

## 2. Data Flow Sequence (`#dataflow`, steps 1–6)

| Step | Frozen behavior | Live behavior | Status |
|---|---|---|---|
| 1 | Client connects; initialize + tools/list return aligned schemas | Confirmed — 8 tools, schemas ⊆ handlers (F6 note) | ✅ MATCH |
| 2 | `tools/call` reaches handler without TypeError | Confirmed — no `unexpected keyword argument` on any call | ✅ MATCH |
| 3 | `_api_key` validated; open mode when unset | Confirmed — both modes verified | ✅ MATCH |
| 4 | Handler delegates to real service | Confirmed — 6 real services | ✅ MATCH |
| 5 | Service → bridge → engine via `asyncio.to_thread` | Confirmed | ✅ MATCH |
| 6 | Real result; errors → structured isError; process survives, stdout clean | ⚠️ Confirmed for protocol validation errors; **handler-level errors (not found/invalid id/bad format) return success results with an `error` key (isError=False)**; stdout clean, process survives | ❌ PARTIAL (F5) |

## 3. API Contract — Tools table (`#api`)

| Tool | Frozen params | Live schema | Status |
|---|---|---|---|
| `get_system_health` | `_api_key?` | `_api_key` | ✅ MATCH |
| `list_recent_sessions` | `project?`, `limit?`, `_api_key?` | project, limit, _api_key | ✅ MATCH |
| `get_session` | `id`(req), `_api_key?` | id(req), _api_key | ✅ MATCH |
| `get_agent_info` | `id`(req), `_api_key?` | id(req), _api_key — **fails at runtime (F1)** | ❌ RUNTIME FAIL |
| `list_skills` | `type?`, `_api_key?` | type, _api_key — **fails at runtime when skills exist (F2)** | ❌ RUNTIME FAIL |
| `search_memories` | `query`(req), `type?`, `project?`, `limit?`, `_api_key?` | query(req), type, project, limit, _api_key | ✅ MATCH |
| `store_memory` | `session_id`(req), `role`(req), `content`(req), `_api_key?` | + **tokens, tokenizer, model (extra optional)** | ⚠️ DRIFT (F6, low) |
| `export_data` | `format?`, `entities?`, `_api_key?` | format, entities, _api_key — **unsupported format accepted (F4)** | ⚠️ BEHAVIOR DRIFT |

## 4. API Contract — Resources table

| Frozen URI | Live template | Status |
|---|---|---|
| `contexter://session/{id}` | `contexter://session/{id}{?_api_key}` | ✅ MATCH |
| `contexter://memory/{id}` | `contexter://memory/{id}{?_api_key}` | ✅ MATCH |
| `contexter://agent/{id}` | `contexter://agent/{id}{?_api_key}` — **read fails on seeded engine (F1)** | ❌ RUNTIME FAIL |
| `contexter://analytics/overview` | `contexter://analytics/overview{?_api_key}` | ✅ MATCH |

## 5. Success / Error shapes

| Shape | Frozen | Live | Status |
|---|---|---|---|
| Success | `result.content[{type:"text", text:"<real data>"}]` | Confirmed — real data payloads (9/12 paths) | ✅ MATCH |
| Error | `error: {code:-32602, message:"Missing required parameter: query"}` or isError | Protocol validation errors → isError result with text (message wording differs: "Missing required argument"); **handler-level errors → success result containing `{"error": "…"}` (isError=False)** | ❌ MISMATCH (F5, medium) |

## 6. Auth flows (data flow step 3 + EC-006/007)

| Case | Frozen | Live | Status |
|---|---|---|---|
| Key set + wrong/missing `_api_key` (tool) | Reject | Reject — isError "API key required." / "Invalid API key." | ✅ MATCH |
| Key set + resource without `?_api_key` | Reject | Reject — McpError on read | ✅ MATCH |
| Key set + resource with `?_api_key` | Accept | Accept | ✅ MATCH |
| Key unset + no `_api_key` | Accept (open mode) | Accept | ✅ MATCH |

---

## Mismatch Summary

| # | Frozen contract element | Live deviation | Severity |
|---|---|---|---|
| M1 | `get_agent_info` + agent resource "Real agent config" | pydantic ValidationError — Rust `Agent` (type/description/capabilities/status/config) lacks Python-required `provider`/`model` | 🔴 HIGH |
| M2 | `list_skills` "Filtered real skill list" | pydantic ValidationError — Rust `Skill` uses `category`, Python requires `type` | 🔴 HIGH |
| M3 | Error shape (step 6 + frozen JSON) | Handler-level errors are success results with `error` key, not isError/protocol errors | 🟠 MEDIUM |
| M4 | `store_memory` frozen param list | 3 extra optional params (tokens/tokenizer/model) — additive | 🟡 LOW |

*Additional contract-table behavior deviations (not shape): unsupported `export_data` format accepted (EC-012, F4); empty `content` accepted and persisted (EC, F3).*

Root cause of M1/M2: Python domain models (`models/agent.py`, `models/skill.py`) and the Rust serde contracts (`contexter-core/src/models/agent.rs`, `skill.rs`) describe different entities; the bridge passes engine JSON straight into `Agent.model_validate` / `Skill.model_validate`. This is the same class of "schema drift" the contract set out to eliminate (REQ-003) — the repair fixed the MCP registration layer but the entity-schema drift remains in the service layer.
