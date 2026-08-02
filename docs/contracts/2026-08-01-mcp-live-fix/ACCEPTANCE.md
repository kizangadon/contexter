---
artifact: acceptance-criteria
version: "1.0"
created: 2026-08-01
status: draft
---

# Acceptance Criteria: MCP Server Live-Functionality Repair

## Story Context

The Contexter MCP server connects to OpenCode but every tool call fails at runtime (MagicMock await errors and schema/handler signature mismatches). This contract repairs the MCP layer so all 8 tools + 4 resources return real data from the real storage engine through a live stdio MCP client. Scope is limited to `contexter-server` MCP layer: tool/resource registration, handler signatures, service wiring, bridge integration. Auth model (`_api_key`, BUG-019) is preserved. Verification must occur through a live MCP client, not only unit tests.

## Happy Path

### AC-1: All 8 tools return real data over live stdio

**Given** a running MCP server (`run_mcp.py`, stdio transport) connected to a real engine

**When** a live MCP client calls each of the 8 tools (`get_system_health`, `list_recent_sessions`, `get_session`, `get_agent_info`, `list_skills`, `search_memories`, `store_memory`, `export_data`) with valid parameters

**Then** every call returns a successful result containing real data from the storage engine — no mock errors, no `unexpected keyword argument` errors, no tracebacks

### AC-2: All 4 resources resolve real data

**Given** a running MCP server connected to a real engine containing at least one session, memory, agent, and analytics record

**When** a live MCP client reads each resource URI (`contexter://session/{id}`, `contexter://memory/{id}`, `contexter://agent/{id}`, `contexter://analytics/overview`)

**Then** each read returns the real record payload for the requested ID / overview

### AC-3: `type` filter parameter works on `list_skills` and `search_memories`

**Given** a running MCP server

**When** a live MCP client calls `list_skills` with `{"type": "mcp"}` and `search_memories` with `{"query": "x", "type": "memory"}`

**Then** both calls succeed and return filtered real data (the `type` argument is accepted by the handlers)

### AC-4: Auth behavior preserved

**Given** the MCP server running without `CONTEXTER_API_KEY` set

**When** a live MCP client calls tools and reads resources without an `_api_key` argument

**Then** calls succeed (backward-compatible behavior)

**And** when `CONTEXTER_API_KEY` is set and the client passes an incorrect `_api_key`, calls are rejected per existing `require_api_key()` behavior

### AC-5: `store_memory` persists to the real engine

**Given** a running MCP server with a real session context

**When** a live MCP client calls `store_memory` with `session_id`, `role`, and `content`

**Then** the memory is persisted to the engine and a subsequent `search_memories` for that content returns it

## Edge Cases

### AC-6: Invalid parameters produce structured errors

**Given** a running MCP server

**When** a live MCP client calls a tool with invalid parameters (e.g., `get_session` with a nonexistent ID, `search_memories` without a query, malformed payloads)

**Then** the server returns a structured MCP error (protocol error or `isError` result) — no process crash, no raw Python traceback on stdout

### AC-7: Empty datasets behave gracefully

**Given** a running MCP server connected to an empty engine (no sessions, no memories, no agents, no skills)

**When** a live MCP client calls list-type tools and resources

**Then** calls return empty lists/overviews with success status (not errors)

## Error States

### AC-8: Engine failure is contained

**Given** an engine path that cannot be opened or an engine operation that fails

**When** a live MCP client invokes a tool

**Then** the server returns a structured error to the client and the server process remains alive for subsequent calls

## Non-Functional Criteria

### AC-9: No mocks in the live path

**Given** the complete live server call path (launcher → services → bridge → engine)

**When** the code is inspected and any tool is invoked live

**Then** no `unittest.mock` object (MagicMock, Mock, AsyncMock, patch) is instantiated or returned in the production path

### AC-10: Existing suite stays green; new tests cover repairs

**Given** the repository's test suite before the fix (≥579 passing, including 59 MCP tests)

**When** the fix is complete

**Then** the full suite passes with no regressions, and new tests that reproduce each repaired failure mode (they fail on the pre-fix code) are present and passing

### AC-11: No stdout pollution

**Given** the MCP server running over stdio transport

**When** tools are invoked

**Then** stdout carries only MCP JSON-RPC frames — no `print()`, no tracebacks, no debug output

## Notes

- Root cause is open by design; Workers must produce evidence before fixing (SPEC Task 1).
- Verification runs against a temporary engine path (per SPEC DAT-001) to avoid mutating user data.
- Scope excludes REST API, CLI, Rust core, web UI, and auth model changes.
