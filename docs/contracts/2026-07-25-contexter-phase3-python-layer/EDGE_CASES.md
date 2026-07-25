---
title: "Phase 3 — Python API Layer: Edge Cases"
version: 1.0
date_created: 2026-07-25
tags: edge-cases, error-states, boundary-conditions
---

# Edge Cases — Phase 3: Python API Layer

This document catalogs every edge case, error state, boundary condition, and failure mode for the Phase 3 Python API Layer implementation.

---

## E-001: Rust Engine shared library not found

**Scenario:** The user ran `pip install` but `contexter-core` was not built via maturin.
**Error:** `ModuleNotFoundError: No module named 'contexter_core'`
**Expected behavior:** The bridge import SHALL raise `ImportError` with a clear message telling the user to run `maturin develop`.
**Verification:** Run `python -c "from contexter_core import Engine"` without having run `maturin develop` first.
**Recovery:** User runs `maturin develop --release -m contexter-core/pyproject.toml` from the project root.

---

## E-002: Rust Engine version mismatch

**Scenario:** The installed `contexter-core` wheel is a different version than the Python code expects.
**Expected behavior:** If the Rust engine lacks a method the bridge expects, a clear `AttributeError` propagates. The bridge SHALL NOT silently swallow missing methods.
**Verification:** API startup tests verify the bridge initialises without attribute errors.

---

## E-003: Bridge large content threshold — exactly 100KB

**Scenario:** Memory content is exactly 102,400 bytes (100KB).
**Expected behavior:** The 100KB threshold is the boundary. Content at exactly 100KB SHALL use the direct PyBytes path (not double JSON).
**Verification:** Test with content of exactly 102,400 bytes and verify no JSON double-encoding overhead.

---

## E-004: Bridge large content threshold — just under 100KB

**Scenario:** Memory content is 102,399 bytes (<100KB).
**Expected behavior:** Normal JSON path is used.
**Verification:** Test with 102,399 bytes and verify JSON round-trip.

---

## E-005: Bridge large content — binary/non-UTF8 data

**Scenario:** Memory content contains binary data that is not valid UTF-8.
**Expected behavior:** The PyBytes path SHALL handle arbitrary bytes without encoding errors. The JSON path SHALL fail gracefully with a clear error if content is not UTF-8.
**Verification:** Test with `bytes(range(256))` for PyBytes path.

---

## E-006: Entity not found — get returns None

**Scenario:** Client requests a session/memory/agent/skill that does not exist.
**Expected behavior:** Bridge returns `None`. Service returns `None`. API returns `404` with `{"detail": "Session not found"}`.
**Verification:** Test each entity type at all three layers.

---

## E-007: Entity not found — update returns None

**Scenario:** Client attempts to update a non-existent entity.
**Expected behavior:** Bridge returns `None`. Service returns `None`. API returns `404`.
**Verification:** Update non-existent ID → 404.

---

## E-008: Entity not found — delete is idempotent

**Scenario:** Client attempts to delete a non-existent entity.
**Expected behavior:** Bridge returns silently (no-op). API returns `204`.
**Rationale:** DELETE is idempotent per HTTP spec.
**Verification:** Delete non-existent ID → 204.

---

## E-009: Empty list operations

**Scenario:** No sessions/memories/agents/skills exist, client requests a list.
**Expected behavior:** Returns `200` with empty list `[]`.
**Verification:** List on empty storage → `[]`.

---

## E-010: Search with empty results

**Scenario:** Search query matches no memories.
**Expected behavior:** Returns `200` with `{"results": [], "total": 0, "page": 1, "limit": 20}`.
**Verification:** Search for non-existent term → empty results.

---

## E-011: Search with special characters

