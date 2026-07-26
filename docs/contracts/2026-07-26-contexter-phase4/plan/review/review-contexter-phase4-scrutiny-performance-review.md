# Performance Review Report

# Contexter Phase 4 — React UI Performance Audit

> Comprehensive performance benchmark analysis of the Contexter web frontend covering bundle size, React Query caching, DataTable rendering, Recharts usage, code-splitting, and rendering patterns across 16 pages and 20+ components.

**Verdict:** FAIL (class: critical)

2026-07-26 · 12 benchmarks · Performance Benchmarker

---

## 01 · Performance Overview

| Metric | Value |
|---|---|
| Bundle size (JS) | 190.53 kB total / **60.00 kB gzipped** |
| Bundle size (CSS) | 31.36 kB total / **6.61 kB gzipped** |
| Total build output | 252 kB (incl. index.html) |
| Route-level code splitting | **NONE** — single monolithic chunk |
| React Query staleTime | **0 ms (default)** — all 30+ queries refetch on every mount |
| `QueryClientProvider` | **MISSING** — app will crash at runtime |
| `React.memo` usage | **ZERO** — no components memoized |
| `useMemo` for derived data | 2/7 pages (MemoryExplorer, SessionManager) |
| `useCallback` for handlers | Inconsistent — 4/10 pages use it |
| Lazy-loaded routes | **NONE** — no `React.lazy()` or dynamic imports |
| Recharts import | Static — bundled into main chunk (~40 kB estimate) |
| framer-motion usage | Single use — toast animations only (~30 kB gzip estimate) |

> **Analysis Scope**
> Full walkthrough of `/home/don/Code/contexter/contexter-web/` — `src/pages/` (16 pages), `src/components/` (20+ components), `src/api/` (hooks, client, types), build output analysis, and dependency impact assessment.

---

## 02 · Benchmark Results

### Benchmark 1: Production Bundle Size

| Asset | Raw Size | Gzipped | % of Total |
|---|---|---|---|
| `index-DsIztlrY.js` | 190.53 kB | 60.00 kB | 85.7% |
| `index-wDJfwvdI.css` | 31.36 kB | 6.61 kB | 14.1% |
| `index.html` | 0.45 kB | 0.29 kB | 0.2% |
| **Total** | **222.34 kB** | **66.90 kB** | **100%** |

**Verdict:** Bundle is acceptable (190 kB raw, 60 kB gzipped) but contains all dependencies (recharts, framer-motion, lucide-react, date-fns, @tanstack/react-query) in a single chunk. Without code-splitting, adding more pages will linearly increase initial load cost.

**Dependency size on disk (node_modules):**

| Dependency | Disk Size | Used For | Can be lazy-loaded? |
|---|---|---|---|
| `lucide-react` | 37 MB | Icons throughout app | Tree-shaken already ✓ |
| `date-fns` | 28 MB | Date formatting (5+ pages) | Tree-shaken already ✓ |
| `framer-motion` | 5.9 MB | Toast enter/exit animations | Heavy for toast-only use |
| `recharts` | 5.4 MB | Charts on 2 analytics pages | **YES** — not lazy-loaded ✗ |
| `@tanstack/react-query` | 1.9 MB | Data fetching (all pages) | Core dependency ✓ |

---

### Benchmark 2: React Query Cache Configuration

All 30+ `useQuery` hooks across the codebase were audited:

| Setting | Current Value | Recommended Value | Status |
|---|---|---|---|
| `staleTime` | **0 ms** (default) | 30s–5m (per query tier) | **CRITICAL** |
| `gcTime` | 5 min (default) | 5–30 min (per data tier) | Acceptable default |
| `refetchOnMount` | true (default) | `false` or `'stale'` for non-critical data | Unoptimized |
| `refetchOnWindowFocus` | true (default) | `false` for dashboard/analytics | Unoptimized |
| `retry` | 3 (default) | 1–2 for data queries | Acceptable |
| `QueryClientProvider` | **NOT PRESENT** in App.tsx | Must wrap `<App />` | **CRITICAL** |

