---
artifact: edge-cases
version: "1.0"
created: 2026-08-01
status: draft
---

# Edge Cases: MCP Server Live-Functionality Repair

## Feature Overview

Repair the Contexter MCP server so all 8 tools + 4 resources work through a live stdio MCP client, returning real engine data. Currently every live call fails with mock-await or schema-signature errors.

**Related Documents:**
- [SPEC.md](./SPEC.md)
- [ACCEPTANCE.md](./ACCEPTANCE.md)

## Edge Case Categories

### Input Validation

| Scenario | Expected Behavior | Priority | Notes |
|----------|------------------|----------|-------|
| `get_session` / `get_agent_info` with nonexistent ID | Structured MCP error (`isError` or protocol error), no crash | P1 | Not found must not kill the process |
| `search_memories` without `query` | Structured validation error | P1 | `query` is required per schema |
| Tool called with extra unknown params | Tolerated or structured error — never `TypeError` leaking as traceback | P1 | Schema drift class of bug |
| Tool called with the registered `type` filter param | Accepted and applied (skills/memories) | P1 | Current failure: `unexpected keyword argument 'type'` |
| `list_recent_sessions` with `limit` beyond data | Returns available sessions (min(limit, count)), success | P2 | Clamp, don't error |
| `store_memory` with empty content | Structured validation error; nothing persisted | P2 | |

### Boundary Conditions

| Scenario | Expected Behavior | Priority | Notes |
|----------|------------------|----------|-------|
| Empty engine (0 sessions/memories/agents/skills) | List tools return empty lists; analytics overview returns zeroed overview; all success | P1 | Graceful empty state |
| Engine with large memory content (≥102400 bytes) | `store_memory`/`search_memories` use the bytes path per bridge; success | P2 | Existing bridge behavior must not regress |
| `limit = 0` or negative | Treated as no-limit or clamped to sane default; success | P3 | |
| `format` param on `export_data` with unsupported value | Structured error or graceful fallback; no traceback | P2 | |

### Error States

| Scenario | Expected Behavior | Priority | Notes |
|----------|------------------|----------|-------|
| Engine path cannot be opened at launch | Server exits with clear stderr message (not stdout); no hang | P1 | stdout is MCP transport |
| Engine operation raises mid-call | Structured MCP error returned; process survives for next call | P1 | |
| `CONTEXTER_API_KEY` set + wrong/missing `_api_key` | `require_api_key()` rejects with auth error | P1 | Preserve BUG-019 behavior |
| `CONTEXTER_API_KEY` unset + no `_api_key` passed | Calls succeed (backward compat) | P1 | |
| Wrong JSON-RPC payload from client | Protocol-level error response; process alive | P2 | |
| FastMCP missing/import failure | `run_mcp.py` exits with clear stderr message (existing behavior) | P2 | |

### Concurrency

| Scenario | Expected Behavior | Priority | Notes |
|----------|------------------|----------|-------|
| Two concurrent tool calls (parallel stdio requests) | Both complete; no cross-talk; no interleaved stdout corruption | P2 | MCP stdio frames must stay intact |
| Concurrent `store_memory` to same session | Both persist; engine serializes via bridge thread pool | P3 | |

### Integration Failures

| Scenario | Expected Behavior | Priority | Notes |
|----------|------------------|----------|-------|
| Bridge/`contexter_core` import or method mismatch | Structured error; `_SYNC_ENGINE_CLASS` validation catches drift — never a MagicMock await | P1 | Core of REQ-001/AC-9 |
| FastMCP version behavior change (schema introspection) | Pin/align registered schemas explicitly to handler signatures | P1 | Root-cause candidate |
| Client disconnects mid-call | Server handles cleanly; no zombie process | P3 | |

## Error Messages

| Error State | User Message | Additional Action |
|-------------|--------------|-------------------|
| Nonexistent session/memory/agent | `Resource not found: <id>` (MCP `isError` result) | Client may retry with valid ID |
| Missing required param | `Missing required parameter: <name>` | Client re-issues with param |
| Auth failure | `Authentication required / invalid API key` | Client sets correct `_api_key` |
| Engine failure | `Engine error: <detail>` | Client may retry; server stays alive |

## Recovery Paths

### Engine failure mid-call

**User sees:** structured MCP error result from the tool.

**Recovery options:**
1. Retry the call (server process remains alive)
2. Restart the server if the engine path is broken

**Data preservation:** No partial writes; engine operations are synchronous through the bridge.

### Auth rejection

**User sees:** auth error result.

**Recovery options:**
1. Pass correct `_api_key`
2. Unset `CONTEXTER_API_KEY` to restore open mode

**Data preservation:** No data touched.

## Test Scenarios

### Must Test (P1)

- [ ] Live stdio client: all 8 tools return real data (no mock/signature errors)
- [ ] Live stdio client: all 4 resources resolve real records
- [ ] `list_skills`/`search_memories` accept `type` parameter
- [ ] Nonexistent IDs return structured errors, process survives
- [ ] Missing `query` on `search_memories` returns structured error
- [ ] Empty engine: list tools return empty success results
- [ ] Auth: key unset → success; wrong key → rejection
- [ ] No MagicMock/Mock/AsyncMock in live call path (code inspection + runtime)
- [ ] stdout carries only MCP frames (no prints/tracebacks)

### Should Test (P2)

- [ ] Large memory content bytes path (≥102400 bytes) works via MCP
- [ ] Concurrent tool calls complete without frame corruption
- [ ] Unsupported `export_data` format → structured error
- [ ] Engine path failure at launch → clean stderr exit

### Nice to Test (P3)

- [ ] `limit` edge values (0, negative, huge) behave sanely
- [ ] Client disconnect mid-call handled cleanly
