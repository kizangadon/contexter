# Performance Review Report

# Contexter Phase 4 — React UI (Final Performance Validation)

> Final performance review of the Contexter Web UI — 530 tests passing, 76 test files, 30+ lazy-loaded route pages across 164 source files. Validates build output, chunk sizes, initial load cost, code splitting, and bundle composition.

**Verdict:** MEETS REQUIREMENTS with recommendations (class: amber)

2026-07-26 · 12 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| **Production Build Time** | 337 ms |
| **Total Output Size** | ~1.12 MB raw / ~328 kB gzip |
| **Initial Load (Critical Path)** | ~891 kB raw / ~260 kB gzip |
| **CSS Bundle** | 32.83 kB raw / 6.82 kB gzip |
| **App Shell (index.js)** | 27.71 kB raw / 7.96 kB gzip |
| **Route-Level Chunks** | 30+ chunks, 0.3–20.2 kB each (minified) |
| **Total Routes** | 30+ (all lazy-loaded via React.lazy) |
| **Vendor Chunks** | 6 segregated (react, query, fm, charts, icons, runtime) |
| **Test Count** | 530 tests · 76 files · 100% pass |
| **Build Status** | CLEAN — 0 errors, 0 lint warnings, 1 chunk size warning |
| **TypeScript** | Strict mode, clean compile |
| **Chunk Size Warning Limit** | 300 kB (exceeded by vendor-charts at 386 kB)

> **Analysis Scope**
> Performance analysis of the Contexter Web React UI on branch `feature/contexter-phase4-react-ui`. Analysis includes: production build output, bundle composition, chunk size distribution, code splitting effectiveness, initial load critical path, vendor dependency cost, test suite performance, and CSS bundle analysis.

---

## 02 · Benchmark Results

### Build Benchmark

| Metric | Value |
|---|---|
| Build tool | Vite 8.1.5 + rolldown |
| Build time | 337 ms |
| Modules transformed | 2,983 |
| Output chunks | 60 JS files + 1 CSS |
| Total dist size | 1,121,730 bytes raw |

### Chunk Size Distribution

```
Rank  Chunk                         Raw (kB)  Gzip (kB)  Category
────  ────────────────────────────  ────────  ─────────  ───────────────
 1    vendor-charts                  386.1     102.5      recharts (vendor)
 2    vendor-react                   276.4      87.9      React+Router (vendor)
 3    vendor-fm                      125.3      40.9      framer-motion (vendor)
 4    index                           27.7       8.0      App shell (entry)
 5    vendor-query                    29.3       9.0      @tanstack/react-query
 6    index.css                       32.8       6.8      Tailwind CSS
 7    vendor-icons                    13.4       5.0      lucide-react (vendor)
 8    MemoryDetailPage                20.2       5.5      Largest page chunk
 9    formatDistanceToNow              9.7       3.3      date-fns utility
10    EfficiencyPage                   9.2       2.9      Dashboard sub-page
11    SessionDetailPage                8.8       2.7      Session sub-page
12    FeedbackPage                     8.8       2.1      Standalone page
13    CorrelationPage                  7.7       1.6      Standalone page
14    SettingsPage                     7.5       2.2      Settings (8 sections)
15    AgentDetailPage                  7.5       2.3      Agent detail
16    AnalyticsDashboardPage           7.5       2.3      Analytics overview
17    (remaining 44 chunks)           ~220      ~60      All <7 kB each
```

### Code Splitting Analysis

**Route-Level Splitting: ✅ Excellent**
- All 30+ page components use `React.lazy()` with dynamic `import()`
- Each page chunk is independently loadable
- Typical page chunk: 1–8 kB (minified)
- Largest page chunk: MemoryDetailPage at 20.2 kB (due to inline sub-components)

**Vendor Chunk Segregation: ✅ Good**
- `manualChunks` splits framework code into 6 vendor buckets via `build.rolldownOptions.output.manualChunks`
- Each vendor bucket is independently cacheable (content-hashed filenames)
- Split: react, query, framer-motion, recharts, lucide-icons, runtime

**Modulepreload Overfetching: ⚠️ Suboptimal**
- `index.html` emits `modulepreload` link tags for ALL vendor chunks
- This means every page load fetches recharts (386 kB) + framer-motion (125 kB) regardless of whether the route uses charts or animations
- framer-motion is only used in 3 components (Toast, Modal, ToastContainer) that are universal UI — these are part of the app shell, so some cost is unavoidable
- recharts is only used in 8 of 30+ pages — the modulepreload forces it on all pages

### Test Performance

| Metric | Value |
|---|---|
| Test files | 76 passed |
| Individual tests | 530 passed (0 failures) |
| Total duration | 16.25 s |
| Slowest test file | routes.test.tsx — 14.3 s (35 route-resolution tests) |
| Fastest test | client.test.ts — 14 ms (11 API client tests) |
| Recharts stderr warnings | 8 pages emit width/height=0 chart warnings in jsdom |

---

## 03 · Performance Bottlenecks

### 🔴 Critical

