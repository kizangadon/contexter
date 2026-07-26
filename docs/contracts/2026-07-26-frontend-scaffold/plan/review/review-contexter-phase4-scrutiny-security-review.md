# Security Review Report

# Contexter Phase 4 — React UI

> Security audit of the Contexter Phase 4 frontend application — covering the API client, API hooks, pages, components, build configuration, and dependency hygiene.

**Verdict:** PASS WITH OBSERVATIONS (class: amber)

2026-07-26 · 6 findings · Security Architect

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 3 |
| Low | 3 |

> **Security Scope**
> Reviewed all source files in `contexter-web/` — API client (`src/api/client.ts`), 12 API hook modules, 19 page components, 8 layout/UI components, routing (`src/routes.tsx`), Vite config, entry HTML, and dependency manifest (`package.json`). Focus areas: credential leakage, XSS/DOM injection, URL injection, client-side auth patterns, CSP posture, and dependency vulnerabilities.

---

## 02 · Vulnerability Findings

### F-001: Error messages from the API may leak sensitive server-side information

**Rule ID:** REACT-NET-001 / JS-CSP-001  
**Severity:** Medium  
**Location:** `src/api/client.ts:43`; `src/pages/Sessions/SessionDetailPage.tsx:178-179`; `src/pages/Agents/AgentDetailPage.tsx:302-303`  
**Evidence:**

```typescript
// src/api/client.ts:42-44
if (!response.ok) {
  const message = await response.text().catch(() => 'Request failed');
  throw new ApiError(response.status, message);
}

// src/pages/Sessions/SessionDetailPage.tsx:178-179
message={
  error instanceof Error
    ? error.message
    : 'The requested session could not be found.'
}

// src/pages/Agents/AgentDetailPage.tsx:302-303
message={
  error instanceof Error
    ? error.message
    : 'The requested agent could not be found.'
}
```

**Impact:** If the API server returns verbose error pages (HTML with stack traces, SQL errors, internal paths) in its response body, the `ApiError.message` will contain that full text. Pages such as `SessionDetailPage` and `AgentDetailPage` then render `error.message` directly into the UI via `<p>` elements. This could leak internal implementation details to authenticated users.

**Fix:**  
In the API client, sanitize the error message to extract only a safe summary:

```typescript
if (!response.ok) {
  const text = await response.text().catch(() => '');
  // Only keep the first line if it looks like a plain-text message
  // Reject HTML responses that may contain stack traces
  const isHtml = text.startsWith('<');
  const message = isHtml
    ? `Request failed with status ${response.status}`
    : text.split('\n')[0].slice(0, 200) || `Request failed (${response.status})`;
  throw new ApiError(response.status, message);
}
```

**Mitigation:** Configure the API server (port 8051) to return structured JSON error responses (e.g., `{ "error": "...", "code": "..." }`) instead of HTML or verbose text for non-2xx responses.

---

### F-002: No authentication or authorization infrastructure present in the client

**Rule ID:** REACT-AUTHZ-001 / REACT-AUTH-001  
**Severity:** Medium  
**Location:** `src/api/client.ts`, all `src/api/hooks/*.ts` files, `src/routes.tsx`  
**Evidence:**

The API client makes all requests without any authentication headers, tokens, or cookies:

```typescript
// src/api/client.ts:36-40
const response = await fetch(url.toString(), {
  ...rest,
  headers,
  body: body !== undefined ? JSON.stringify(body) : undefined,
});
```

No `Authorization` header is set. No `credentials: 'include'` is used. No token storage, refresh mechanism, or auth provider exists. All routes in `src/routes.tsx` are publicly accessible with no client-side guards.

**Impact:** If the backend API requires authentication (which is likely for exposing session, memory, and settings data), the frontend currently has no mechanism to authenticate. Additionally, when auth is added, the lack of existing patterns increases the risk of insecure token storage (e.g., `localStorage`).

**Fix:**  
Implement authentication infrastructure before connecting to a production API. Create a centralized auth module:

