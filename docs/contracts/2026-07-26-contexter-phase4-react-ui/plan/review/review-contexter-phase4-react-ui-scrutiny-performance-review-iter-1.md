# Performance Review Report

# Contexter Phase 4 React UI — Iteration 1 Performance Re-Review

> Auto Bug Loop Iteration 1: Re-validating staleTime=30_000 fix, DataTable key fix, code splitting status, bundle size, and checking for new performance regressions across the contexter-web React application.

**Verdict:** FAIL — 3 Critical Issues Remain Unresolved (class: fail)

2026-07-26 · 6 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Build Size (JS) | `index-BiK_bY2u.js` | 990.93 kB (raw) / 279.79 kB (gzip) |
| Build Size (CSS) | `index-CZkTvO4T.css` | 32.50 kB (raw) / 6.76 kB (gzip) |
| Chunk Warning | Vite | "Some chunks are larger than 500 kB after minification" |
| staleTime (global) | `QueryClient` default in `App.tsx` | 30,000 ms ✅ |
| DataTable key | `item.id ?? rowIdx` | ✅ Correct fallback pattern |
| Code Splitting | `React.lazy()` / `Suspense` | ❌ Not implemented — 0 instances

> **Analysis Scope**
> Full performance audit of contexter-web (React 19, Vite 8, react-router 7, @tanstack/react-query 5, recharts 2, framer-motion 12, lucide-react, date-fns). Examined bundle size via production build, reviewed all 20+ page components, examined all query hooks, checked for lazy loading, recharts static imports, and inline component definitions.

---

## 02 · Benchmark Results

### Bundle Composition

> **Single monolithic chunk** — No code splitting, no manualChunks, no lazy loading.

| Bundle Component | Estimated Size | Notes |
|---|---|---|
| react + react-dom + scheduler | ~130 kB | Core framework — unavoidable |
| react-router (v7.18.1) | ~55 kB | Client-side routing |
| @tanstack/react-query (v5.101.4) | ~35 kB | Data fetching + cache |
| recharts (v2.15.4) | ~40 kB | Loaded on every page, used by only 4 pages |
| framer-motion (v12.42.2) | ~30 kB | Used by Modal, Toast, ToastContainer |
| lucide-react (v0.468.0) | ~25 kB | Tree-shaken icon imports at 41 sites |
| date-fns (v4.1.0) | ~15 kB | Date formatting utilities |
| Application code | ~650 kB | All 20+ page components, layout, hooks, types |

> **Gzip shrinks 990 kB → 280 kB**, so raw size matters less for bandwidth but still affects parse/execute time on low-end devices.

### staleTime Verification

- **App.tsx line 12**: `staleTime: 30_000` globally configured via `QueryClient({ defaultOptions: { queries: { staleTime: 30_000 } } })`
- All 15+ query hooks (useMemories, useSessions, useAnalytics*, useSearch, useNotifications, etc.) inherit this default — none override staleTime
- **Status: ✅ Fixed and effective**

### DataTable Key Fix

- **DataTable.tsx line 170**: `key={(item as Record<string, unknown>)?.id as string | number | undefined ?? rowIdx}`
- Correctly uses `item.id ?? rowIdx` pattern with fallback to row index when id is undefined
- Prevents React key warnings and ensures stable identity for list reconciliation
- **Status: ✅ Fixed and correct**

### Code Splitting Audit

- Searched entire `src/` for `lazy(`, `Suspense`, `React.lazy` — **zero matches**
- All 20+ page components in `routes.tsx` are statically imported at the top of the file
- First paint includes ALL page bundles regardless of which route the user visits
- **Status: ❌ Not implemented**

### Recharts Static Import Analysis

| File | Import | Impact |
|---|---|---|
| `AnalyticsDashboardPage.tsx` | LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer | ~40KB charting lib in main bundle |
| `AnalyticsModelsPage.tsx` | Same imports | Same — but page is rarely visited |
| `AgentDetailPage.tsx` | Same imports | Same — chart only in Efficiency tab |
| `SkillDetailPage.tsx` | Same imports | Same — chart only in Effectiveness tab |