**Scenario:** Search query contains regex metacharacters `[ ] ( ) . ^ $ * + ? { } \ |`, or non-ASCII Unicode, emoji, or null bytes.
**Expected behavior:** The search endpoint SHALL escape or handle special characters without error. Empty results are valid. Null bytes SHALL be rejected with 422.
**Verification:** Search with `[`, `(`, `\`, `ñ`, `😀`, `\x00` → appropriate handling.

---

## E-012: Malformed request body — missing required fields

**Scenario:** POST/PUT request body omits a required field.
**Expected behavior:** Pydantic validation rejects the request with `422` and a list of missing fields.
**Verification:** POST `/sessions` with empty body → 422 with validation errors.

---

## E-013: Malformed request body — wrong type

**Scenario:** POST/PUT request body provides a string where a number is expected.
**Expected behavior:** Pydantic coercion or validation error. If the field is strict, return `422`.
**Verification:** POST with field type mismatch → 422.

---

## E-014: Extremely large request body

**Scenario:** POST/PUT request body exceeds typical limits (e.g., >50MB).
**Expected behavior:** FastAPI's `max_request_size` or a middleware SHALL reject large payloads with `413`.
**Verification:** POST >50MB payload → 413.

---

## E-015: Concurrent session creation — same ID

**Scenario:** Two clients concurrently create sessions with the same ID.
**Expected behavior:** The Rust engine's atomic create operation ensures one succeeds, the other returns an error (or succeeds with different ID if auto-generated). The bridge propagates the error.
**Verification:** Race two concurrent creates with the same ID → one 201, one 409.

---

## E-016: Config file corrupted

**Scenario:** `~/.contexter/config.yaml` exists but contains invalid YAML.
**Expected behavior:** The settings service SHALL log a warning and fall back to defaults. SHALL NOT crash.
**Verification:** Create a corrupted config file, restart settings service → defaults load, warning logged.

---

## E-017: Config file is a directory

**Scenario:** `~/.contexter/config.yaml` is a directory, not a file.
**Expected behavior:** The settings service SHALL log a warning and create/replace with a default config file.
**Verification:** Create directory at config path, restart → file created.

---

## E-018: Config file write permission denied

**Scenario:** The config directory exists but is not writable.
**Expected behavior:** The settings service SHALL log a warning and continue with in-memory defaults.
**Verification:** Chmod config dir to 500, attempt to write → warning logged, fallback to defaults.

---

## E-019: Port 8051 already in use

**Scenario:** Another process is listening on port 8051 when the FastAPI server starts.
**Expected behavior:** The server SHALL fail to start with `OSError: [Errno 98] Address already in use`. A clear error message SHALL be logged.
**Verification:** Start something on 8051, then start the FastAPI server → error logged.

---

## E-020: Port 8052 already in use

**Scenario:** Another process is listening on port 8052 when the MCP server starts.
**Expected behavior:** Same as E-019.
**Verification:** Start something on 8052, then start the MCP server → error logged.

---

## E-021: MCP client disconnects mid-request

**Scenario:** An AI agent sends an MCP tool request but disconnects before receiving the response.
**Expected behavior:** The MCP framework handles disconnection gracefully. No resource leak or crash.
**Verification:** Simulate disconnection after tool request → server continues running.

---

## E-022: MCP — unknown tool requested

**Scenario:** Client calls a tool that does not exist.
**Expected behavior:** MCP returns a "tool not found" error response.
**Verification:** Call `nonexistent_tool` → error response.

---

## E-023: MCP — unknown resource requested

**Scenario:** Client requests a resource with an unknown ID or type.
**Expected behavior:** MCP returns a "resource not found" response.
**Verification:** GET `contexter://session/nonexistent` → not found response.

---

## E-024: Bridge thread pool exhaustion

**Scenario:** Many concurrent requests exceed the bridge's `max_workers=4` thread pool.
**Expected behavior:** Requests queue in the `ThreadPoolExecutor`. No requests are lost. Response times SHALL increase but the system SHALL NOT crash.
**Verification:** Fire 20 concurrent bridge requests → all complete, no errors.

---

## E-025: Bridge call timeout

**Scenario:** A Rust operation takes unexpectedly long (e.g., massive checkpoint).
**Expected behavior:** The bridge SHALL NOT hang indefinitely. A timeout (configurable, default 30s) SHALL be applied to `to_thread()` calls.
**Verification:** Mock a slow Rust operation → timeout exception raised.

---

## E-026: Analytics — no data available

**Scenario:** Analytics endpoint is called when no sessions or memories exist.
**Expected behavior:** Returns `200` with zeroed metrics: `{"total_sessions": 0, "total_memories": 0, ...}`.
**Verification:** Call analytics on empty storage → 200 with zeroes.

