# Contexter Web

> React dashboard for the Contexter context management system. Dark-only UI with 30+ routes, 530+ tests, and a bounded-context domain architecture.

---

## Stack

| Tool | Version | Purpose |
|------|---------|---------|
| React | 19 | UI library (StrictMode, lazy routes) |
| TypeScript | ~6.0.2 | Strict mode, full type coverage |
| Vite | 8 | Build tool, HMR dev server |
| Tailwind CSS | 4 | Utility-first CSS via `@tailwindcss/vite` |
| React Router | 7 | File-adjacent route definitions |
| TanStack Query | 5 | Server state (50+ hooks, staleTime=30s, retry=1) |
| Framer Motion | 12 | Animated page/component transitions |
| Recharts | 2 | Analytics, efficiency, and cost charts |
| Lucide React | 0.468 | Icon library |
| Vitest | 3 | Test runner (jsdom, globals) |
| Testing Library | 16 | Component and hook tests |
| MSW | 2 | API mock server (15 handler modules) |
| Oxlint | 1 | Linting |
| date-fns | 4 | Date formatting |

---

## Design System

**V2-DEEP** — dark-only theme. No light mode.

| Token | Value |
|-------|-------|
| Base background | `#181716` |
| Surface | `#1e1d1c` |
| Accent | `#7C5CFC` |
| Accent hover | `#6b4fe0` |
| Text primary | `#f5f4f2` |
| Text secondary | `#a09e9c` |
| Border | `#2e2d2c` |

Semantic colors: `#22c55e` (success), `#f59e0b` (warning), `#ef4444` (error), `#3b82f6` (info).

Font stack: Inter (UI), JetBrains Mono (code). Border radii: 6–16px.

Defined in `src/styles/tokens.css` — the single source of truth consumed by `@theme` in Tailwind v4.

---

## Architecture

### Layout

The UI uses an **AppShell** pattern (`src/components/layout/AppShell.tsx`):

```
┌──────────┬────────────────────────────────┐
│          │  TopBar (breadcrumbs, notif.)   │
│ Sidebar  ├────────────────────────────────┤
│ 240px    │                                │
│ (coll.   │  Main content (max 1440px)     │
│  60px)   │                                │
│          │                                │
└──────────┴────────────────────────────────┘
```

- Sidebar collapses to 60px with animated transition (300ms)
- Content area has max-width 1440px, horizontal centering
- `RootLayout` wraps the router with `QueryClientProvider` + `ToastProvider`
- All pages lazy-loaded via `React.lazy()`

### Domain Architecture (DDD Bounded Contexts)

17 page directories map to domain contexts, each with typed API hooks and MSW handlers:

| Context | Routes | Hooks | Description |
|---------|--------|-------|-------------|
| Dashboard | 1 | — | System overview, summary stats |
| Sessions | 2 | `useSessions` | Session CRUD, detail with turns |
| Memories | 2 | `useMemories` | Memory explorer, version history |
| Agents | 2 | `useAgents` | Agent registry, detail with efficiency |
| Skills | 2 | `useSkills` | Skill registry, detail with effectiveness |
| Efficiency | 7 | `useEfficiency` | Memory/session/agent/skill/token/correlation views |
| Analytics | 7 | `useAnalytics` | Health, performance, resources, costs, models, services |
| Settings | 2 | `useSettings` | 8-section settings with provider config |
| Search | 1 | `useSearch` | Cross-entity full-text search |
| Playground | 1 | — | API exploration (REST + MCP + Schema) |
| Notifications | 1 | `useNotifications` | Notification inbox |
| Feedback | 1 | `useFeedback` | Bug reports + feature requests |
| Exports | 1 | `useExports` | Export job management |
| Onboarding | 1 | `useOnboarding` | Step-by-step setup wizard |
| Correlation | 1 | `useCorrelation` | Statistical correlation explorer |
| Audit | 1 | `useAudit` | Immutable audit log viewer |
| 404 | 1 | — | Fallback route |