**Hooks inspected:** `useSessions`, `useSession`, `useMemories`, `useMemory`, `useMemorySearch`, `useMemoryVersions`, `useAgents`, `useAgent`, `useSkills`, `useSkill`, `useEfficiencyOverview`, `useEfficiencyMemory`, `useEfficiencySessions`, `useEfficiencyAgents`, `useEfficiencySkills`, `useEfficiencyTokens`, `useEfficiencyCorrelation`, `useAnalyticsOverview`, `useAnalyticsHealth`, `useAnalyticsPerformance`, `useAnalyticsResources`, `useAnalyticsCosts`, `useAnalyticsModelDetail`, `useAnalyticsServices`, `useSettings`, `useNotifications`, `useUnreadCount`, `useSearch`, `useExports`, `useAudit`, `useCorrelationOverview`, `useCorrelationTimeline`, `useCorrelationCompare`, `useOnboardingStatus`, `useChangelog`, `useSubmitBugReport`, `useSubmitSuggestion` (+ their mutations).

**Key observation:** No hook specifies `staleTime`. Every query refetches on mount, even for data that changes infrequently (e.g., `useAnalyticsHealth`, `useSettings`, `useSkills`). This means:
- Navigate from Dashboard → Analytics → Dashboard → all 3 Dashboard queries refetch
- Navigate to Settings → all settings refetch from network
- Window refocus → all active queries refetch

---

### Benchmark 3: DataTable Rendering Analysis

File: `src/components/ui/DataTable.tsx`

| Aspect | Current | Impact | Recommendation |
|---|---|---|---|
| Row key | `rowIdx` (array index) | **High** — causes DOM reconciliation issues on data mutation | Use `item.id` or stable unique key |
| `React.memo` | Not wrapped | **Medium** — unnecessary re-renders on parent state change | Wrap component in `React.memo` |
| `pageData` computation | `data.slice()` every render | **Low** — O(n) but creates new array ref | Wrap in `useMemo` with `[data, pageSize, currentPage]` |
| Sort state | Local `useState` | **Low** — resets on unmount | Acceptable pattern |
| Inline arrow functions in JSX | `onClick` handlers, render props | **Medium** — new function refs each render | Use `useCallback` or stable refs |
| Pagination controls | Previous/Next with disabled states | **Low** — acceptable | Good a11y with disabled states |

**Key issue — index keys:** When data is filtered or reordered (e.g., client-side sort in MemoryExplorer), React cannot reconcile rows efficiently because `item[0]` becomes a different entity after reorder. This causes unnecessary DOM updates.

---

### Benchmark 4: Recharts Usage

Files: `src/pages/Analytics/AnalyticsDashboardPage.tsx`, `src/pages/Analytics/AnalyticsModelsPage.tsx`

| Aspect | Current | Impact |
|---|---|---|
| Import method | Static top-level `import` | **High** — adds ~40 kB to main bundle |
| `ResponsiveContainer` | Used correctly with `width="100%" height="100%"` | ✅ Good |
| Chart wrapper | `<div className="h-64 w-full">` container | ✅ Good |
| Tooltip styling | Inline `contentStyle` object | **Low** — creates new object each render |
| `React.memo` on chart sections | Not used | **Low** — charts re-render on any parent state change |
| Data memoization | `performanceData` used directly, not memoized | **Low-Medium** — depends on upstream caching |

**Impact:** Recharts is the single largest render-cost library in the app. It's imported statically on 2 pages but adds to the initial bundle load for ALL users, even those who never visit Analytics.

---

### Benchmark 5: Component Rendering Patterns