```typescript
// src/api/auth.ts
let accessToken: string | null = null;

export function setToken(token: string) {
  accessToken = token;
}

export function clearToken() {
  accessToken = null;
}

export function getAuthHeaders(): Record<string, string> {
  return accessToken ? { Authorization: `Bearer ${accessToken}` } : {};
}
```

Wire into the API client:

```typescript
// In request() function, after headers setup:
const authHeaders = getAuthHeaders();
for (const [key, value] of Object.entries(authHeaders)) {
  headers.set(key, value);
}
```

**Mitigation:** Prefer short-lived tokens stored in-memory (not `localStorage`) with HTTP-only cookie-based session tokens from the server. See REACT-AUTH-001 guidance.

---

### F-003: No Content Security Policy (CSP) configured anywhere in the application

**Rule ID:** REACT-CSP-001 / JS-CSP-001  
**Severity:** Medium  
**Location:** `index.html:1-13`, `vite.config.ts:1-23`  
**Evidence:**

```html
<!-- index.html — no CSP meta tag or headers -->
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Contexter</title>
  </head>
  ...
</html>
```

No CSP configuration exists in `vite.config.ts`, no headers are configured at the server/CDN level visible in the repository, and no `<meta http-equiv="Content-Security-Policy">` tag exists.

**Impact:** Without CSP, the application has no defense-in-depth against XSS attacks. If an XSS vulnerability were introduced (e.g., via a future markdown renderer, rich text field, or third-party script), there would be no browser-level barrier to script execution.

**Fix:**  
Add a strict CSP either via the server/CDN or via a `<meta>` tag in `index.html`. Since this is a Vite SPA, a `<meta>` tag approach works:

```html
<meta http-equiv="Content-Security-Policy" content="
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  font-src 'self';
  connect-src 'self' http://localhost:8051;
  frame-ancestors 'none';
  form-action 'self';
  base-uri 'self';
">
```

**Note:** `style-src 'unsafe-inline'` is required for Tailwind/Vite CSS injection during both dev and production (Vite inlines styles). This is a documented CSP trade-off.

---

### F-004: Download URL from API rendered directly in `<a href>` without scheme validation

**Rule ID:** REACT-URL-001 / JS-URL-002  
**Severity:** Low  
**Location:** `src/pages/Exports/ExportsPage.tsx:57-64`  
**Evidence:**

```typescript
// src/pages/Exports/ExportsPage.tsx:57-64
render: (e) =>
  e.status === 'completed' && e.download_url ? (
    <a
      href={e.download_url}
      className="inline-flex items-center gap-1 text-sm font-medium text-accent hover:text-accent-hover"
    >
      <Download className="h-4 w-4" />
      Download
    </a>
  ) : null,
```

**Impact:** The `download_url` comes from an API response (`ExportJob.download_url`). If the API were compromised or returned a URL with a `javascript:` or `data:` scheme, clicking the link could execute attacker-controlled code in the application origin. While this is an API-supplied value (not direct user input), it crosses a trust boundary and should be validated.

**Fix:**  
Add a URL scheme validator:

```typescript
function isSafeDownloadUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === 'https:' || parsed.protocol === 'http:';
  } catch {
    return false;
  }
}

// In the render function:
href={isSafeDownloadUrl(e.download_url) ? e.download_url : '#'}
```

---

### F-005: Dependency lockfile and audit practices not enforceable in CI

**Rule ID:** REACT-SUPPLY-001 / JS-SUPPLY-001  
**Severity:** Low  
**Location:** `package.json`, CI configuration (not present)  
**Evidence:**

```json
// package.json — dependencies use `^` semver ranges
"dependencies": {
  "@tanstack/react-query": "^5.62.0",
  "framer-motion": "^12.6.3",
  "react": "^19.2.7",
  "react-dom": "^19.2.7",
  "react-router": "^7.5.0",
  "recharts": "^2.15.0"
}
```

**Impact:** While `^` ranges are standard practice, without CI enforcement of `npm ci` and `npm audit`, dependency drift and known-vulnerability regressions can go undetected. The project has no visible CI configuration to enforce lockfile integrity or run vulnerability scans.

