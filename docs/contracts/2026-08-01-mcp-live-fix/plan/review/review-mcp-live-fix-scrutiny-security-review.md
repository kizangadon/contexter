# Security Review Report

# MCP Server Live-Functionality Repair

> Security review of the repair that replaced the MagicMock engine stub with the real Rust engine bridge, restored handler schema parity, canonicalized the CONTEXTER_API_KEY env name, added {?_api_key} to resource URI templates, and added a mock-rejection guard to the bridge dispatch.

**Verdict:** PASS — auth model intact, no HIGH/MEDIUM findings (class: CONDITIONAL PASS (4 LOW + 3 informational hardening observations))

2026-08-01 · 7 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 4 |

> **Security Scope**
> Threat-modeled the changed surface of the MCP live-fix: (1) authentication (mcp_tools/auth.py require_api_key, api/deps.py get_api_key, mcp_server.py env read, {?_api_key} resource templates) against the frozen BUG-019/028/029 contract; (2) bridge dispatch (core/bridge.py) mock-rejection guard for bypasses, 100 KB bytes-path memory bounds, json.loads boundary; (3) handler input validation and structured error containment (no tracebacks to clients, no stdout pollution); (4) secrets hygiene (.gitignore, no key in stdout/stderr/error messages); (5) stdio boundary purity (stdout = JSON-RPC frames only).

---

## 02 · Vulnerability Findings

## 1. LOW — API key travels in MCP resource URI query strings

**Location:** `contexter-server/src/contexter_server/mcp_server.py` L204/L216/L228/L240 — `contexter://session/{id}{?_api_key}`, `contexter://memory/{id}{?_api_key}`, `contexter://agent/{id}{?_api_key}`, `contexter://analytics/overview{?_api_key}`.

**Issue:** The frozen BUG-029 design carries the shared secret in the resource URI (`?_api_key=<key>`). Over stdio transport this is local IPC, so exposure is limited, but MCP clients (e.g. OpenCode) commonly log the URIs they read, and any future transport (SSE on port 8052, `--sse` flag already present in run_mcp.py) puts the key in a URL that may be captured by proxies, browser history, or server access logs.

**Risk:** Credential disclosure if the server is ever reached over a network transport. No transport change occurred in this contract; the auth model is frozen per SPEC REQ-004.

**Recommendation (documentation-level):** Keep the frozen design; add a note in the contract/README that resource URIs containing `_api_key` should be treated as secret-bearing and that SSE deployment should be gated (TLS + non-internet exposure). The template list itself only broadcasts the `{?_api_key}` placeholder, not a concrete key — verified safe.

## 2. LOW — No input size bounds on MCP tool parameters (resource exhaustion + unbounded input echo)

**Location:** `contexter-server/src/contexter_server/mcp_tools/handlers.py` L42-44 (`handle_store_memory` UUID error echo), L46-48 (`session not found` echo), L66-86 (`handle_search_memories` — no max query length), L20-63 (`store_memory` — no max content length).

**Issue:** `store_memory` accepts arbitrarily large `content` (encoded to UTF-8 and written to RocksDB; the 100 KB threshold only switches transport, it does not bound size). `search_memories` accepts unbounded `query` strings. Error paths echo client input verbatim (`invalid session_id (not a valid UUID): {session_id}`) without truncation — a hostile client can send a multi-MB ID and receive it back, inflating response frames and, if the client logs error text, polluting logs.

**Risk:** DoS via resource exhaustion from a local MCP client; log/response bloat. Stdio transport limits practical exposure to processes that can already write to the pipe.

**Recommendation:** Add explicit length caps at the handler boundary (e.g. content ≤ 1 MB, query ≤ 4096 chars, id ≤ 512 chars) returning a structured validation error; truncate echoed values in error messages.

## 3. LOW — Engine-open failure at launch surfaces as a raw traceback on stderr

**Location:** `contexter-server/run_mcp.py` L49-60 — `main()` calls `build_services(engine_path)` unguarded; `StorageEngine.__init__` (`core/bridge.py` L124) calls `_SyncEngine.open(expanded_path)`.

**Issue:** If `CONTEXTER_PATH` points at an unopenable location, `Engine.open` raises and Python prints a full traceback. This goes to **stderr** (stdout purity preserved — PASS), and the process exits without hanging (EDGE_CASES met), but the failure message is a raw traceback rather than the clean one-line stderr message the edge-case table anticipates.

**Risk:** Minor; no stdout corruption, no secret material beyond the engine path itself. Traceback may expose internal paths if `CONTEXTER_PATH` is set oddly.

**Recommendation:** Wrap engine construction in `try/except` and emit a single structured stderr message (`Engine failed to open at <path>`) before `sys.exit(1)`, mirroring the existing `fastmcp not installed` pattern.

## 4. LOW — Truncated memory content is logged at the bridge

**Location:** `contexter-server/src/contexter_server/core/bridge.py` L56-95 (`_truncated_args_summary`), L160, L166-174 (logged in `bridge_call_end` / `bridge_call_failed`).

**Issue:** The args summary includes a ~97-character prefix of string/bytes arguments — i.e. the first ~97 chars of memory `content` reach stderr logs on every `create_memory`/`update_memory`/`search_memories` call. The API key never reaches the bridge (stripped at handler boundary), so no secret leak, but user-authored content (potentially sensitive) is persisted to logs. This is a pre-existing pattern retained by the fix, and truncation correctly prevents large-string allocation.

**Risk:** Sensitive-content disclosure in logs; log-growth noise.

**Recommendation:** Redact or hash content-bearing arguments in the summary (e.g. `content=<97-char prefix>` already; consider `<content omitted>` for memory payloads specifically).