| Page/Component | Pattern | Issue | Severity |
|---|---|---|---|
| `DashboardPage` | `trends` object recreated every render | No `useMemo` | Medium |
| `DashboardPage` | `statusVariant` defined at module level | ✅ Good — stable ref | — |
| `DashboardPage` | `sessionColumns` defined at module level | ✅ Good — stable ref | — |
| `MemoryExplorerPage` | `sortedData` via `useMemo` | ✅ Good pattern | — |
| `MemoryExplorerPage` | `columns` defined outside component | ✅ Good — not recreated | — |
| `SessionManagerPage` | `sortedSessions` via `useMemo` | ✅ Good pattern | — |
| `SessionManagerPage` | `handleSort`, `handleRowClick` via `useCallback` | ✅ Good | — |
| `CorrelationPage` | Array index as key (`idx`) for topCorrs and timeline | **Unstable keys** | High |
| `CorrelationPage` | `handleRetry` via `useCallback` | ✅ Good | — |
| `EfficiencyPage` | `computeTrend` function recreated every render | **Not in useCallback** — but not passed as prop | Low (not passed down) |
| `EfficiencyPage` | `skillsColumns` at module level | ✅ Good | — |
| `SettingsPage` | `useEffect` depends on `data` object | **Reference instability** — re-syncs on every render if data is new object | Medium |
| `SettingsPage` | `handleFieldChange` via `useCallback` | ✅ Good | — |
| `FeedbackPage` | `ChangeBadge` sub-component defined inside `ChangelogPanel` | **Recreated every render** | Medium |
| `NotificationsPage` | `handleMarkRead`, `handleMarkAllRead` via `useCallback` | ✅ Good | — |
| `AppShell` | `ShellLayout` inner component | **Recreated every render** (defined inside AppShell) | Medium |
| `TimeframeFilter` | Uses `useId` | ✅ Good — accessible | — |
| `LoadingSkeleton` | Module-level `keyframesInjected` state | Side effect pattern (works but non-standard) | Low |

---

### Benchmark 6: Page Load / Data Fetching Waterfall (Simulated)

Without code-splitting, a user visiting the Dashboard triggers this sequence:

```
1. HTML load (0.45 kB) ✓
2. CSS parse (31 kB) ✓
3. JS parse & execute (190 kB) — includes ALL pages' code
4. React mount + render
5. useSessions → GET /api/v1/sessions  (staleTime=0 → always fetches)
6. useMemories → GET /api/v1/memories  (staleTime=0 → always fetches)
7. useEfficiencyOverview → GET /api/v1/efficiency/overview (staleTime=0)
8. All 3 queries complete → render stat cards + table + quick actions
```

**Issue:** Steps 5–7 make concurrent requests with no caching. If the user navigates to Sessions and back, all 3 fire again.

---

### Benchmark 7: Bundle Composition Estimate

Based on import analysis, estimated composition of the 190 kB JS bundle:

| Source | Estimated Size | Share |
|---|---|---|
| React + ReactDOM (19.x) | ~42 kB gzip | 22% |
| recharts | ~40 kB | 21% |
| framer-motion | ~28 kB | 15% |
| react-router | ~18 kB | 9% |
| @tanstack/react-query | ~14 kB | 7% |
| lucide-react (tree-shaken icons) | ~12 kB | 6% |
| date-fns (tree-shaken functions) | ~8 kB | 4% |
| Application code (pages + components) | ~28 kB | 15% |

**Key insight:** ~36% of bundle (recharts + framer-motion) is used on only 3 of 16 pages.

---

## 03 · Performance Bottlenecks

### 🔴 CRITICAL: Missing QueryClientProvider

**Location:** `src/App.tsx` + all `src/api/hooks/*.ts`

**Description:** The `App` component renders only `<div><p>Contexter</p></div>`. There is no `<QueryClientProvider>` wrapping the component tree. All `useQuery`, `useMutation`, and `useQueryClient` calls depend on React Query context. At runtime, every page using data hooks will throw: `No QueryClient set, use QueryClientProvider to set one`. Tests work because each wraps in their own `QueryClientProvider`.

**Evidence:** `grep` shows 37+ files importing from `@tanstack/react-query`. Zero references in `App.tsx` or `main.tsx`. `main.tsx` renders `<App />` directly with no provider.

**Impact:** Application is non-functional. All data-fetching pages crash on mount.

---

### 🔴 CRITICAL: No staleTime on Any Query

**Location:** All `src/api/hooks/*.ts` — 30+ `useQuery` calls

**Description:** Every `useQuery` uses the default `staleTime: 0`. This means:
- Every component mount triggers a network fetch
- Window refocus triggers fetches on all visible queries
- Dashboard users see a loading spinner on every navigation back
- Backend receives redundant requests for data that changed minutes ago

**Examples of queries that should not refetch on mount:**
- `useSettings` — settings rarely change
- `useAnalyticsHealth` — health status is polled, not stale
- `useSkills` — skill registry is relatively static
- `useChangelog` — changelog is append-only