**Fix:**  
Add a CI step (e.g., GitHub Actions) that enforces:

```yaml
- run: npm ci          # fails if lockfile is out of sync
- run: npm audit       # fails on critical/high vulnerabilities
```

**Mitigation:** At minimum, verify `package-lock.json` exists and is committed. Run `npm audit` locally before major changes.

---

### F-006: Playground page echoes user input back in response without server-side validation

**Rule ID:** REACT-XSS-002  
**Severity:** Low  
**Location:** `src/pages/Playground/PlaygroundPage.tsx:12`  
**Evidence:**

```typescript
const handleSubmit = () => {
  if (!input.trim()) return;
  setResponse(`You entered: "${input}"`);
  setInput('');
};
```

The response is rendered via React JSX (`<p className="text-sm text-text-primary">{response}</p>`), which means React's auto-escaping protects against XSS here. This is **not** a vulnerability in the current implementation.

**Observations:**  
- If this pattern evolves into direct HTML rendering or is used as a pattern for future "echo" features, it could become a risk
- The response text is purely client-side state — it never reaches the server or other users, so stored XSS is impossible
- Consider marking this as a learning point for future development: always prefer structured data rendering over string interpolation, even for "echo" patterns

**No fix required** for the current implementation. This is an informational observation.

---

## 03 · Security-Critical Code Highlights

### Areas of Strength (No Issues Found)

- **No `dangerouslySetInnerHTML` usage** anywhere in the codebase — excellent adherence to React security best practices
- **No DOM XSS sinks** — zero uses of `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, or `eval`
- **No localStorage/sessionStorage for tokens or secrets** — the codebase doesn't store any sensitive data in Web Storage
- **No hardcoded secrets, API keys, or credentials** — no sensitive values in source code
- **All user content rendered through JSX** — `MessageBubble`, `MemoryDetail`, `SearchPage`, and all other components use React's built-in escaping (`{value}` in JSX), which prevents HTML injection by default
- **No open redirect patterns** — no `window.location` manipulation with user-supplied values
- **No postMessage usage** — eliminates cross-window communication attack surface
- **No iframe/embed/object tags** — reduces clickjacking and plugin-based attack surface
- **No service worker registration** — eliminates SW-based attack surface
- **Single-origin API client** — `BASE_URL` is static (`/api/v1`), preventing SSRF via dynamic URL construction

### Areas Requiring Attention

- **API error handling** (F-001): the client passes raw server response text to consumers, which could leak sensitive internal details
- **Authentication gap** (F-002): no auth infrastructure exists in the client, which is a blocker for production deployment against a real API
- **Missing CSP** (F-003): defense-in-depth against future XSS vulnerabilities is absent
- **Download URL validation** (F-004): one instance of API-supplied URL used directly in an anchor `href` without scheme validation

---

## 04 · Remediation Recommendations

> **Must Fix**

1. **F-001** — Sanitize API error messages in the client to prevent leakage of server internals. Modify `request()` in `client.ts` to strip HTML and truncate error text. (Medium)

> **Should Fix**

2. **F-002** — Design and implement an authentication layer (token management, auth headers, route guards) before deploying against a production API. Use in-memory token storage with HTTP-only cookie fallback from the server. (Medium)
3. **F-003** — Deploy a Content Security Policy via server headers or `<meta>` tag. Start with report-only mode to test compatibility, then enforce. See the fix in F-003 for a recommended policy. (Medium)

> **Consider**

4. **F-004** — Add URL scheme validation (`https:`/`http:` only) for the `download_url` in ExportsPage before passing it to `<a href>`. (Low)
5. **F-005** — Add CI pipeline with `npm ci` and `npm audit` enforcement. Verify `package-lock.json` is committed. (Low)
6. **F-006** — No action required; this is an informational observation about the PlaygroundPage echo pattern. (Informational)

---

_Generated by Security Architect · 2026-07-26 · Validation Contract: 2026-07-26-frontend-scaffold_
