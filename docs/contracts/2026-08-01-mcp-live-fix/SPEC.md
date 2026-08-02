---
title: MCP Server Live-Functionality Repair
version: 1.0
date_created: 2026-08-01
last_updated: 2026-08-01
owner: Orchestrator / Distinguished Backend Engineer
tags: [mcp, contexter-server, bug-fix, backend, fastmcp]
---

# MCP Server Live-Functionality Repair

## 1. Purpose & Scope

The Contexter MCP server (`contexter-server/run_mcp.py` + `contexter_server/mcp_server.py`) is registered in OpenCode's MCP configuration and connects successfully, but **every tool call fails at runtime** with one of two error classes:

- `An asyncio.Future, a coroutine or an awaitable is required` / `object MagicMock can't be used in 'await' expression`
- `handle_list_skills() got an unexpected keyword argument 'type'` (and the same for `search_memories`)

The purpose of this specification is to repair the MCP server so that all 8 tools and 4 resources return **real data from the real storage engine** when invoked through a live MCP client (stdio transport), with zero mock behavior, no schema/handler drift, and no regression of the existing test suite.

**In scope:** the MCP layer of `contexter-server` — tool/resource registration, handler signatures, service wiring, and any root cause in the bridge or FastMCP integration that produces mock objects or signature mismatches.

**Out of scope:** REST API, CLI, Rust core (`contexter-core`), web UI, authentication model changes, new features.

## 2. Definitions

| Term | Definition |
|---|---|
| MCP | Model Context Protocol — client/server protocol for agent tool exposure |
| FastMCP | Python MCP server framework used by `mcp_server.py` |
| stdio transport | MCP server run as a subprocess speaking JSON-RPC over stdin/stdout |
| Tool | An MCP-callable function exposed by the server (8 total) |
| Resource | An MCP URI-addressable data source (4 total) |
| MagicMock | A unittest.mock double that, when awaited or called in production, indicates test doubles leaking into the live path |
| Schema drift | Mismatch between the tool input schema registered by FastMCP and the actual handler function signature |
| Bridge | `bridge.py` — `StorageEngine` wrapper around the Rust `Engine` via `asyncio.to_thread` |

## 3. Requirements, Constraints & Guidelines

- **REQ-001**: All 8 MCP tools SHALL return real data from the real storage engine when invoked via a live MCP client. No tool SHALL return a mock, stub, or placeholder object.
- **REQ-002**: All 4 MCP resources SHALL resolve real data via their URIs (`contexter://session/{id}`, `contexter://memory/{id}`, `contexter://agent/{id}`, `contexter://analytics/overview`) when invoked via a live MCP client.
- **REQ-003**: The registered input schema of every tool SHALL match its handler function signature exactly. No tool call SHALL fail with `got an unexpected keyword argument`.
- **REQ-004**: The `_api_key` authentication pattern (BUG-019/028/029) SHALL be preserved: optional key, `require_api_key()` enforcement, backward-compatible when `CONTEXTER_API_KEY` is unset.
- **REQ-005**: The live server (`run_mcp.py`, stdio) SHALL start cleanly and SHALL NOT emit tracebacks to stdout (stdout is the MCP transport).
- **REQ-006**: The existing test suite (≥579 tests, including 59 MCP tests) SHALL remain green; new tests SHALL be added for every repaired failure mode.
- **REQ-007**: Error conditions (invalid params, missing IDs, engine failure) SHALL be returned as structured MCP tool errors, not as process crashes or Python tracebacks.
- **CON-001**: DDD applies — the MCP layer remains a thin adapter over the domain services (`MemoryService`, `SessionService`, `AgentService`, `SkillService`, `AnalyticsService`, `ExportService`). Business logic SHALL NOT move into handlers.
- **CON-002**: The fix SHALL be implemented via TDD — reproducing failing tests first, then the fix that turns them green.
- **CON-003**: Observability — handlers SHALL emit meaningful logs on entry, success, and failure (including tool name and outcome) without leaking sensitive data.
- **GUD-001**: Prefer the boring, obvious fix. Do not redesign the MCP layer; repair it.

## 4. Interfaces & Data Contracts

### 4.1 MCP Tools (8)