**Total: 35 routes** defined in `src/routes.tsx`, all lazy-loaded.

### Shared UI Primitives

20 reusable components in `src/components/ui/`:

Badge, Breadcrumb, Button, DataTable, EmptyState, EntityLink, FilterBar, Input, LoadingSkeleton, Modal, NotificationToast, SearchInput, StatCard, TabBar, Tag, TimeframeFilter, ToggleChip, Toast, ToastContainer, ToastProvider.

Each has a corresponding `.test.tsx` for coverage.

### Data Layer

- **Client** (`src/api/client.ts`): Thin fetch wrapper with JSON serialization, param handling, sanitized error messages, and `api:error` custom events
- **Types** (`src/api/types.ts`): 30+ TypeScript interfaces across 13 domains
- **Hooks** (`src/api/hooks/`): 18 hook modules (~50 hooks total) using TanStack Query with typed return values from MSW-mocked endpoints

---

## Routes

```typescript
// Core entity routes
/dashboard
/sessions, /sessions/:id
/memories, /memories/:id
/agents, /agents/:id
/skills, /skills/:id

// Efficiency (7 sub-pages)
/efficiency{/memory,/sessions,/agents,/skills,/tokens,/correlation}

// Analytics (7 sub-pages)
/analytics{/health,/performance,/resources,/costs,/models,/services}
/analytics/costs/models/:id

// Feature pages
/settings, /settings/:section
/search, /playground, /notifications, /feedback
/exports, /onboarding, /correlation, /audit
```

---

## Testing

- **530+ tests** across 77 test files (76 component/page/hook tests + 1 route test)
- **93 `describe` blocks** organizing test suites
- Framework: Vitest + Testing Library + jsdom
- Mock strategy: MSW handlers (15 modules) + 4 test factories
- Run: `npm test`, `npm run test:watch`, `npm run test:coverage`
- Coverage thresholds: 80% branches/functions/lines/statements

---

## Build

Production build produces **60+ chunks** with manual code splitting:

| Chunk | Contents |
|-------|----------|
| `vendor-react` | React, ReactDOM, React Router |
| `vendor-query` | TanStack Query |
| `vendor-fm` | Framer Motion |
| `vendor-charts` | Recharts (lazy loaded, excluded from preload) |
| `vendor-icons` | Lucide React |

Main entry: ~28KB. Chunk size warning limit: 300KB.

---

## Getting Started

```bash
# Install dependencies
npm install

# Start dev server (proxies /api → localhost:8051)
npm run dev

# Run tests
npm test

# Run tests in watch mode
npm run test:watch

# Type check
npm run typecheck

# Lint
npm run lint

# Production build
npm run build

# Preview production build
npm run preview
```

### Environment

The dev server runs on `http://localhost:5173` and proxies all `/api` requests to `http://localhost:8051` (configured in `vite.config.ts`). Start the Contexter API server first.

---

## Key Files

| Path | Purpose |
|------|---------|
| `src/App.tsx` | Root component, providers, router |
| `src/routes.tsx` | All route definitions (lazy-loaded) |
| `src/api/client.ts` | Fetch wrapper with auth and error handling |
| `src/api/types.ts` | All domain TypeScript interfaces |
| `src/api/hooks/` | 18 React Query hook modules |
| `src/styles/tokens.css` | V2-DEEP design system (Tailwind v4 theme) |
| `src/components/ui/` | 20 shared primitives |
| `src/components/layout/` | AppShell, Sidebar, TopBar, RootLayout |
| `tests/mocks/` | MSW handlers + factories |
| `vite.config.ts` | Vite config, proxy, chunk splitting |
| `vitest.config.ts` | Vitest config with coverage thresholds |

---

## Related

- [Contexter Core](../contexter-core/) — Rust storage engine
- [Contexter Server](../contexter-server/) — Python API server
