# Security Review Report

# MCP Server Live-Functionality Repair — Auto Bug Loop Iteration 1

> Security re-review of the ENTIRE feature scope after 18 bug-contract fixes (validation caps, error-shape drift, launch-error handling, bridge log hygiene, env canonicalization, handler observability, analytics telemetry mapping, limit pushdown/clamping, store_memory schema conformity, bridge double-encode, agent/skill schema drift, scratch cleanup). Baseline findings F1-F7 re-verified against the current working tree (HEAD 27e031d + uncommitted changes).

**Verdict:** CONDITIONAL PASS (class: 0 Critical / 0 High / 2 Low / 3 informational)

2026-08-01 · 5 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 0 |
| Low | 2 |

> **Security Scope**
> Threat-modeled the full changed surface of Iteration 1: (1) `_api_key` auth path — constant-time compare (hmac.compare_digest, mcp_tools/auth.py + api/deps.py), byte-identical MCPAuthError/ValueError serialization, `{?_api_key}` RFC-6570 templates on all 4 resources (mcp_server.py L198/L210/L222/L234), auth-first ordering in all 8 tools + 4 resources; (2) input validation — content cap 1 MB, query cap 10k, limit clamping (session 0..10_000, search 1..100), export allowlist, bounded validation messages, no unbounded echo of client input; (3) secrets/scratch hygiene — no CONtexTER_ reads, API key presence-only logging, .gitignore covers docs/tests, mock stub removed, bridge mock-import guard; (4) structured error paths — HandlerError/MCPAuthError as isError frames, no success smuggling, mid-call engine failure contained (verified live: isError=True, no traceback), stdout purity (stderr-only prints); (5) launch error handling — clean one-line stderr + exit 2 + full diagnostics in launch log (3 failure modes); (6) telemetry — real engine camelCase mapping, _safe_get logs key mismatches, overview exposes counts/storage/uptime only; (7) bridge log hygiene — 64-char cap on content args. 298 security-relevant tests executed, all green.

---

## 02 · Vulnerability Findings

### F-IT1-01 — LOW — Not-found error path echoes unbounded client-controlled input

**Location:** `contexter-server/src/contexter_server/mcp_tools/handlers.py` — `not_found_error(resource)` call sites L254 (`get_session`), L317 (`get_agent_info`), L442 (`get_memory`), L470 (`list_memories`), L498 (`get_analytics_overview`); helper defined in `mcp_tools/errors.py`.
**Contract refs:** REQ-IV-005 / EC-IV-009 (validation must reject oversized inputs; no unbounded echo of attacker-controlled data), AC-IV-001 (input validation at the boundary).
**Description:** `not_found_error()` interpolates the raw resource id verbatim into the error message with no length bound. The `_bounded(value, max_chars=64)` helper exists and is used for the export-format error path, but is NOT applied to resource-id echoes. Empirically confirmed: a tool call with a 1,000,000-char id returns a HandlerError whose message is 1,000,020 chars (full input echoed after `Resource not found: `). Blast radius: unbounded error-message amplification across 5 handlers; a single hostile request forces the server to build (and the client to receive) a multi-MB error payload — a low-severity DoS/amplification vector and a violation of the "no unbounded echo" validation intent.
**Evidence:** live client repro (1MB id → 1,000,020-char message); code inspection shows `_bounded` unused on all five not-found sites.
**Fix boundary:** apply `_bounded(id, 64)` (or a dedicated resource-id truncator) at each `not_found_error` call site in handlers.py.

### F-IT1-02 — LOW — Handler `call_received`-style logs bind unbounded client-controlled input

**Location:** `contexter-server/src/contexter_server/mcp_tools/handlers.py` — log bindings at L146, L241, L275, L304, L332, L429, L489.
**Contract refs:** REQ-HO-002 (bounded log content; no raw payloads in logs), B9 (handler-observability bounds), EDGE_CASES observability section.
**Description:** The per-call success/attempt log lines bind raw client-provided ids (`session_id`, `resource_id`) and search/project values into structlog bindings without applying `_bounded`. Content arguments ARE bounded (per B9), but the id bindings are not: a 1MB id produces a ~1MB log line through the `call_received`/`handle_*` logging path. Confirmed by code inspection (L241 binds raw id) — the same 1MB-id request that triggers F-IT1-01 also writes an unbounded log line. Blast radius: log inflation / storage amplification from a single hostile request; obscures the audit trail.
**Evidence:** code inspection; log binding at L241 carries raw id (no `_bounded`).
**Fix boundary:** wrap id/project/type bindings with `_bounded` at all listed log sites (same helper as F-IT1-01).

### F-IT1-03 — informational — API key travels in MCP resource URIs as query-string parameters (documentation gap)

**Location:** `mcp_server.py` L198/L210/L222/L234 (`contexter://session/{id}{?_api_key}`, `contexter://memory/{id}{?_api_key}`, `contexter://agent/{id}{?_api_key}`, `contexter://analytics/overview{?_api_key}`).
**Contract refs:** baseline finding F1 (frozen BUG-029 design — auth must remain unchanged).
**Description:** Re-stated from baseline F1. The URI templates use RFC-6570 `{?_api_key}` placeholder syntax (safe — placeholders never materialize server-side without the client supplying the key), but resource URIs are secret-bearing when a key is present. The README does not document that these URIs must be treated as secrets nor that the MCP endpoint (stdio/SSE, port 8052) must be gated (non-public, TLS-terminated). Design itself remains accepted and unchanged per contract; the documentation gap persists.
**Evidence:** README review; template inspection.
**Fix boundary:** README note (documentation only, no code change).

