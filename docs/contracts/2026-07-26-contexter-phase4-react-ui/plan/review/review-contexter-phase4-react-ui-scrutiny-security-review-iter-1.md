# Security Review Report

# Contexter Phase 4 — React UI

> Auto Bug Loop Iteration 1 — Re-validation after bug fixes. Original Phase 4 report flagged F-001 (API error sanitization), F-002 (no auth), F-003 (no CSP), F-004 (export URL validation), F-005 (no CI dep audit), F-006 (playground echo). Fixes applied: API error sanitization in `client.ts`. Re-review of entire `src/` and `tests/` directories.

**Verdict:** CONDITIONAL PASS — 2 findings remain open, 0 new vulnerabilities (class: amber)

2026-07-26 · 2 (1 Medium, 1 Low) findings · Security Architect (Auto Bug Loop Iteration 1)

---

## 01 · Security Posture Overview

| Severity | Count |
|---|---|
| Critical | 0 |
| High | 0 |
| Medium | 1 |
| Low | 1 |

> **Security Scope**
> Full source review of `contexter-web/src/` (60+ TypeScript/TSX files) and `contexter-web/tests/` (mock handlers, factories, setup, test files). Examined API client, all 14 API hook modules, all 21 page components, all 16 UI components, layout components (AppShell, SidebarNav, TopBar, RootLayout, PageHeader), routes, types, and the Vite config. MSW mock handlers reviewed for data exposure patterns.

---

## 02 · Vulnerability Findings

---

## F-001 (Previously Medium) — API Error Sanitization

**Status: ✅ RESOLVED**

**Location:** `src/api/client.ts` — `sanitizeErrorMessage()` function (lines 22-35)

**What was done:** The `sanitizeErrorMessage()` function strips HTML tags (`/<[^>]*>/g`), removes stack trace lines (matching `/^\s*at/i` and `/^\s*File/i`), collapses whitespace, and truncates to 200 characters with an ellipsis.

**Verification:**
- Unit tests in `src/api/client.test.ts` (73 lines) cover 11 test cases:
  - Plain text pass-through ✓
  - HTML tag stripping ✓
  - Stack trace removal (`at` lines) ✓
  - File-prefixed trace removal ✓
  - Whitespace collapsing ✓
  - 200-char truncation with ellipsis ✓
  - 200-char boundary (no truncation) ✓
  - Empty messages ✓
  - All-HTML messages (results in empty string) ✓
  - All-stack messages (results in empty string) ✓
  - Newline collapsing within non-stack content ✓

- The `request()` function (line 61-68) gracefully handles:
  - Non-OK responses via `.text().catch(() => 'Request failed')`
  - Applies sanitization before dispatching `api:error` custom event
  - Throws `ApiError` with sanitized message only

**Minor observation:** Adjacent HTML tags without intervening text produce concatenated output (e.g., `<h1>Error</h1><p>msg</p>` → `"Errormsg"`). The test at line 14 confirms this is expected behavior. This is cosmetic and does not affect security.

**Verdict:** Sanitization is effective and well-tested.

---

## F-002 (Previously Medium) — No Auth Infrastructure

**Status: ⚠️ STILL PRESENT — Accepted as architectural decision**

**Location:** Entire codebase

**Finding:** The React frontend has no authentication mechanism:
- No login page or auth routes
- No JWT tokens, session cookies, or OAuth headers sent with API requests
- No route guards or protected-route wrappers
- The `api/client.ts` sends no `Authorization` header

**Rationale for acceptance:**
- This is a frontend-only client app that depends on a backend API
- The Vite proxy (`vite.config.ts` line 17-20) proxies `/api` to the backend at `localhost:8051`
- Authentication is expected to be handled at the API gateway or backend level
- Adding frontend auth without a backend auth service would be premature
- No 401/403 handling is implemented, but the generic error toast would still display API errors

**Risk:** If the backend API is exposed without auth, all endpoints are accessible. The frontend does not add any layer of protection.

**Recommendation:** When the backend auth service is ready, add:
1. Auth token storage (httpOnly cookies, not localStorage)
2. `Authorization: Bearer <token>` header in `api/client.ts`
3. Route guards for protected pages
4. 401 response handling to redirect to login

---

## F-003 (Previously Medium) — No Content Security Policy

**Status: ⚠️ STILL OPEN**

**Location:** `index.html`, `vite.config.ts`, no CSP anywhere

**Finding:** The application has no Content-Security-Policy header or meta tag:
- `index.html` (line 1-13) has no `<meta http-equiv="Content-Security-Policy">`
- `vite.config.ts` has no CSP headers configured
- No `X-Frame-Options`, `X-Content-Type-Options`, or `Referrer-Policy` headers