**1. vendor-charts (recharts) modulepreloaded on all pages**
- **Cost**: 386 kB raw / 102 kB gzip — largest single output file
- **Impact**: Every route pays the cost of loading recharts upfront, even pages that don't use it (e.g., Dashboard, Settings, Search, Notifications, Playground, NotFound, etc.)
- **Root cause**: `index.html` includes `<link rel="modulepreload">` for `vendor-charts` — Vite's default behavior includes all statically imported vendor chunks in the initial modulepreload set
- **Mitigation feasibility**: High. Vite supports `modulePreload` configuration to exclude specific chunks, or the `manualChunks` function can be enhanced to defer recharts into a conditionally-loaded chunk only when a page that imports it is navigated to

### 🟡 High

**2. vendor-fm (framer-motion) preloaded on all pages**
- **Cost**: 125 kB raw / 41 kB gzip
- **Impact**: Added to every page's initial load
- **Usage**: Only 3 source files import framer-motion (`Toast.tsx`, `Modal.tsx`, `ToastContainer.tsx`) — all universal UI components that are part of the app shell. The cost is partially justified since these are always-present components, but the animation library could potentially be replaced with CSS transitions for toast/enter animations
- **Mitigation**: Consider CSS-only animations for Toast/Modal, or defer framer-motion chunk to only load when animated components mount

**3. Cumulative initial load ~260 kB gzip**
- **Cost**: ~891 kB raw / ~260 kB gzip transferred on first paint
- **Context**: For a dashboard/monitoring SPA, this is at the upper end of acceptable. Typical performance budgets target <200 kB gzip for initial load
- **Breakdown**: vendor-charts (102 kB gzip) + vendor-react (88 kB) + vendor-fm (41 kB) = 231 kB of the 260 kB total

### 🟢 Medium

**4. CSS bundle at 32.8 kB raw / 6.8 kB gzip**
- Tailwind CSS v4 output is well within acceptable thresholds
- No unused CSS purging concerns identified

**5. MemoryDetailPage (20.2 kB) largest route chunk**
- Contains 361 lines including 3 inline sub-components (ContentTab, VersionsTab, RelatedTab)
- Imports date-fns (format, formatDistanceToNow) and 7 UI components
- Consider extracting sub-components into separate files for cleaner code-splitting boundaries

**6. Recharts chart test warnings in 8 files**
- `stderr` output: "The width(0) and height(0) of chart should be greater than 0"
- Root cause: jsdom renders charts at 0×0 — ResponsiveContainer/recharts requires explicit container dimensions
- **Impact**: Low (cosmetic, doesn't affect test pass/fail), but indicates chart components lack proper responsive container setup in tests

### Test Performance Observations

**7. routes.test.tsx is slowest at 14.3 seconds**
- 35 route resolution tests, each doing a full lazy-load + render cycle
- Each test awaits a lazy-loaded page component, contributing to cumulative time
- Could be optimized by testing route resolution without full render, but acceptable for current scale

---

## 04 · Optimization Recommendations

> **High Impact**
> **1. Remove recharts from modulepreload set**
- **Effort**: Low (config change in `vite.config.ts`)
- **Gain**: 386 kB raw / 102 kB gzip removed from initial load on pages not using charts
- **Implementation**: Add `modulePreload: { resolveDependencies: (url, deps) => deps.filter(d => !d.includes('vendor-charts')) }` or restructure `manualChunks` to keep recharts outside the initial modulepreload set
- **Savings**: ~40% of initial load gzip weight eliminated from non-chart pages
- **Pages benefiting**: Dashboard, Settings, Search, Playground, Notifications, Feedback, Exports, Onboarding, NotFound, Session pages, Memory pages, Skill Registry (17 of 30+ routes)

**2. Evaluate framer-motion replacement with CSS transitions**
- **Effort**: Medium (refactor Toast, Modal, ToastContainer)
- **Gain**: 125 kB raw / 41 kB gzip removed from initial load
- **Toast animations**: Spring-based slide-in (Toast.tsx lines 36-40) can be replaced with CSS `@keyframes` and Tailwind `animate-*` utilities
- **Modal overlay**: AnimatePresence + motion.div can be replaced with CSS transitions
- **Savings**: ~16% of initial load gzip

**3. Add performance budgets to CI**
- **Effort**: Low
- **Gain**: Prevents regressions automatically
- **Implementation**: Add `bundlesize` or `size-limit` config to fail CI when vendor-charts exceeds 400 kB or initial load exceeds 300 kB gzip

> **Medium Impact**
> **4. Extract sub-components from MemoryDetailPage**
- **Effort**: Low (extract ContentTab, VersionsTab, RelatedTab to separate files)
- **Gain**: Smaller initial code-split boundary for the memory detail route

**5. Fix Recharts container dimensions in tests**
- **Effort**: Low (add container with explicit dimensions to test render wrappers)
- **Gain**: Clean test output, no stderr warnings

**6. Add web-vitals RUM monitoring**
- **Effort**: Low (install `web-vitals` package, add tracking to App.tsx)
- **Gain**: Real user performance data to validate synthetic measurements

> **Quick Wins**
> **7. Enable Vite's `build.reportCompressedSize`**
- Already enabled (gzip sizes are reported in build output)

**8. Add `modulePreload` filter in vite.config.ts**
- Single config change with outsized impact (per #1 above)

**9. Add Lighthouse CI to pipeline**
- Catch performance regressions before they reach production

---

_Generated by Performance Benchmarker · 2026-07-26 · Validation Contract: 2026-07-26-contexter-phase4-react-ui_
