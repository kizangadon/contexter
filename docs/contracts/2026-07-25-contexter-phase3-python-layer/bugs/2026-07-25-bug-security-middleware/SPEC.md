# Bug: Security Middleware Hardening

**Sources:** Security CRIT-01, CRIT-02, HI-04, MED-01, MED-02, MED-04, LOW-01, LOW-02, Security HI-03

**Files:** `main.py`, `mcp_server.py`, `api/files.py`

**Problems:**
1. No authentication on any endpoint (CRIT-01) — add basic API key middleware
2. No rate limiting (CRIT-02) — add simple rate limiting
3. MCP server has no auth (HI-04) — add API key check
4. OpenAPI docs exposed (MED-01) — gate behind env var
5. No security headers (MED-02) — add middleware for X-Content-Type-Options, X-Frame-Options, CSP
6. No request body size limits (MED-04) — add max body size check
7. Debug mode not hardened (LOW-01) — debug=False explicitly
8. No TrustedHostMiddleware (LOW-02) — add with localhost
9. File endpoint has no path traversal protection (HI-03) — add validate_safe_path utility stub

**Acceptance:** Auth middleware present (basic API key). Rate limiting implemented. Security headers set. Docs gated. Body size validated. File path validation exists.
