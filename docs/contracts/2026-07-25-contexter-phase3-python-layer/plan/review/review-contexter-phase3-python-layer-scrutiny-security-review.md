# Security Review Report

# Contexter Phase 3 — Python API Layer

> Security review of the Python FastAPI server layer for the Contexter agent memory system, covering 16 API route modules, Pydantic models, service layer, MCP server, CLI, and Rust bridge integration. Phase 3 implements the full web API with CRUD endpoints for sessions, memories, agents, skills, search, analytics, settings, audit, notifications, export, and file operations.

**Verdict:** FAIL (class: REQUIRES_ACTION)

2026-07-25 · 16 (2 Critical, 4 High, 5 Medium, 4 Low, 1 Informational) findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 2 |
| High | 4 |
| Medium | 5 |
| Low | 4 |

> **Security Scope**
> Full-stack Python API layer including FastAPI application (main.py), 16 route modules in api/, 11 Pydantic model files in models/, 12 service modules in services/, MCP SSE server (mcp_server.py + mcp_tools/), StorageEngine bridge to Rust (core/bridge.py), Click CLI (cli/). No authentication module (planned for later phase). Data flow: HTTP/FastMCP -> Pydantic validation -> Service layer -> Bridge (JSON/PyBytes) -> Rust Engine. All endpoints run without authentication; rate limiting is absent.

---

## 02 · Vulnerability Findings


### CRIT-01: No Authentication/Authorization — Full API Access Without Restrictions

**Severity:** Critical
**Location:** All `api/v1/*` endpoints — `main.py:90-107`, all router files
**Evidence:** No `Depends()` auth dependency exists anywhere. No authentication middleware. No token validation. The `deps.py` file only provides service injection, not auth.
**Impact:** Any process that can reach the API port (default: localhost:8051) can read, create, update, and delete all data — sessions, memories, agents, skills, settings, and audit logs. If exposed beyond localhost (e.g., via reverse proxy, Docker networking, or configuration change), an unauthenticated attacker has full read/write access to the entire system.
**Fix:** Implement authentication middleware (e.g., API key via `Authorization` header) and apply it as a router-level dependency. Or at minimum, bind to localhost-only (currently done in `RESTConfig.host` default) and document the security boundary.
**Mitigation:** Verify the app is only accessible from localhost in production. The `RESTConfig.host` default is `127.0.0.1` (`models/settings.py:41`), which limits exposure.

---

### CRIT-02: No Rate Limiting on Any Endpoint — Unbounded Brute Force / DoS Vector

**Severity:** Critical
**Location:** All endpoints — no rate limiting middleware or per-endpoint throttling
**Evidence:** `pyproject.toml` dependencies list `fastapi`, `uvicorn`, `pydantic`, `structlog`, `pyyaml`, `click`, `httpx` — none include `slowapi` or any rate-limiting library. `main.py` has no rate-limiting middleware. No custom rate-limiting decorators exist.
**Impact:** Attacker can flood any endpoint (e.g., `POST /api/v1/feedback/bug` which takes `data: dict` with zero validation) without limit. Export operations in `ExportService.submit()` enumerate all entities via the bridge, creating a CPU/memory amplification vector.
**Fix:** Add `slowapi` dependency, create rate limiting middleware, and apply strict limits (e.g., 100 req/min per IP) to all endpoints, with stricter limits on mutation endpoints.
**Mitigation:** If deployed behind a reverse proxy (nginx, Cloudflare), apply rate limiting at the edge.

---

### HI-01: Unvalidated `dict` Input on State-Changing Endpoints — Mass Assignment / Arbitrary Data Injection

**Severity:** High
**Location:**
- `api/feedback.py:9` — `async def report_bug(data: dict)`
- `api/feedback.py:16` — `async def suggest_feature(data: dict)`
- `api/settings.py:30` — `async def update_section(section: str, data: dict)`
- `api/onboarding.py:22` — `async def submit_wizard(data: dict)`
- `api/files.py:31` — `async def watch_files(data: dict)`

**Evidence:** These endpoints accept `data: dict` as the request body with zero Pydantic validation. Any arbitrary JSON payload is accepted and (in the case of settings and onboarding) directly passed to the service layer. The settings endpoint writes user-controlled data to the YAML config file at `~/.contexter/config.yaml` after minimal property existence checks.
**Impact:** Attackers can inject arbitrary configuration values into settings (e.g., modifying `LLMProviderConfig.api_key`, storage paths, server host/port bindings). The feedback endpoints accept unlimited arbitrary data, enabling mass injection or DoS via oversized payloads.
**Fix:** Create Pydantic models for all request bodies. For feedback, define `BugReport` and `FeatureSuggestion` models. For settings, use the existing `Settings` model or dedicated section-update models. For onboarding, define a `WizardData` model.
**Mitigation:** At minimum, validate against a schema before processing.

