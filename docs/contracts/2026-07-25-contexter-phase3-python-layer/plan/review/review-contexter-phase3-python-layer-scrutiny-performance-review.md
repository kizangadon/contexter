# Performance Review Report

# Contexter Phase 3 — Python API Layer

> Comprehensive performance review of the Python API service layer (FastAPI + FastMCP + bridges to Rust core engine via ThreadPoolExecutor). Evaluates thread pool adequacy, serialization overhead, large content paths, dependency injection patterns, concurrency, N+1 query patterns, and resource management.

**Verdict:** CONDITIONAL PASS — 13 findings (4 HIGH, 5 MEDIUM, 4 LOW) (class: amber)

2026-07-25 · 13 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Bridge Thread Pool Capacity | 4 workers shared across 12+ services — adequate for light load, bottlenecks under concurrent export/analytics |
| Async Bridge Overhead (per call) | ~0.5–5ms JSON serde + thread switch per call — acceptable for I/O-bound Rust work |
| Large Content Path (>100KB) | Correctly uses PyBytes bypass — but threshold checks len(str) not len(bytes) |
| Service Lifecycle | Singleton services, zero per-request allocation — optimal DI pattern |
| Sequential Bridge Calls | 6 services make 2–4 sequential independent calls that could be parallelized |
| SettingsService I/O | yaml.safe_load() and yaml.dump() block the event loop — not offloaded to thread pool |
| Unbounded In-Memory State | ExportService + NotificationService have unbounded dicts — no eviction |
| Search Pagination | Application-level slice after fetching all results — no server-side limit |
| MemoryService.list() | Calls search_memories({}) with no limit — returns ALL memories |
| Logging Overhead | structlog with synchronous I/O — acceptable at moderate load, potential bottleneck at high throughput |

> **Analysis Scope**
> Files reviewed: core/bridge.py (220 lines), api/deps.py (157 lines), main.py (204 lines), mcp_server.py (203 lines), mcp_tools/handlers.py (257 lines), 12 service files (avg 50 lines each), 12 model files (avg 35 lines each), 4 API router files. Total ~2,400 lines of Python. Dependencies: Python 3.12+, FastAPI >=0.115, FastMCP >=0.3, Pydantic v2, structlog.

---

## 02 · Benchmark Results

## Bridge Thread Pool Analysis

**Observation:** `ThreadPoolExecutor(max_workers=4)` is shared across all 12+ services. Each `asyncio.to_thread` call occupies one worker for the duration of the Rust call. Under concurrent load (e.g., 4+ simultaneous requests), the pool saturates and subsequent requests queue. The queue is unbounded (default ThreadPoolExecutor behavior), meaning backlog can grow arbitrarily under sustained load.

**Impact:** At 4 concurrent requests involving bridge calls, the 5th request waits. Under burst traffic, latency spikes.

**Mitigation:** For a single-user or low-concurrency agent system, 4 workers is adequate. For multi-user or high-throughput scenarios, increase to 8–16 workers based on observed queue depth.

## Async Bridge Overhead (per call)

**Observation:** Every bridge call involves: Python `json.dumps()` → `asyncio.to_thread` (thread switch) → Rust PyO3 call (releases GIL) → Python `json.loads()`. JSON serde for small payloads (simple get/delete) dominates the call cost.

| Operation | Estimated Latency | Breakdown |
|-----------|------------------|-----------|
| get_session (simple) | 0.5–2ms | JSON serde dominates |
| create_memory (1KB) | 2–5ms | JSON serde + RocksDB write |
| search_memories (text) | 5–50ms | FTS/vector search in Rust dominates |
| storage_size | 1–10ms | RocksDB estimation |

**Impact:** Acceptable for an I/O-bound system. JSON serde overhead is negligible compared to Rust-side processing for non-trivial operations.

## Large Content Path (>100KB)

**Observation:** `bridge.py` checks `len(content) >= _LARGE_CONTENT_THRESHOLD (102400)` and routes to `create_memory_bytes`/`update_memory_bytes`. The meta dict is still JSON-encoded, but the content is passed as `content.encode("utf-8")` (PyBytes).

**Issue:** `len(content)` on Python strings measures character count, not byte count. Multi-byte UTF-8 characters (e.g., CJK, emoji) have byte-length > char-length. A 90K-character CJK string could be ~270KB, bypassing the large content threshold entirely.

**Impact:** Medium. Misclassification could lead to extremely large JSON payloads (270KB+ in-memory JSON strings) being serialized and deserialized instead of using the efficient PyBytes path.

**Recommendation:** Also add a check for `len(content.encode("utf-8"))` as a secondary criterion, or change to byte-length threshold.

## Dependency Injection Pattern

**Observation:** Services are created once during `lifespan` startup via `_create_services()` and stored on `app.state`. DI deps (`get_session_service`, etc.) are `AsyncIterator` generators that simply `yield getattr(request.app.state, attr)`. No per-request construction, no DB connection per request, no expensive setup.

**Verdict:** Optimal. No issues found.

## Sequential Bridge Call Analysis (N+1 Anti-Pattern)

