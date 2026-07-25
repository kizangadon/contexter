# Security Review Report

# Contexter Phase 3 — Python API Layer (Iteration 1 Re-Verification)

> Auto Bug Loop Iteration 1 security re-verification of the Contexter FastAPI REST API. Audits all security controls implemented under BUG-008 (API key auth, security headers, body limits, TrustedHostMiddleware, docs gating, path traversal, debug mode) plus an assessment of the full application security posture across 16 API routers, the MCP server, settings endpoints, and the CLI.

**Verdict:** CONDITIONAL PASS (class: pass)

2026-07-25 · 7 findings · Security Architect (Scrutiny)

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 1 |
| Medium | 2 |
| Low | 4 |

> **Security Scope**
> Full re-verification of all 5 security-related bug contracts (BUG-008: Security Middleware covering 8a-8g) plus complete application security posture review across all API routers, models, services, MCP server, CLI, and test coverage. Original findings: 2 CRIT, 4 HI, 5 MED, 4 LOW (16 total).

---

## 02 · Vulnerability Findings

## F-01 (HIGH) — Sensitive data exposure via LLM provider settings endpoint

**Location:** `contexter-server/src/contexter_server/services/settings_service.py:65-87`, `contexter-server/src/contexter_server/models/settings.py:47-48`

**Evidence:** `SettingsService.get_section("llm_providers")` at line 74 calls `model_dump()` on `LLMProviderConfig` objects, which includes the `api_key` field (model field at `models/settings.py:48`). The response is returned through `GET /api/v1/settings/llm_providers` to any authenticated API consumer.

**Impact:** Any user with a valid Contexter API key can retrieve LLM provider API keys (OpenAI, Anthropic, etc.) in plaintext. Blast radius includes downstream LLM provider account compromise and billing abuse.

**Fix:** Add a response model for LLM provider settings that excludes `api_key`, or redact the `api_key` field in `get_section()` before returning. Example fix:
```python
def _sanitize_section(section_name: str, val: object) -> dict:
    if section_name == "llm_providers" and isinstance(val, list):
        return {"items": [{k: ("***redacted***" if k == "api_key" else v) for k, v in p.model_dump().items()} for p in val]}
```

---

## F-02 (MEDIUM) — MCP server has no application-level authentication enforcement

**Location:** `contexter-server/src/contexter_server/mcp_server.py:67-73`

**Evidence:** The MCP server only logs a warning if `CONtexTER_API_KEY` is not set (line 71-73). It does NOT enforce authentication on any tool or resource call. The comment at line 66 states "MCP transport-level auth can be configured separately (e.g. via reverse proxy)."

**Impact:** The MCP server on port 8052 (SSE transport) is fully accessible without authentication. Tools like `store_memory`, `search_memories`, `export_data` can be called by anyone who can reach port 8052. No rate limiting or auth exists at the application layer.

**Mitigation:** Currently documented as a reverse-proxy responsibility. Acceptable if network-level controls (firewall, ingress) restrict port 8052. Flagged for awareness.

**Fix (optional):** Add a FastMCP `@mcp.auth` check or validate the API key in each handler. If the SSE transport supports custom headers, pass the Bearer token through.

---

## F-03 (MEDIUM) — No rate limiting on any API endpoint

**Location:** `contexter-server/src/contexter_server/main.py` (no rate-limiting middleware or dependency anywhere)

**Evidence:** No `slowapi`, `fastapi-limiter`, or custom rate-limiting middleware is imported or configured. All 16 API v1 routers, including POST (create/update/delete) endpoints and search endpoints, have zero rate limiting.

**Impact:** Brute-force attacks against the API key are possible without rate throttling. The search endpoint (`GET /api/v1/search?q=...`) and memory search can be abused for resource exhaustion. This was an original MEDIUM finding that remains unaddressed.

**Fix:** Integrate `slowapi` or a custom rate-limiting middleware. Apply per-client-IP or per-API-key limits (e.g., 100 req/min for reads, 20 req/min for writes).

---

## F-04 (LOW) — Non-timing-safe API key comparison

**Location:** `contexter-server/src/contexter_server/api/deps.py:63`

**Evidence:** The API key comparison uses Python's `!=` operator:
```python
if token != api_key:
    raise HTTPException(...)
```
This is a standard string comparison that short-circuits on the first differing character.

**Impact:** Theoretical timing side-channel could allow character-by-character brute-force of the API key over a local network. Requires thousands of measurements per character. Low severity in practice because the API key is a long random string and the attacker needs network access.

**Fix:** Use `hmac.compare_digest(token, api_key)` for constant-time comparison:
```python
import hmac
if not hmac.compare_digest(token, api_key):
    raise HTTPException(...)
```

---