## 5. informational — Legacy misspelled env var still read in the bridge

**Location:** `contexter-server/src/contexter_server/core/bridge.py` L112 — `os.environ.get("CONtexTER_BRIDGE_POOL_SIZE", "")`.

**Issue:** The bridge still reads the misspelled `CONtexTER_BRIDGE_POOL_SIZE` (same misspelling class that caused BUG-019). It is a thread-pool tuning knob, not a credential, and it degrades gracefully to 8 workers, so there is no security impact — but the fix canonicalized `CONTEXTER_API_KEY` everywhere while leaving this sibling behind, which is inconsistent and invites future confusion.

**Recommendation:** Canonicalize to `CONTEXTER_BRIDGE_POOL_SIZE` (optionally with the legacy name as a fallback) in a follow-up.

## 6. informational — Silent key collision in `_camelize_payload_keys`

**Location:** `contexter-server/src/contexter_server/core/bridge.py` L42-53.

**Issue:** If a payload ever contained both `memory_type` and `memoryType`, the dict comprehension would silently drop one (last wins). Today every payload originates from a single pydantic schema via `model_dump`, so both spellings cannot coexist; nested maps pass through untouched by design. Theoretical only.

**Recommendation:** Optionally assert `len(translated) == len(payload)` or document the invariant.

## 7. informational — Handler-level entry/success/failure logging absent (CON-003 auditability)

**Location:** `contexter-server/src/contexter_server/mcp_tools/handlers.py` (all handlers).

**Issue:** Handlers emit no logs of their own. Security events are still captured — auth failures via `mcp_tool.auth.missing_api_key` / `mcp_tool.auth.invalid_api_key` (auth.py L50/L57) and every storage op via `bridge_call_end` — so the audit trail is adequate for the threat model. SPEC CON-003 asks for tool-level entry/success/failure logs; that gap is an observability deviation rather than a vulnerability.

**Recommendation:** Consider logging tool name + outcome at the FastMCP wrapper layer (no arguments, no payloads).

---

## 03 · Security-Critical Code Highlights

**Verified secure — no findings:**

- **Constant-time comparison preserved (REQ-004 / BUG-028):** `require_api_key` uses `hmac.compare_digest(api_key, expected)` (`auth.py` L56); REST layer `deps.py` L64 uses `hmac.compare_digest(token, api_key)`. No `==` comparison on secrets anywhere in the changed surface.
- **Canonical env name everywhere:** `CONTEXTER_API_KEY` read in `auth.py` L45, `mcp_server.py` L68, `api/deps.py` L51, `main.py` (docstring). Regression test `TestEnvVarCanonicalName` (tests/mcp/test_mcp_auth.py) proves the legacy `CONtexTER_API_KEY` no longer gates anything.
- **No secret in logs or errors:** Logs carry only event names (`mcp_server.api_key_configured`, `mcp_tool.auth.invalid_api_key`); error messages reference the env var name, never the key value. `mcp_server.py` L68-74 logs presence/absence only.
- **Auth-first ordering:** `require_api_key(_api_key)` is the first statement in all 8 tool handlers and all 4 resource handlers — before service lookups or existence checks, so unauthenticated callers get no existence/timing oracle.
- **`_api_key` never persisted:** stripped at the handler boundary; not present in `MemoryCreate`, `SearchQuery`, `ExportRequest`, or any bridge payload.
- **Mock-rejection guard has no bypass:** class-level check (mock attr on real class → `TypeError`, bridge.py L144), instance-level check (mock attr on real instance → `TypeError`, L151), wholesale-mock tolerance is short-circuited to test doubles only. A subclass-of-Engine overriding methods with Mocks is still caught (class is not a Mock).
- **stdio boundary purity:** all `print()` calls in `run_mcp.py` target `sys.stderr` (L59, L66); bridge/mcp_server log via structlog (stderr). No stdout writes in the changed surface. AC-11 preserved.
- **Structured error containment:** `MCPAuthError` subclasses `ValueError` so FastMCP serializes it as a JSON-RPC error, not a crash; engine errors re-raise through the bridge to FastMCP's error path; no tracebacks reach stdout.
- **Secrets/scratch hygiene:** `.gitignore` now covers `**/docs/tests/` at any depth; the MagicMock stub (`contexter_core.py`) is deleted, and `tests/core/test_engine_real.py` + `test_mcp_launcher_wiring.py` assert the live path holds a real `StorageEngine` over the real Rust extension.
- **Supply chain:** `fastmcp~=3.4.0` pin replaces unbounded `fastmcp>=0.3` (`pyproject.toml`).
- **Memory bounds in bridge logging:** `_truncated_args_summary` avoids materializing 100 KB+ reprs in log lines.

---

## 04 · Remediation Recommendations

> **Must Fix**
> None. Zero Critical/High/Medium findings; the frozen auth model (BUG-019/028/029) is intact.

> **Should Fix**
> - F2: Add length caps on `store_memory` content, `search_memories` query, and echoed IDs in handler error messages (handlers.py).
- F3: Wrap `Engine.open` in `run_mcp.py` to emit a clean one-line stderr failure message instead of a raw traceback.
- F4: Consider omitting/redacting content-bearing args from bridge log summaries.

> **Consider**
> - F1: Document that `{?_api_key}` URIs are secret-bearing; gate any future SSE/network transport behind TLS + non-public exposure.
- F5: Canonicalize `CONtexTER_BRIDGE_POOL_SIZE` → `CONTEXTER_BRIDGE_POOL_SIZE`.
- F6: Assert no key loss in `_camelize_payload_keys` (defensive invariant).
- F7: Add wrapper-level tool-name/outcome logging to satisfy CON-003 observability.

---

_Generated by Security Architect · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