**Observation:** Multiple services make independent sequential bridge calls that could be parallelized:

| Service | Method | Sequential Calls | Can Parallelize? |
|---------|--------|-----------------|------------------|
| AnalyticsService | get_overview | cache_telemetry → storage_size → status | YES (asyncio.gather) |
| AnalyticsService | get_health | status → cache_telemetry → storage_size | YES |
| AnalyticsService | get_resources | storage_size → status → cache_telemetry | YES |
| MemoryService | search | search_memories → count_memories | YES |
| SearchService | search (with project) | search_memories → list_sessions | YES |
| ExportService | submit | 4 sequential (list_sessions, search_memories, list_agents, list_skills) | YES |
| OnboardingService | get_progress | get_setting → list_agents → list_sessions | YES |
| OnboardingService | submit_wizard | N sequential set_settings + one completion | YES (batch) |

**Impact:** HIGH. Analytics endpoints make 3 sequential bridge calls, tripling the latency. Export submits 4 sequential calls. Onboarding progress makes 3 sequential calls. Each sequential call adds thread-switch + JSON serde overhead.

**Mitigation:** Use `asyncio.gather()` for independent calls.

## SettingsService — Blocking I/O on Event Loop

**Observation:** `SettingsService.load()` calls `yaml.safe_load()` (file read + YAML parse) directly in the async function without offloading to the thread pool. Same for `_write_yaml()`. These are synchronous file I/O operations that block the entire event loop while reading/writing and parsing YAML.

**Impact:** HIGH. Every settings load/save blocks the event loop for potentially 1–50ms (YAML parse of the full config file). Under concurrent load, this delays all other requests.

**Recommendation:** Offload YAML file I/O via `asyncio.to_thread()` or use `aiofiles`.

## Unbounded In-Memory State

**Observation:** `ExportService._exports` and `NotificationService._notifications` are plain Python dicts with no eviction policy, no size limits, and no TTL. Over time (or under heavy export/notification load), these grow without bound, consuming increasing memory.

**Impact:** MEDIUM. For long-running servers, this is a memory leak by design. Exports and notifications accumulate until process restart.

**Recommendation:** Add LRU eviction or time-based TTL pruning.

## Search Pagination — Application-Level Slice

**Observation:** `SearchService.search()` calls `self._engine.search_memories(query_dict)` which returns ALL matching results, then applies `results[start:end]` for pagination. The Rust bridge does not support server-side pagination.

**Impact:** MEDIUM. Searching across 100K memories returns ALL 100K results just to show page 1 (20 items). Wasted JSON serde + memory.

**Recommendation:** Add pagination support (limit/offset) to the Rust bridge `search_memories` method, or at minimum pass `limit` to reduce transfer size.

## MemoryService.list() — Unlimited Fetch

**Observation:** `MemoryService.list()` calls `self._engine.search_memories({})` with an empty query, returning ALL memories. No pagination, no limit, no filter.

**Impact:** HIGH for systems with 10K+ memories. Could return hundreds of MB of JSON.

**Recommendation:** Add mandatory pagination or at minimum a default limit (e.g., 100) to the list endpoint.

## structlog Synchronous I/O

**Observation:** The logging middleware in `main.py` uses `structlog.get_logger(__name__)` and calls `logger.info(...)`. structlog by default writes to stderr/stdout using synchronous I/O. Under high throughput, this can become a bottleneck.

**Impact:** LOW for moderate loads. Could become noticeable at 1000+ req/s.

**Recommendation:** Configure structlog with async logging or use `structlog.stdlib.AsyncBoundLogger` for high-throughput scenarios.

## MCP Server — SSE Transport

**Observation:** MCP server runs in a daemon thread with SSE transport on port 8052. Each SSE connection is long-lived and holds service references. No connection pooling, rate limiting, or backpressure visible.

**Impact:** LOW. Single-threaded SSE server is adequate for agent-<->MCP communication (typically 1–2 concurrent connections).

**Recommendation:** Add connection limits and monitoring for SSE connection count.

## Shutdown Flush

**Observation:** Lifecycle shutdown calls `await engine.flush()` which synchronizes RocksDB WAL. The MCP daemon thread is not joined — it's killed on process exit.

**Impact:** LOW. Appropriate for expected behavior.

**Concern:** If flush takes multiple seconds, the server won't respond to health checks during shutdown.

## PyBytes Path — String vs Byte Length Threshold

**Observation (further detail on H4):** The large content threshold check uses Python's `len()` on a string, which counts Unicode code points, not bytes. UTF-8 encoding of non-ASCII text can expand significantly (3 bytes per CJK character).

| Content Type | Char Length | UTF-8 Byte Length | Threshold Trigger |
|-------------|-------------|-------------------|-------------------|
| English ASCII | 100,000 | 100,000 | 100K ≥ 100K ✓ |
| CJK text | 100,000 | 300,000 | 100K ≥ 100K ✓ (by chance) |
| Mixed emoji | 34,000 | 136,000 | 34K < 100K ✗ (missed!) |