---

## E-027: Analytics — division by zero

**Scenario:** Formulas like "average memories per session" when session count is zero.
**Expected behavior:** Service SHALL guard against division by zero and return `0` or `null`.
**Verification:** Analytics on empty storage → no ZeroDivisionError.

---

## E-028: Export — entity deleted before export completes

**Scenario:** User submits an export, the process starts, and the target entity is deleted mid-export.
**Expected behavior:** The export SHALL fail gracefully with an error status. The export record SHALL NOT be deleted — it SHALL show `failed` status.
**Verification:** Start export, delete entity, check export status → `failed`.

---

## E-029: Export — very large dataset

**Scenario:** Export request covers millions of memories.
**Expected behavior:** Export SHALL be processed asynchronously (status polling). The response SHALL NOT time out.
**Verification:** Export large dataset → immediate `202` with `status_id`, polling returns `in_progress` then `completed`.

---

## E-030: Feedback — rate limiting

**Scenario:** Client submits 100 feedback entries in 1 second.
**Expected behavior:** Rate limiting SHALL apply (configurable, default 5/min per IP). Returns `429`.
**Verification:** Rapid feedback submission → 429.

---

## E-031: Null bytes in search query

**Scenario:** Search query contains `\x00` (null byte).
**Expected behavior:** Bridge/Search service SHALL reject with `422` "Search query contains null byte".
**Verification:** Search with `hello\x00world` → 422.

---

## E-032: Empty string for entity ID

**Scenario:** Client requests `/api/v1/sessions/` (empty ID) or passes empty string as ID.
**Expected behavior:** FastAPI route validation SHALL reject empty IDs. Return `404` or `422`.
**Verification:** GET `/api/v1/sessions/` → 404 (empty path segment) or 422.

---

## E-033: Very long entity ID

**Scenario:** Client uses a 10,000-character entity ID.
**Expected behavior:** The ID field SHALL have a maximum length (256 chars recommended). Return `422` if exceeded.
**Verification:** POST with 10,000-char ID → 422.

---

## E-034: CLI — no configuration directory

**Scenario:** `~/.contexter/` does not exist and CLI is invoked.
**Expected behavior:** The CLI SHALL create `~/.contexter/` with default `config.yaml` on first invocation.
**Verification:** Remove `~/.contexter/`, run CLI → directory and config created.

---

## E-035: CLI — session create with invalid data

**Scenario:** CLI `session create` command with missing or invalid arguments.
**Expected behavior:** Click validation SHALL reject with a clear error message.
**Verification:** `contexter session create` without required args → error + usage.

---

## E-036: Async shutdown — cleanup resources

**Scenario:** The Python process receives SIGTERM or SIGINT.
**Expected behavior:** The server SHALL call the bridge `flush()` on shutdown to persist pending data. Graceful shutdown within a timeout (default 10s), then force exit.
**Verification:** Send SIGTERM → logs show flush, clean exit.

---

## E-037: MCP resource URI — malformed URI

**Scenario:** Client requests `contexter://invalid` or `contexter://session/` (no ID).
**Expected behavior:** MCP returns an error response for unknown or malformed resource URIs.
**Verification:** GET `contexter://session/` → error. GET `contexter://invalid` → error.

---

## E-038: Cache telemetry — empty cache

**Scenario:** `cache_telemetry()` called when the cache is empty.
**Expected behavior:** Returns a dict with zero counts, not an error.
**Verification:** Fresh engine → `cache_telemetry()` → `{"entries": 0, ...}`.

---

## E-039: Notification delete while being fetched

**Scenario:** User fetches `GET /notifications` while another process deletes notifications.
**Expected behavior:** The list operation returns the snapshot at query time. No crash.
**Verification:** Concurrent list + delete operations → both succeed.

---

## E-040: Semantic search with no index configured

**Scenario:** Search endpoint is called with `type=semantic` but no embedding model is configured.
**Expected behavior:** The search service SHALL return an error indicating that semantic search requires an embedding model configuration.
**Verification:** `GET /api/v1/search?q=test&type=semantic` with no embedding config → 400 or appropriate error.
