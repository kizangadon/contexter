# Handoff Report: CSP + SidebarNav <a>→<Link>

## Summary

Two surgical fixes applied to `contexter-web/`.

## Fix 1: Content-Security-Policy

### Files changed
- **`index.html`** — Added `<meta http-equiv="Content-Security-Policy">` with:
  ```
  default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';
  img-src 'self' data:; font-src 'self'; frame-ancestors 'none'; base-uri 'self'
  ```

- **`vite.config.ts`** — Added `cspPlugin()` function that injects the same CSP via Vite's `transformIndexHtml` hook. Registered in plugins array.

### How it works
| Environment | CSP source |
|---|---|
| Production (static) | `<meta>` tag in `index.html` — copied verbatim to `dist/index.html` |
| Dev (Vite server) | `cspPlugin().transformIndexHtml` injects `<meta>` into served HTML |

## Fix 2: SidebarNav `<a>` → `<Link>`

### Files changed
- **`src/components/layout/SidebarNav.tsx`**
  - Added `import { Link, useLocation } from 'react-router'`
  - Replaced `<a href={item.href}>` with `<Link to={item.href!}>` (line 70-80)
  - Added `useLocation()` call to derive `isActive` from pathname as fallback alongside `activeItemId` prop
  - No `<a>` tags remain in the file

- **`src/components/layout/SidebarNav.test.tsx`**
  - Added `import { MemoryRouter } from 'react-router'`
  - Wrapped `renderSidebarNav()` helper in `<MemoryRouter>` (required by `<Link>`)

- **`src/components/layout/AppShell.test.tsx`**
  - Extended the `vi.mock('react-router', ...)` mock to include stubs for `Link`, `useLocation`, `MemoryRouter`, `useNavigate`, and `useParams` (needed because `AppShell` renders `SidebarNav` which now uses these)

## Verification

### Tests
```
Test Files  62 passed (62)
     Tests  437 passed (437)
```
All 62 test files, 437 tests pass.

### Build
```
npx vite build → ✓ built in 371ms
```
Vite build succeeds. The `npm run build` script (`tsc -b && vite build`) fails on a pre-existing `rolldownOptions` TypeScript type issue unrelated to these changes.

### CSP verification
Built `dist/index.html` contains the CSP `<meta>` tag with correct policy.

### No `<a>` tags
`grep '<a ' SidebarNav.tsx` returns no matches — all anchor tags replaced with `<Link>`.

## Files summary
| # | File | Change |
|---|---|---|
| 1 | `index.html` | Added CSP `<meta>` tag |
| 2 | `vite.config.ts` | Added `cspPlugin()` for dev CSP injection |
| 3 | `SidebarNav.tsx` | `<a>` → `<Link>`, added `useLocation` |
| 4 | `SidebarNav.test.tsx` | Wrapped in `<MemoryRouter>` |
| 5 | `AppShell.test.tsx` | Extended `react-router` mock with new exports |