- recharts is ~40KB minified; because there's no code splitting, it's forced on ALL users regardless of which pages they visit
- **Status: ❌ Still unresolved**

---

## 03 · Performance Bottlenecks

### 🔴 Critical: No Route-Level Code Splitting

- **Severity**: High
- **Evidence**: `grep -r 'lazy(' src/` → 0 results. All routes are static imports in `routes.tsx`
- **Impact**: Every page load downloads the full 990 kB bundle, even for a single-page visit. On slow 3G (1.5 Mbps), that's ~1.5s just for download + gzip decompression + JS parse (estimate 300-500ms parse on mid-range mobile)
- **Root cause**: Routes defined with `<DashboardPage />` instead of `lazy(() => import('./pages/Dashboard/DashboardPage'))` in `routes.tsx`

### 🔴 Critical: No manualChunks / Vendor Splitting

- **Severity**: High
- **Evidence**: `vite.config.ts` has no `build.rollupOptions.output.manualChunks` or `codeSplitting` configuration
- **Impact**: Frameworks (react-router, recharts, framer-motion) are bundled into the same monolithic chunk as application code. Browsers can't cache them independently between deployments
- **Root cause**: Missing rollupOptions configuration; a simple `manualChunks` split for `react`, `react-dom`, `recharts`, `framer-motion` would separate stable vendor code from app code

### 🟡 Medium: Recharts Forced Into Main Bundle

- **Severity**: Medium
- **Evidence**: 4 page files import from recharts; with no lazy loading, recharts (~40KB) ships to every route
- **Impact**: Dashboards, Sessions, and Settings pages pay the cost of chart library they never render
- **Root cause**: Same as above — no code splitting

### ✅ Resolved: staleTime (was 0, now 30,000)

- **Before**: Every query re-fetched on mount (network waterfall on page navigation)
- **After**: Queries are fresh for 30 seconds before re-fetching. Verified in `App.tsx` line 12

### ✅ Resolved: DataTable key (was rowIdx only)

- **Before**: `key={rowIdx}` — React re-renders all rows on data change, lost input state
- **After**: `key={item.id ?? rowIdx}` — stable IDs, correct list reconciliation

---

## 04 · Optimization Recommendations

> **High Impact**
> **1. Implement React.lazy() + Suspense for all route-level page components**
   Replace all static imports in `routes.tsx` with `lazy(() => import('./pages/...'))` wrappers. Wrap routes in `<Suspense fallback={<PageSkeleton />}>`. Estimated savings: 400-650 kB from initial bundle, reducing initial JS parse time by ~40-60%.

**2. Configure manualChunks in vite.config.ts**
   Add `build.rollupOptions.output.manualChunks` to split vendor libraries (react, react-dom, recharts, framer-motion, react-router, @tanstack/react-query) into stable, cacheable vendor chunks. Estimated savings: Framework code cached separately, app-only updates ~300 kB smaller.

> **Medium Impact**
> **3. Lazy-load Recharts on chart pages**
   Pages that display charts (AnalyticsDashboard, AnalyticsModels, AgentDetail, SkillDetail) should dynamically import chart components only when the chart tab or section is visible. Use `lazy(() => import('recharts'))` or wrap chart sections in a component that uses dynamic import internally.

**4. Profile and optimize framer-motion usage**
   framer-motion ~30 kB minified. Used in Modal (AnimatePresence + motion.div), Toast (motion.div), ToastContainer (AnimatePresence). Consider lightweight alternatives for simple animations, or ensure unused animation features are tree-shaken.

> **Quick Wins**
> **5. Add chunk size budget to CI**
   Add `build.chunkSizeWarningLimit: 300` (or lower) to vite.config.ts to fail builds that exceed 300 kB per chunk. This prevents bundle regressions going unnoticed.

**6. Consider icon subset for lucide-react**
   41 import sites across the app. While tree-shaking works with ESM, dynamic icon imports or a `<LucideIcon name={...}>` component could further reduce icon payload.

---

_Generated by Performance Benchmarker · 2026-07-26 · Validation Contract: 2026-07-26-contexter-phase4-react-ui_