### F-IT1-04 — informational — `_camelize_payload_keys` silent key-collision invariant not asserted

**Location:** `core/bridge.py` L42-53.
**Contract refs:** baseline finding F6; camelization-coverage bug contract.
**Description:** Re-stated from baseline F6. `_camelize_payload_keys` converts snake_case to camelCase by naive title-case join; two distinct snake_case keys that camelize identically would silently collide. The camelization live-coverage tests substantially reduce drift risk, but no unit test asserts the collision-free invariant for the current engine API surface. No collision found in the current surface.
**Evidence:** bridge.py inspection; camelization test suite green.
**Fix boundary:** optional invariant test (assertion that `key != key2 ⇒ camelize(key) != camelize(key2)` for the engine payload keys).

### F-IT1-05 — informational — `MAX_REQUEST_BODY` env read lacks canonical prefix (literal REQ-EV-001 deviation)

**Location:** `contexter-server/src/contexter_server/main.py` L205.
**Contract refs:** REQ-EV-001 ("All env var reads use the canonical CONTEXTER_ prefix only"), env-var-canonicalization bug contract.
**Description:** `os.getenv("MAX_REQUEST_BODY", ...)` reads a non-prefixed variable. Zero security impact (REST body limit only; unrelated to the typo-confusion class, and `CONTEXTER_MAX_REQUEST_BODY` would be the canonical form), but it is a literal deviation from the canonicalization requirement as written. All other server env reads verified canonical (`CONTEXTER_API_KEY`, `CONTEXTER_RATE_LIMIT_ENABLED`, `CONTEXTER_RATE_LIMIT`, `CONTEXTER_ENABLE_DOCS`, `CONTEXTER_LOG_FILE`, `CONTEXTER_PATH`, `CONTEXTER_MCP_PORT`, `CONTEXTER_*` in run_mcp.py).
**Evidence:** grep of all `getenv`/`environ.get` reads; main.py inspection.
**Fix boundary:** rename read to `CONTEXTER_MAX_REQUEST_BODY` (keep default) or document exclusion.


---

## 03 · Security-Critical Code Highlights

- **Constant-time auth:** `hmac.compare_digest` used for `_api_key` verification in both `mcp_tools/auth.py` and `api/deps.py` — no timing side channel; byte-identical `MCPAuthError`/HTTP 401 serialization for missing vs. wrong key (tested matrix).
- **Auth-first ordering:** all 8 tools and 4 resources enforce `_api_key` before any engine access; resource templates use RFC-6570 `{?_api_key}` placeholders only.
- **No secrets in code/logs/models:** no `api_key`/`secret`/`token`/`password`/`credential` fields in any pydantic model; API key is presence-logged only (never the value); no hardcoded credentials; README documents canonical `CONTEXTER_` envs without secret values.
- **Structured, bounded errors:** `HandlerError`/`MCPAuthError` serialize as isError frames (never success-smuggled); validation error messages bounded; launch failure = clean one-line stderr + exit code 2 + full diagnostics in server-side launch log (3 failure modes tested, no traceback on stderr).
- **Mid-call engine failure containment:** empirically verified — a RuntimeError raised inside an engine call surfaces as a structured JSON-RPC isError frame with the exception detail, no stack trace, no process crash.
- **Input validation at the boundary:** content capped at 1 MB, query at 10k chars, session-list limit clamped 0..10_000, search limit 1..100, export format allowlist (json/yaml/csv) — caps enforced server-side and mirrored in pydantic models (defense in depth).
- **Rate limiting on:** `CONTEXTER_RATE_LIMIT_ENABLED` defaults true, 100/min default, IP-keyed — applies to HTTP control-plane; MCP stdio is per-process.
- **Log hygiene:** bridge content-bearing args truncated to 64 chars; error logs carry correlation_id/duration_ms/error_kind/tool but no payloads; stdout stays clean (all launch prints to stderr).
- **Analytics telemetry:** `_safe_get` logs missing-key mismatches instead of silently defaulting; overview exposes counts/storage/uptime only — no memory content or PII.
- **Scratch/secret cleanup:** `.gitignore` covers `**/docs/tests/`; MagicMock stub removed; bridge refuses mock/placeholder engine imports at load time.


---

## 04 · Remediation Recommendations

> **Must Fix**
> None in this iteration (0 Critical, 0 High).

> **Should Fix**
> - **F-IT1-01 (LOW):** apply `_bounded(id, 64)` (or a resource-id truncator) at the five `not_found_error` call sites in handlers.py — L254, L317, L442, L470, L498 — to stop unbounded echo of client-controlled ids in error messages (REQ-IV-005/EC-IV-009).
- **F-IT1-02 (LOW):** wrap id/project/type log bindings with `_bounded` at the handler log sites (L146, L241, L275, L304, L332, L429, L489) so a hostile oversized id cannot inflate log lines (REQ-HO-002 / B9 bounds).

> **Consider**
> - **F-IT1-03 (informational):** README note: MCP resource URIs carrying `_api_key` are secret-bearing; document SSE endpoint (port 8052) gating (non-public / TLS) — documentation only.
- **F-IT1-04 (informational):** add invariant test asserting `_camelize_payload_keys` is collision-free over the current engine payload keys.
- **F-IT1-05 (informational):** align `MAX_REQUEST_BODY` env read with the canonical `CONTEXTER_` prefix (or document the deliberate exclusion).

---

_Generated by Security Architect · 2026-08-01 · Validation Contract: 2026-08-01-mcp-live-fix_