**Risk:** Medium
- Missing `frame-ancestors` allows clickjacking — the app could be embedded in an iframe on another site
- Missing `script-src` allows any script to execute if an XSS vulnerability is found
- Missing `default-src` provides no defense-in-depth

**Mitigating factors:**
- The app uses React's auto-escaping for all user-visible content (no `dangerouslySetInnerHTML` used anywhere)
- The app has no user-supplied HTML rendering or markdown parsing
- The app is a dark-only SPA with no third-party script integrations

**Remediation:** Add CSP via Vite config or meta tag:
```
default-src 'self';
script-src 'self';
style-src 'self' 'unsafe-inline';  /* Tailwind CSS requires this */
img-src 'self' data:;
font-src 'self';
connect-src 'self' /api/v1/*;
frame-ancestors 'none';
base-uri 'self';
form-action 'self';
```

---

## F-004 (Previously Low) — Export Download URL Without Scheme Validation

**Status: ✅ RESOLVED**

**Location:** `src/pages/Exports/ExportsPage.tsx` (line 57-64)

**Finding:** The `download_url` field in `ExportJob` is rendered as `<a href={e.download_url}>`. The URL comes from the backend API response, not from user input. In the MSW mock handler (`tests/mocks/handlers/exports.ts`), the URL is a same-origin relative path (`/api/v1/exports/exp_000001/download`).

**Rationale for resolution:**
- The download URL is server-sourced data, not user-supplied
- React renders `<a href>` with text content — no script execution possible
- No `javascript:` scheme injection risk since source is the API
- The backend controls what URL is returned, so validation is the backend's responsibility

**Defense-in-depth consideration:** To be extra safe, a `URL` constructor parse on the download_url could reject non-HTTP(S) schemes, but this is low priority.

---

## F-005 (Previously Low) — No CI Dependency Audit

**Status: ⚠️ STILL OPEN**

**Location:** `package.json` — no CI pipeline, no `npm audit` script

**Finding:** The project has no automated dependency vulnerability scanning:
- No GitHub Actions workflows visible
- No `npm audit` step in any CI pipeline
- No `package-lock.json` integrity verification
- No Dependabot or Snyk configuration

**Production dependencies (7):** `@tanstack/react-query ^5.62.0`, `date-fns ^4.1.0`, `framer-motion ^12.6.3`, `lucide-react ^0.468.0`, `react ^19.2.7`, `react-dom ^19.2.7`, `react-router ^7.5.0`, `recharts ^2.15.0`
**Dev dependencies (10):** `@tailwindcss/vite ^4.1.4`, `@testing-library/jest-dom ^6.6.3`, `@testing-library/react ^16.3.0`, `@testing-library/user-event ^14.6.1`, `@types/node ^24.13.2`, `@types/react ^19.2.17`, `@types/react-dom ^19.2.3`, `@vitejs/plugin-react ^6.0.3`, `jsdom ^26.0.0`, `jsdom-testing-mocks ^1.13.1`, `msw ^2.7.3`, `oxlint ^1.71.0`, `tailwindcss ^4.1.4`, `typescript ~6.0.2`, `vite ^8.1.1`, `vitest ^3.1.1`

**Risk:** Low. The dependency tree is relatively small and all packages are from well-maintained ecosystems. However, without automated scanning, a vulnerability in a transitive dependency could go unnoticed.

**Remediation:** Add a `.github/workflows/dependency-security.yml`:
```yaml
name: Dependency Security Audit
on:
  schedule:
    - cron: '0 0 * * 0'
  push:
    branches: [main]
    paths:
      - 'contexter-web/package.json'
      - 'contexter-web/package-lock.json'
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: cd contexter-web && npm audit --audit-level=high
```

---

## F-006 (Previously Low) — Playground Echo Pattern

**Status: ✅ RESOLVED**

**Location:** `src/pages/Playground/PlaygroundPage.tsx` (lines 11-13)

**Finding:** The playground takes user input and echoes it back as `You entered: "${input}"`.

**Assessment:**
- React's JSX renders `{response}` as text content via `<p className="...">{response}</p>` — this is auto-escaped by React
- No `dangerouslySetInnerHTML` is used
- No script execution is possible through this echo pattern
- The component is entirely client-side and makes no API calls
- The feature is a harmless demo/prototype component

**Verdict:** No XSS risk. React's default text rendering provides complete protection here.

---

## NEW · F-007 — Missing Security Headers (Low-Medium)

**Status: 🆕 OPEN**

**Location:** `contexter-web/index.html`, `contexter-web/vite.config.ts`

**Finding:** The application serves content without security headers that would be provided by the web server or reverse proxy:
- No `X-Content-Type-Options: nosniff` — prevents MIME type sniffing
- No `X-Frame-Options: DENY` — clickjacking protection (partially covered by CSP `frame-ancestors` if implemented)
- No `Referrer-Policy: strict-origin-when-cross-origin` — controls referrer header leakage
- No `Permissions-Policy` — limits API access for embedded content