---

### HI-02: Config File Stores LLM API Keys in Plaintext YAML

**Severity:** High
**Location:** `models/settings.py:47-49` — `LLMProviderConfig.api_key: Optional[str] = None`
`services/settings_service.py:104-109` — `_write_yaml()` dumps to `~/.contexter/config.yaml`

**Evidence:** The `LLMProviderConfig` model has an `api_key` field that, when set, is serialized to a plaintext YAML file. The config file path defaults to `~/.contexter/config.yaml`, and `yaml.dump()` is used without encryption.
**Impact:** Any process with filesystem access to `~/.contexter/config.yaml` can read LLM API keys (OpenAI, Anthropic, etc.) in plaintext. If the config file permissions are too permissive (default umask), other local users could steal credentials. If the app is containerized and config is persisted unboundedly, container escape exposes credentials.
**Fix:** Implement secret encryption at rest (e.g., encrypt `api_key` fields using Fernet or a keyring). Or store secrets separately in an encrypted keychain, with only non-sensitive config in YAML.
**Mitigation:** Restrict filesystem permissions on `~/.contexter/` directory (`chmod 700`). Document that users should set restrictive permissions.

---

### HI-03: No Path Traversal Protection on File Endpoints

**Severity:** High
**Location:** `api/files.py:10` — `path: str = Query(".", description="Directory path to list")`
**Evidence:** The `GET /api/v1/files` endpoint accepts a user-controlled `path` query parameter with no sanitization, allowlisting, or path traversal checks. While currently a TODO stub, the `path` value is echoed back to the client in the response, establishing a pattern that will likely be used when real file listing is implemented. No `Path` validation (`Path(...)`) or directory traversal checks exist.
**Impact:** When file listing is implemented, an attacker could read arbitrary files outside the intended directory (e.g., `/etc/passwd`, `/app/.env`) by passing `../` sequences. Currently, the endpoint is a stub, but the lack of guardrails means future implementation will likely be vulnerable.
**Fix:** Before implementing real file listing, add path sanitization: use `os.path.abspath()`, verify the resolved path is within an allowed base directory, reject paths with `..` components, and use an allowlist of permitted directories.
**Mitigation:** Pre-implement a `validate_safe_path()` utility before the TODO is filled in.

---

### HI-04: MCP Server (SSE/WebSocket) Has No Authentication or Origin Validation

**Severity:** High
**Location:** `mcp_server.py:64-67` — FastMCP instantiation, lines 73-163 (8 tools), lines 169-198 (4 resources)
**Evidence:** The FastMCP server runs on port 8052 with SSE transport. It registers 8 tools (`store_memory`, `search_memories`, `get_session`, `list_recent_sessions`, `get_agent_info`, `list_skills`, `get_system_health`, `export_data`) and 4 resources — all with no authentication, no API key check, and no origin header validation. Tools like `store_memory` and `export_data` can mutate/read all data.
**Impact:** Any process or client that can connect to port 8052 (default: localhost) can read, create, and export all data via MCP tools without any authentication. If the port is exposed (Docker networking, misconfiguration), external attackers have full access.
**Fix:** Implement API key authentication for MCP tools, validate origin headers for SSE connections, and add per-tool authorization checks. At minimum, bind MCP to localhost-only.
**Mitigation:** Verify `MCPServerConfig.host` is `127.0.0.1` (already set in `models/settings.py:33`). Never expose port 8052 publicly.

---

### MED-01: OpenAPI Documentation Exposed Without Restriction

**Severity:** Medium
**Location:** `main.py:187-191` — `app = FastAPI(title="Contexter API", version="0.1.0", lifespan=lifespan)`
**Evidence:** No `docs_url=None`, `redoc_url=None`, or `openapi_url=None` is set. FastAPI defaults enable `/docs`, `/redoc`, and `/openapi.json` endpoints. Using defaults means the full API schema with all routes, parameters, models, and data structures is publicly discoverable.
**Impact:** Information disclosure — an attacker can enumerate all API endpoints, understand request/response schemas, and build targeted attacks without reverse-engineering the client.
**Fix:** Set `docs_url=None`, `redoc_url=None`, `openapi_url=None` in production. Or gate access behind an environment variable.
**Mitigation:** Add `docs_url=None` in `create_app()` now. Enable docs during development via config flag.

---

### MED-02: No Security Headers (CSP, X-Content-Type-Options, HSTS)