## F-05 (LOW) — Body size limit bypass via chunked transfer encoding

**Location:** `contexter-server/src/contexter_server/main.py:174-191`

**Evidence:** The `_add_body_size_limit_middleware` function only checks the `Content-Length` header (line 180-190). If a request uses `Transfer-Encoding: chunked` (which omits `Content-Length`), no size check is performed. The request body will be read entirely into memory by FastAPI/Starlette.

**Impact:** An attacker can send a very large chunked body that bypasses the 50 MiB `MAX_REQUEST_BODY` limit, potentially causing memory exhaustion (OOM).

**Fix:** Implement actual body streaming with size tracking, or use a reverse proxy (nginx) with `client_max_body_size` for chunked transfer enforcement. At minimum, document that this middleware only covers `Content-Length` and recommend edge-level enforcement for chunked traffic.

---

## F-06 (LOW) — Default max body size of 50 MiB is permissive for text API

**Location:** `contexter-server/src/contexter_server/main.py:179`

**Evidence:** Default `MAX_REQUEST_BODY` is 52,428,800 bytes (50 MiB). The task specification indicated a 1 MB default should have been configured.

**Impact:** Memory exhaustion risk. A single 50 MiB JSON body consumes significant memory during parsing. For a text-based memory/session API, the legitimate request body size is typically <1 MB.

**Fix:** Reduce default to 1 MB (`1 * 1024 * 1024`) and document the env var override for special cases.

---

## F-07 (LOW) — File diff endpoint query params lack path validation

**Location:** `contexter-server/src/contexter_server/api/files.py:83-93`

**Evidence:** The `GET /files/{hash}/diff` endpoint accepts `base` and `compare` query parameters (line 84) without any path validation. Currently these are TODO stubs that return empty diffs, but when implemented they must use path traversal protection.

**Impact:** Future implementation could introduce path traversal if the `base`/`compare` parameters are passed to `FileResponse` or filesystem operations without validation.

**Fix:** Add `validate_safe_path()` calls on `base` and `compare` when implementing the diff logic. Add a TODO comment referencing the validation requirement.

---

## 03 · Security-Critical Code Highlights

### Secured — API Key Authentication
`contexter-server/src/contexter_server/api/deps.py:38-67` — `get_api_key()` validates Bearer token against `CONtexTER_API_KEY` env var
`contexter-server/src/contexter_server/main.py:100-119` — All 16 v1 routers receive `[Depends(get_api_key)]`
`contexter-server/src/contexter_server/main.py:282-285` — `/health` endpoint intentionally exempt from auth

### Secured — Security Headers
`contexter-server/src/contexter_server/main.py:144-164` — 4 response headers set: `X-Content-Type-Options`, `X-Frame-Options`, `CSP default-src 'self'`, `Referrer-Policy`

### Secured — Docs Gating
`contexter-server/src/contexter_server/main.py:194-204` — `/docs`, `/redoc`, `/openapi.json` return 404 unless `CONtexTER_ENABLE_DOCS=true`

### Secured — Body Size Limiting
`contexter-server/src/contexter_server/main.py:167-191` — Content-Length check against `MAX_REQUEST_BODY` (default 50 MiB), returns 413

### Secured — TrustedHostMiddleware
`contexter-server/src/contexter_server/main.py:277-280` — `TrustedHostMiddleware` with `["127.0.0.1", "localhost"]`

### Secured — Path Traversal Protection
`contexter-server/src/contexter_server/api/files.py:27-63` — `validate_safe_path()` checks raw `..`, encoded `..` (`%2e`, `%2E`), and resolves via `os.path.abspath()`

### Secured — Debug Mode
`contexter-server/src/contexter_server/main.py:267` — `debug=False` explicitly set

### Test Coverage
`contexter-server/tests/api/test_security.py:1-275` — 27 tests covering all 8a-8g security controls

---

## 04 · Remediation Recommendations

> **Must Fix**
> 1. F-01: Redact `api_key` from LLM provider settings response in `settings_service.py:get_section()` (HIGH — active data leak)

> **Should Fix**
> 1. F-02: Add authentication enforcement or document MCP server auth strategy (MEDIUM)
2. F-03: Add rate limiting middleware for API endpoints (MEDIUM)

> **Consider**
> 1. F-04: Use `hmac.compare_digest()` for constant-time API key comparison (LOW)
2. F-05: Add chunked transfer encoding body size enforcement or document edge-level mitigation (LOW)
3. F-06: Reduce default `MAX_REQUEST_BODY` from 50 MiB to 1 MiB (LOW)
4. F-07: Add path traversal validation TODO on file diff `base`/`compare` parameters (LOW)

---

_Generated by Security Architect (Scrutiny) · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase3-python-layer_