---

### 🔴 HIGH: No Route-Level Code Splitting

**Location:** All page imports (would require routing setup first)

**Description:** The entire application — all 16 pages, including recharts (40 kB) and framer-motion (28 kB) — is bundled into a single JS chunk. A user on the Playground page downloads the full Analytics dashboard code including charting libraries.

**Impact:** Unnecessary bandwidth and parse cost. Recharts alone adds ~40 kB of JS that users on Playground, Search, or Settings never use. Without `React.lazy()` and `Suspense`, bundle grows linearly with every new page.

---

### 🔴 HIGH: DataTable Uses Array Index as Row Key

**Location:** `src/components/ui/DataTable.tsx:168`

```tsx
{pageData.map((item, rowIdx) => (
  <tr key={rowIdx} ...
```

**Description:** Using the array index as React key breaks reconciliation when data changes. When data is sorted, filtered, or updated, React cannot identify which rows changed. This causes unnecessary DOM updates and can lead to incorrect component state for any child components.

**Impact:** Visible on MemoryExplorerPage (client-side sort) and SessionManagerPage (client-side sort). Sorting columns causes all rows to unmount and remount instead of efficient reordering.

---

### 🔴 HIGH: Recharts Should Be Lazy-Loaded

**Location:** `src/pages/Analytics/AnalyticsDashboardPage.tsx:3-11`, `src/pages/Analytics/AnalyticsModelsPage.tsx:3-11`

```tsx
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
```

**Description:** Recharts is imported statically at the top of both analytics page files. Since there is no code-splitting at the route level, recharts is bundled in the main entry chunk. Even users who never visit Analytics pay the ~40 kB cost.

**Fix:** Route-level code splitting via `React.lazy()` would naturally move recharts to a separate chunk loaded only when `/analytics` or `/analytics/models` is visited.

---

### 🟡 MEDIUM: CorrelationPage Uses Array Index as Key

**Location:** `src/pages/Correlation/CorrelationPage.tsx:81-83`, `:137-139`

```tsx
{topCorrs.map((corr, idx) => (
  <StatCard key={idx} ...
{timelineData.map((point, idx) => (
  <div key={idx} ...
```

**Description:** Array index keys for correlation items and timeline entries. While these lists may not reorder in practice, using index keys prevents efficient reconciliation and can cause rendering bugs if the data changes.

---

### 🟡 MEDIUM: SettingsPage useEffect Reference Instability

**Location:** `src/pages/Settings/SettingsPage.tsx:175-180`

```tsx
useEffect(() => {
  if (data?.settings) {
    setEditedValues({ ...data.settings });
    setHasChanges(false);
  }
}, [data]);
```

**Description:** The `useEffect` depends on the `data` object reference. If React Query returns a new object reference on every data-fetch (even with identical values), this effect runs on every render, resetting edited values and clearing the "unsaved changes" state. This is a bug triggered by default `refetchOnMount` + `staleTime: 0`.

---

### 🟡 MEDIUM: Sub-Components Defined Inside Render

**Location:** `src/pages/Feedback/FeedbackPage.tsx` (`ChangeBadge` inside `ChangelogPanel`), `src/components/layout/AppShell.tsx` (`ShellLayout` inside `AppShell`)

**Description:** Defining components inside other components causes them to be recreated on every render. React sees them as new types each time and unmounts/remounts the entire subtree. This defeats React's reconciliation.

---

### 🟡 MEDIUM: Toast Animation Library Overhead

**Location:** `src/components/ui/Toast.tsx`, `src/components/ui/ToastContainer.tsx`

**Description:** `framer-motion` (~5.9 MB on disk, ~28 kB in bundle) is used only for toast enter/exit animations and `AnimatePresence`. This is a heavy dependency for simple slide-in/out animations that could be done with CSS transitions + keyframe animations.

---

### 🟢 LOW: LoadingSkeleton Module-Level Side Effect

**Location:** `src/components/ui/LoadingSkeleton.tsx:32-40`

**Description:** Injects a `<style>` tag into `document.head` via module-level side effect on first render. While this works, it's a non-standard pattern. Better approach: define `@keyframes skeleton-pulse` in the CSS tokens file or use Tailwind's built-in animation utilities.