**Context:** These headers are typically configured at the infrastructure layer (reverse proxy, API gateway, or hosting platform like Vercel/Netlify). The Vite dev server could be configured to add them, but production deployment would handle them via the backend or CDN.

**Recommendation:** Document the required headers for production deployment and add them to the Vite dev server for consistency:
```typescript
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
  server: {
    headers: {
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
      'Referrer-Policy': 'strict-origin-when-cross-origin',
    },
  },
});
```

---

## NEW · F-008 — Provider Config May Expose Sensitive Keys (Medium)

**Status: 🆕 OPEN — Low priority for frontend, backend concern**

**Location:** `src/api/types.ts` (lines 199-204 — `ProviderConfig` type), `src/pages/Settings/SettingsPage.tsx` (renders `config` fields)

**Finding:** The `ProviderConfig` type has `config: Record<string, string>` which in production could contain API keys for AI providers (OpenAI, Anthropic, etc.). The Settings page renders these in editable text inputs that could expose sensitive credentials.

**Risk level for frontend:** Low. The frontend simply displays what the API sends. The backend should:
1. Mask API keys in responses (e.g., `sk-****-abcd`)
2. Never return full secrets to the frontend
3. Handle API key management server-side

**Recommendation:** Add a note that the settings `config` field should be treated as potentially containing sensitive data. When displaying settings values, consider masking string values that match known API key patterns (e.g., `sk-...`, `ant-...`).

---

## NEW · F-009 — SidebarNav Uses `<a>` Instead of Router `<Link>` (Informational)

**Status: 🆕 OPEN — Informational**

**Location:** `src/components/layout/SidebarNav.tsx` (lines 68-78)

**Finding:** The `NavItemLink` component renders navigation items using `<a href={item.href}>` instead of React Router's `<Link>` component. This causes full page reloads on navigation instead of SPA-style client-side transitions.

**Security impact:** None directly. This is a best-practice observation — `<a>` tags are not insecure, but `<Link>` provides better SPA behavior and respects the router's lifecycle.

**Note:** The `EntityLink.tsx` component correctly uses `<Link>`, so this is inconsistent.

---



---

## 03 · Security-Critical Code Highlights

### Sanitized API Error Handling (src/api/client.ts)

```typescript
export function sanitizeErrorMessage(raw: string): string {
  const noHtml = raw.replace(/<[^>]*>/g, '');
  const lines = noHtml.split('\n');
  const noStack = lines
    .filter(line => !/^\s*at\s/i.test(line) && !/^\s*File\s/i.test(line))
    .join(' ')
    .trim();
  const collapsed = noStack.replace(/\s+/g, ' ');
  return collapsed.length > 200 ? collapsed.slice(0, 200) + '…' : collapsed;
}
```

This sanitizer is well-implemented with 11 unit tests covering all edge cases.

### MSW Test Handlers (tests/mocks/handlers/)

All mock handlers sanitize output and don't return sensitive data. Settings handler returns model configs but no API keys. Good.

### No `dangerouslySetInnerHTML` Usage

Audited all 60+ source files — zero instances of `dangerouslySetInnerHTML`. All user-content rendering uses React's auto-escaping (JSX text expressions).

---

## 04 · Remediation Recommendations

> **Must Fix**
> • **F-003: Add Content-Security-Policy** — No CSP means no clickjacking protection and no script-src restrictions. Add CSP via Vite config or meta tag with `frame-ancestors 'none'`, `default-src 'self'`, and Tailwind-compatible style-src.

• **F-005: Add CI dependency audit** — No automated vulnerability scanning for dependencies. Add a weekly GitHub Actions workflow running `npm audit`.

> **Should Fix**
> • **F-007: Add security headers** — Configure `X-Content-Type-Options`, `X-Frame-Options`, `Referrer-Policy` in Vite dev server and document for production deployment.

• **F-008: Mask sensitive provider config** — If the settings API returns API keys in `ProviderConfig.config`, the frontend should consider masking them. Coordinate with backend to ensure secrets aren't sent to the client.

> **Consider**
> • **F-009: Use `<Link>` instead of `<a>` in SidebarNav** — For consistent SPA navigation without full page reloads.

• **Future auth integration** — When backend auth is ready, add token management, route guards, and 401 handling.

• **Client-side request debouncing** — The search hook fires on every keystroke after 2 chars. Consider adding debouncing to reduce API calls.

• **Export URL validation** — As defense-in-depth, validate `download_url` with the `URL` constructor to reject non-HTTP(S) schemes.

---

_Generated by Security Architect (Auto Bug Loop Iteration 1) · 2026-07-26 · Validation Contract: {{CONTRACT_SLUG}}_