| Tool | Handler | Parameters (schema) | Notes |
|---|---|---|---|
| `get_system_health` | `handle_get_system_health` | `_api_key` (optional) | Returns engine health + resource usage |
| `list_recent_sessions` | `handle_list_recent_sessions` | `project` (optional), `limit` (optional), `_api_key` | Returns recent sessions |
| `get_session` | `handle_get_session` | `id` (required), `_api_key` | Returns session details |
| `get_agent_info` | `handle_get_agent_info` | `id` (required), `_api_key` | Returns agent config |
| `list_skills` | `handle_list_skills` | `type` (optional), `_api_key` | Returns skills; `type` filter must be accepted |
| `search_memories` | `handle_search_memories` | `query` (required), `type` (optional), `project` (optional), `limit` (optional), `_api_key` | Returns memories |
| `store_memory` | `handle_store_memory` | `session_id` (required), `role` (required), `content` (required), optional metadata, `_api_key` | Stores a memory |
| `export_data` | `handle_export_data` | `format` (optional), `entities` (optional), `_api_key` | Exports data |

### 4.2 MCP Resources (4)

| URI | Handler | Auth |
|---|---|---|
| `contexter://session/{id}` | `handle_session_resource` | `require_api_key()` |
| `contexter://memory/{id}` | `handle_memory_resource` | `require_api_key()` |
| `contexter://agent/{id}` | `handle_agent_resource` | `require_api_key()` |
| `contexter://analytics/overview` | `handle_analytics_overview_resource` | `require_api_key()` |

### 4.3 Success/Error Shape

Successful tool call: MCP JSON-RPC response with `result` containing real data payload.
Failed tool call: MCP JSON-RPC response with `error` — structured `isError` result or protocol error object; never a raw traceback on stdout.

## 5. Acceptance Criteria

- **AC-001**: Live MCP client calls all 8 tools; every call returns real data from the engine (no mock errors, no signature errors).
- **AC-002**: Live MCP client reads all 4 resources; every URI resolves real data.
- **AC-003**: `list_skills` and `search_memories` accept the `type` parameter without error.
- **AC-004**: `_api_key` auth works: calls succeed when key unset; calls with wrong key are rejected per existing behavior.
- **AC-005**: Invalid parameters produce structured MCP errors, not crashes or stdout tracebacks.
- **AC-006**: Full existing test suite passes; new tests cover each repaired failure mode (they fail on the unfixed code).
- **AC-007**: No `MagicMock` or other unittest.mock object appears anywhere in the live server call path.

## 6. Test Automation Strategy

- **Test Levels**: Unit (handlers with real services), Integration (FastMCP server with real services), End-to-End (live stdio subprocess + MCP client invocation).
- **Frameworks**: pytest (existing), FastMCP client transport for live protocol tests.
- **Test Data Management**: temp engine directories per test (existing `conftest.py` pattern); no shared state.
- **Coverage**: new handler/schema tests cover every tool and resource; suite coverage not reduced.
- **CI**: full `pytest` run must pass; `cargo test` untouched unless root cause crosses the bridge.

## 7. Rationale & Context

The MCP server was hardened for auth (BUG-019/028/029/030) but never validated through a live MCP client; unit tests exercise handlers directly, which is why the suite was green while live calls failed. The `MagicMock` errors indicate something in the registration/call path awaits a mock — either FastMCP registering a wrapper, a test double leaking into the production wiring, or the bridge returning mocks. Root cause is open; the Worker investigation (SPEC Task 1) must produce evidence before any fix.

## 8. Dependencies & External Integrations

### External Systems
- **EXT-001**: MCP client (OpenCode) — stdio transport subprocess launch of `run_mcp.py`.

### Technology Platform Dependencies
- **PLT-001**: Python 3 + FastMCP — version behavior of schema introspection and tool registration SHALL be verified, and the working version pinned if drift is version-related.
- **PLT-002**: `contexter_core` Rust `Engine` via `bridge.py` — sync bridge dispatch; `_SYNC_ENGINE_CLASS` validation (not MagicMock instances) per existing memory.

### Data Dependencies
- **DAT-001**: Engine path from `CONTEXTER_PATH` or `~/.contexter` — live verification SHALL run against a temp engine to avoid mutating user data.

## 9. Examples & Edge Cases

```python
# Live client call that currently fails (schema drift)
client.call_tool("list_skills", {"type": "mcp"})
# → Expected: list of skills; Actual today: TypeError: unexpected keyword argument 'type'

# Live client call that currently fails (mock)
client.call_tool("get_system_health", {})
# → Expected: health payload; Actual today: "object MagicMock can't be used in 'await' expression"
```

## 10. Validation Criteria

- Every REQ-001..007 has implementation code and a passing test (SPEC Compliance Validator).
- Live E2E verification passes all 8 tools + 4 resources over stdio (User-Testing Validator).
- Security review confirms auth model intact, no secrets logged.
- Performance review confirms no pathological latency introduced (engine calls are bounded; no N+1 across tools).

## 11. Related Specifications / Further Reading

- `docs/contracts/2026-07-25-bug-019-mcp-auth/`
- `docs/contracts/2026-07-26-bug-028-mcp-auth-constant-time/`