**Severity:** Medium
**Location:** `main.py:110-129` — Logging middleware, no header-setting middleware exists
**Evidence:** The `_add_logging_middleware()` function only adds request logging. No middleware sets `X-Content-Type-Options: nosniff`, `X-Frame-Options`, `Content-Security-Policy`, or `Referrer-Policy` headers. No `CORSMiddleware` is registered.
**Impact:** Although the API primarily returns JSON (not HTML), missing security headers can enable XSS in browsers that render API responses as HTML, clickjacking if the API is embedded in iframes, and MIME-type sniffing attacks.
**Fix:** Create a middleware that sets security headers on every response. At minimum: `X-Content-Type-Options: nosniff`, `X-Frame-Options: DENY`, `Content-Security-Policy: default-src 'self'`.
**Mitigation:** If behind a reverse proxy, set these headers at the edge (nginx, Cloudflare).

---

### MED-03: Settings Service Updates Accept Arbitrary Keys via Unvalidated `dict`

**Severity:** Medium
**Location:** `services/settings_service.py:87-102` — `update_section(section, data: dict)`
**Evidence:** The `update_section` method iterates over `data.items()` and calls `setattr(section_obj, key, value)` for each key. While it checks `hasattr(section_obj, key)`, this still allows an attacker to set ANY attribute on the config section model. There's no validation of value types, ranges, or allowable values.
**Impact:** An attacker who can reach `PUT /api/v1/settings/{section}` can modify configuration values in ways the API designer did not intend. For example, setting `mcp_server.port` to a different port, or `cache.ttl_secs` to 0, or `storage.path` to an attacker-controlled path.
**Fix:** Create explicit update models per section (`ProjectUpdate`, `CacheUpdate`, etc.) with validated fields, and use Pydantic to parse before applying changes.

---

### MED-04: No Request Body Size Limits

**Severity:** Medium
**Location:** App-wide — no `max_request_size` middleware or limit
**Evidence:** FastAPI/Starlette defaults accept request bodies up to unlimited size by default. There is no `max_body_size` middleware, no `request_size` limit, and no multipart upload size constraint. The `memory` create/update endpoints accept large `content` text fields.
**Impact:** An attacker can send multi-gigabyte JSON payloads to exhaust server memory, causing denial of service. The bridge's `_LARGE_CONTENT_THRESHOLD = 102400` (100KB) in `core/bridge.py:19` indicates large memory content is expected, but there's no upper bound.
**Fix:** Set `max_body_size` on the ASGI server (Uvicorn `--limit-max-request-body`) and/or add middleware to reject oversized requests.
**Mitigation:** Configure nginx/AWS ALB `client_max_body_size` at the edge.

---

### MED-05: Memory Export Stores Complete Data in Memory Without Bound

**Severity:** Medium
**Location:** `services/export_service.py:15,57-59` — `self._exports: dict[str, ExportStatus] = {}` and storing `{id}_data`
**Evidence:** The `ExportService.submit()` method calls `list_sessions({})`, `search_memories({})`, `list_agents({})`, `list_skills({})` ALL at once, loading ALL entities into memory. The results are stored in the `_exports` dict with no eviction, no size limit, and no pagination. Each export creates a new full-data entry.
**Impact:** With enough export requests, the in-memory `_exports` dict grows unboundedly, causing memory exhaustion DoS. A single export of a large data set can also exhaust memory.
**Fix:** Implement in-memory cache eviction (e.g., LRU cache with max N entries), add `max_export_size` limits, stream export output to disk instead of holding in memory, and/or paginate entity queries.

---

### LOW-01: Debug Mode and Reload Not Hardened

**Severity:** Low
**Location:** `main.py:187-191` — `FastAPI()` without explicit `debug=False`
**Evidence:** The `FastAPI` constructor does not pass `debug=False`. However, the default is `False` so this is not immediately exploitable. The app runs via Uvicorn (no reload mode visible in code). This is noted as a defense-in-depth improvement.
**Fix:** Explicitly set `FastAPI(..., debug=False)` to make the production posture clear.

---

### LOW-02: No Host Header Validation (TrustedHostMiddleware)

**Severity:** Low
**Location:** `main.py` — no `TrustedHostMiddleware` registered
**Evidence:** TrustedHostMiddleware from Starlette is not configured. If the app generates URLs dynamically from the `Host` header (future feature), it would be vulnerable to host header injection.
**Fix:** Add `TrustedHostMiddleware` with an allowlist of trusted hosts.
**Mitigation:** Verify reverse proxy strips untrusted `Host` headers.

---

### LOW-03: CLI Commands Re-raise Raw Exception Messages

**Severity:** Low
**Location:** All CLI command handlers — `except Exception as e: raise click.ClickException(str(e))`
**Evidence:** Internal error messages (database paths, engine errors) are leaked to CLI stdout via `str(e)`. Examples: `session_commands.py:69-70`, `memory_commands.py:99-100`, `export_commands.py:86-87`.
**Impact:** Minimal — CLI is a local admin tool. But exception messages could leak internal paths, storage engine details, or data that aids an attacker with local access.
**Fix:** Log the full exception with `logger.exception()`, return a generic "operation failed" message to the CLI user.