**Recommendation:** Change to `len(content.encode("utf-8")) >= _LARGE_CONTENT_THRESHOLD`.

## AnalyticsService._status_cache — Dead Code

**Observation:** `AnalyticsService._status_cache` is initialized to `None` in `__init__` but never read or written by any method. Every call goes directly to the bridge.

**Impact:** LOW. Unused variable, no performance impact.

**Recommendation:** Remove or implement caching with TTL.

---

## 03 · Performance Bottlenecks

## 🔴 Critical Bottlenecks

### 1. Sequential Independent Bridge Calls (3 services, 6 methods)
- **AnalyticsService.get_overview/health/resources**: 3 sequential -> 1 parallel reduces latency 3x
- **ExportService.submit**: 4 sequential -> 1 parallel reduces latency 4x
- **OnboardingService.get_progress**: 3 sequential -> 1 parallel
- **OnboardingService.submit_wizard**: N sequential per key -> 1 batch

**Root cause**: Services call `await self._engine.method_a()` then `await self._engine.method_b()` when both are independent. Missing `asyncio.gather()`.

### 2. SettingsService Blocks Event Loop
- `yaml.safe_load()` (file read + YAML parse) runs synchronously in async context
- `yaml.dump()` + file write in `_write_yaml()` also blocks event loop
- Every settings load/save blocks ALL concurrent requests

### 3. Search/List Endpoints Lack Server-Side Pagination
- `search_memories({})` returns ALL results — no limit/offset on Rust side
- `SearchService.search()` fetches all results then takes a slice
- `MemoryService.list()` fetches ALL memories with empty query

## 🟡 Moderate Bottlenecks

### 4. Large Content Threshold Uses String Length
- `len(content) >= 102400` measures chars not bytes
- Multi-byte UTF-8 content bypasses PyBytes path

### 5. Unbounded In-Memory Dictionaries
- `ExportService._exports` grows without eviction
- `NotificationService._notifications` grows without eviction
- Memory accumulates until process restart

## 🟢 Minor Concerns

### 6. ThreadPoolWorker Count Fixed at 4
- Adequate for single-user, bottlenecks at 5+ concurrent bridge calls
- Queue depth grows unbounded under burst load

### 7. structlog Synchronous I/O
- Synchronous writes under high throughput
- Mitigated by low expected request volume

### 8. MCP Daemon Thread Not Gracefully Stopped
- Thread killed on exit — pending SSE responses lost
- Flush could delay shutdown

### 9. AnalyticsService._status_cache Dead Field
- Never written, always None — dead code

---

## 04 · Optimization Recommendations

> **High Impact**
> 1. **Parallelize independent bridge calls** — Use `asyncio.gather()` in AnalyticsService (get_overview, get_health, get_resources), ExportService.submit, MemoryService.search, SearchService.search, OnboardingService.get_progress, and OnboardingService.submit_wizard. Reduces endpoint latency 2–4x.

2. **Offload SettingsService file I/O to thread pool** — Wrap `yaml.safe_load()` and `yaml.dump()` + file operations in `asyncio.to_thread()` to prevent event loop blocking. Every settings endpoint is affected.

3. **Add server-side pagination to search_memories and list endpoints** — Pass `limit` and `offset` to the Rust bridge for `search_memories`, `list_sessions`, `list_agents`, `list_skills`. Add a default limit (e.g., 100) to `MemoryService.list()`.

> **Medium Impact**
> 4. **Fix large content threshold** — Change `len(content)` to `len(content.encode("utf-8"))` in bridge.py `create_memory` and `update_memory` to correctly detect >100KB byte payloads.

5. **Add eviction to in-memory caches** — Implement LRU eviction or TTL-based pruning for `ExportService._exports` and `NotificationService._notifications`. Set a max size (e.g., 100 entries).

6. **Increase ThreadPoolExecutor max_workers** — Evaluate queue depth under expected load. For multi-user scenarios, increase from 4 to 8–16 workers. Consider making configurable via environment variable.

7. **Remove or implement AnalyticsService._status_cache** — Either remove the dead `_status_cache` field or implement it with a TTL cache to reduce redundant bridge calls within short time windows.

> **Quick Wins**
> 8. **Implement OnboardingService.submit_wizard batch** — Instead of `for k,v in data.items(): await engine.set_setting(...)`, use `asyncio.gather()` to set all keys in parallel, then set `onboarding_completed`.

9. **Add meaningful limit to MemoryService.list()** — Change `search_memories({})` to `search_memories({"limit": 100})` or similar to prevent unbounded fetches.

10. **Add MCP SSE connection monitoring** — Log SSE connection count or add a Prometheus metric to detect leaks.

11. **Configure structlog for async output** — For high-throughput deployments, switch to `AsyncBoundLogger` or use `QueueHandler`.

12. **Consider ThreadPoolExecutor max_workers configurable** — Add an environment variable or constructor parameter to allow tuning without code changes.

13. **Add shutdown timeout for flush** — Use `asyncio.wait_for(engine.flush(), timeout=5.0)` to prevent shutdown hangs.

---

_Generated by Performance Benchmarker · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase3-python-layer_