---

## 04 · Optimization Recommendations

> **High Impact**

1. **Add `QueryClientProvider` to App.tsx (Blocker)** — Wrap `<App />` with `<QueryClientProvider client={queryClient}>`. Create a `queryClient` instance with sensible defaults outside the component. This is a runtime blocker — without it, the application does not function.

2. **Set tiered `staleTime` on all queries** — Group queries by data volatility:
   - **Static data** (skills, settings, changelog): `staleTime: 5 * 60 * 1000` (5 minutes)
   - **Semi-static data** (sessions list, agent list, memories): `staleTime: 30 * 1000` (30 seconds)
   - **Dynamic data** (analytics health, notifications unread): `staleTime: 10 * 1000` (10 seconds)
   - **Real-time** (efficiency overview, active sessions): `staleTime: 0` (always fresh)
   - Global defaults: `refetchOnWindowFocus: false`, `refetchOnMount: 'stale'` (only if stale)

3. **Implement route-level code splitting** — Once routing is set up, wrap each page import in `React.lazy()`:
   ```tsx
   const AnalyticsDashboardPage = React.lazy(() => import('./pages/Analytics/AnalyticsDashboardPage'));
   const PlaygroundPage = React.lazy(() => import('./pages/Playground/PlaygroundPage'));
   ```
   Wrap routes in `<Suspense fallback={<PageSkeleton />}>`. This naturally moves recharts, framer-motion, date-fns chunks into page-specific bundles.

4. **Fix DataTable row keys** — Change `key={rowIdx}` to `key={item.id}`. Requires items to have a unique `id` property. If T is generic, make the column definition or a dedicated `rowKey` prop accept `(item: T) => string | number`.

> **Medium Impact**

5. **Wrap DataTable in `React.memo`** — Prevents unnecessary re-renders when parent state changes (e.g., filter state in MemoryExplorerPage). With page-level state changes, DataTable re-renders even if its `data` prop hasn't changed.

6. **Memoize `pageData` in DataTable** — Wrap the slice computation in `useMemo`:
   ```tsx
   const pageData = useMemo(
     () => isLoading ? [] : data.slice(startIndex, endIndex),
     [data, startIndex, endIndex, isLoading],
   );
   ```

7. **Replace framer-motion with CSS animations** — Toast styles can be handled with Tailwind CSS and CSS `@keyframes`. This eliminates ~28 kB from the bundle for a feature used with 3 animation states. Example:
   ```css
   @keyframes slide-in-right {
     from { opacity: 0; transform: translateX(80px); }
     to { opacity: 1; transform: translateX(0); }
   }
   ```

8. **Fix CorrelationPage keys** — Replace `idx` with stable identifiers. For `topCorrs` use a composite key `${corr.variable_1}-${corr.variable_2}`. For timeline entries use `point.date`.

9. **Extract sub-components from render** — Move `ShellLayout` out of `AppShell`, move `ChangeBadge` to module level in FeedbackPage. Prevents unnecessary unmount/remount cycles.

10. **Stabilize SettingsPage effect** — Compare settings contents rather than object reference. Use deep comparison or a hash of serialized settings:
    ```tsx
    const settingsJson = JSON.stringify(data?.settings);
    useEffect(() => {
      if (data?.settings) { ... }
    }, [settingsJson]);
    ```

> **Quick Wins**

11. **Set global React Query defaults** — In `main.tsx` or a new `api/queryClient.ts`:
    ```tsx
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          staleTime: 30 * 1000,
          refetchOnWindowFocus: false,
          retry: 1,
        },
      },
    });
    ```
    Single change, immediate impact on all 30+ queries.

12. **Reduce LoadingSkeleton keyframe injection** — Move the `@keyframes skeleton-pulse` definition into `tokens.css` or use Tailwind's `animate-pulse`. Eliminates the module-level side-effect pattern.

13. **Use `gap-lg` consistently** — App uses Tailwind design tokens (`gap-lg` = 24px). Confirmed consistent across all pages. ✅

14. **Add `displayName` to components** — Not a perf optimization but helps DevTools profiling identify components in React Profiler.

---

_Generated by Performance Benchmarker · 2026-07-26 · Validation Contract: contexter-phase4-react-ui_