---

### LOW-04: Export History Returns All Statuses Without Limit Enforcement on Internal Data

**Severity:** Low
**Location:** `services/export_service.py:78-85` — `history()`
**Evidence:** The `limit` parameter defaults to 20 but the method sorts all entries in memory. If the `_exports` dict grows large (see MED-05), this becomes a performance concern.
**Fix:** Truncate at the dict retrieval level, not after sorting all items.

---

### INF-01: httpx Dependency Listed but Not Used in API Layer

**Severity:** Informational
**Location:** `pyproject.toml:19` — `httpx>=0.28`
**Evidence:** The `httpx` library is a project dependency but no `httpx` imports exist in any of the reviewed source files. This represents an unnecessary dependency that increases the attack surface.
**Impact:** Minimal — unused dependencies are not an active vulnerability but increase maintenance burden.
**Fix:** Remove `httpx` from dependencies until it is actually needed, or move it to an optional dependency group.


---

## 03 · Security-Critical Code Highlights


### Critical Attack Surface Map

```
Internet/Frontend
    │
    ├── HTTP :8051 (api/v1/*  —  16 routers, ~50 endpoints)
    │     │
    │     ├── NO AUTH              ← CRIT-01
    │     ├── NO RATE LIMITING     ← CRIT-02
    │     ├── NO SECURITY HEADERS  ← MED-02
    │     ├── NO BODY SIZE LIMITS  ← MED-04
    │     │
    │     ├── api/feedback.py      → data: dict (unvalidated)     ← HI-01
    │     ├── api/settings.py      → data: dict (unvalidated)     ← HI-01/HI-02
    │     ├── api/onboarding.py    → data: dict (unvalidated)     ← HI-01
    │     ├── api/files.py         → path: str (no sanitization)  ← HI-03
    │     │
    │     └── All other endpoints  → Pydantic validated ✅
    │
    ├── SSE/MCP :8052 (8 tools, 4 resources)
    │     │
    │     ├── NO AUTH              ← HI-04
    │     ├── NO ORIGIN CHECK      ← HI-04
    │     ├── store_memory()       → Full write access
    │     └── export_data()        → Full read access
    │
    ├── /docs, /redoc, /openapi.json  ← MED-01
    │
    └── Config: ~/.contexter/config.yaml
          └── LLM API keys in plaintext  ← HI-02
```

**Threat Model:** An attacker who can reach port 8051 or 8052 has FULL read/write access to all system data. The current security posture relies entirely on network-level isolation (binding to localhost). If that boundary is crossed (Docker networking, Kubernetes pod-to-pod, reverse proxy misconfiguration, or intentional exposure), there are zero application-level defenses.


---

## 04 · Remediation Recommendations

> **Must Fix**
> 
1. **CRIT-01** — Implement authentication (API key or JWT) as a router-level dependency on all protected endpoints. Bind-only-to-localhost is insufficient alone.
2. **CRIT-02** — Add rate limiting middleware (slowapi or custom) to all endpoints, with strict limits on mutation endpoints (POST/PUT/DELETE).
3. **HI-01** — Replace all `data: dict` signatures with proper Pydantic request models for feedback, settings, onboarding, and files endpoints.
4. **HI-02** — Encrypt LLM API keys at rest in the config YAML, or store them in a separate encrypted keychain file.
5. **HI-03** — Implement path traversal protection (`validate_safe_path()`) before the file listing endpoint is implemented for real.
6. **HI-04** — Add authentication and origin validation to the MCP server. Ensure all 8 tools require authorization.


> **Should Fix**
> 
1. **MED-01** — Disable OpenAPI docs in production (`docs_url=None`, `redoc_url=None`). Enable via env var for dev.
2. **MED-02** — Add security headers middleware (X-Content-Type-Options, X-Frame-Options, CSP).
3. **MED-03** — Create typed Pydantic update models per config section instead of accepting raw dicts.
4. **MED-04** — Set request body size limits (Uvicorn `--limit-max-request-body` and/or application middleware).
5. **MED-05** — Implement LRU eviction and size limits on in-memory export storage.


> **Consider**
> 
1. **LOW-01** — Set `FastAPI(debug=False)` explicitly.
2. **LOW-02** — Add `TrustedHostMiddleware` for host header validation.
3. **LOW-03** — Log full exception details server-side, return only generic messages in CLI output.
4. **LOW-04** — Bound the export history dict size.
5. **INF-01** — Remove unused `httpx` dependency or move to optional.
6. **Documentation** — Add security docs to README explaining the current trust model (no auth, localhost-only) and deployment requirements (reverse proxy, authentication proxy like oauth2-proxy).


---

_Generated by Security Architect · 2026-07-25 · Validation Contract: 2026-07-25-contexter-phase3-python-layer_
