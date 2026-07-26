---
title: Contexter Phase 4 — React UI Specification
version: 1.0
date_created: 2026-07-26
owner: Orchestrator
tags: frontend, react, ui, contexter
---

# Contexter Phase 4 — React UI Specification

## 1. Purpose & Scope

This specification defines the React SPA frontend for Contexter — a RAG-like memory, agent, skill, and session management platform for AI coding agents. The React UI serves as the primary human interface for managing sessions, memories, agents, skills, analytics, and system configuration.

**Tech Stack:**
- React 19 + TypeScript (strict mode)
- Tailwind CSS v4 (design tokens as CSS custom properties)
- React Router v7 (client-side routing)
- TanStack Query v5 (server state management)
- Framer Motion (animations)
- Lucide React (icons)
- Recharts (charts)
- Vitest + Testing Library + MSW (testing)

**API Base URL:** `http://localhost:8051/api/v1`

**Scope:** Build all 22+ pages, the V2-DEEP design system component library, AppShell layout, API client + hooks, and complete test suite.

## 2. Definitions

| Term | Definition |
|------|------------|
| AppShell | Root layout component with sidebar navigation and top bar |
| V2-DEEP | The approved dark design system (Stripe-inspired, warm dark + purple accent) |
| TanStack Query | Server state management library for fetching, caching, and synchronizing server data |
| AppShell | Left sidebar + top bar + content area layout |
| MCP | Model Context Protocol — secondary API surface on port 8052 |

## 3. Requirements, Constraints & Guidelines

### REQ-001: Project Scaffold
- **REQ-001.1**: Vite-based React 19 + TypeScript project with strict mode
- **REQ-001.2**: Tailwind CSS v4 configured with V2-DEEP design tokens as CSS custom properties
- **REQ-001.3**: React Router v7 with all routes defined in a single routes config
- **REQ-001.4**: TanStack Query v5 configured with a QueryClientProvider
- **REQ-001.5**: Framer Motion configured for layout animations
- **REQ-001.6**: Lucide React installed for iconography
- **REQ-001.7**: Dev/build/lint/test scripts operational

### REQ-002: Design System Implementation
- **REQ-002.1**: All V2-DEEP tokens defined as CSS custom properties in `:root` within `tokens.css`
- **REQ-002.2**: Shared UI component library: `Button`, `Badge`, `Input`, `DataTable`, `StatCard`, `Modal`, `Toast`, `Tag`, `ToggleChip`, `EmptyState`, `LoadingSkeleton`, `TimeframeFilter`, `SearchInput`, `FilterBar`, `TabBar`, `Breadcrumb`, `EntityLink`, `NotificationToast`
- **REQ-002.3**: Every component handles loading, empty, error, and edge-case states
- **REQ-002.4**: Components are properly typed with TypeScript interfaces

### REQ-003: AppShell + Navigation
- **REQ-003.1**: Collapsible left sidebar (240px expanded, 60px collapsed)
- **REQ-003.2**: Top bar with page title, breadcrumbs, search trigger (⌘K), notification bell
- **REQ-003.3**: Sidebar items: Dashboard, Sessions, Memories, Agents, Skills, Analytics, Settings
- **REQ-003.4**: Active route highlighted in sidebar with accent left border
- **REQ-003.5**: All routes defined and resolvable
- **REQ-003.6**: 404 page for unknown routes

### REQ-004: API Client + Hooks
- **REQ-004.1**: Typed HTTP client wrapping `fetch()` targeting `http://localhost:8051/api/v1`
- **REQ-004.2**: React Query hooks for all API endpoints
- **REQ-004.3**: Optimistic updates where appropriate (session/memory CRUD)
- **REQ-004.4**: Error handling with toast notifications
- **REQ-004.5**: Loading states returned alongside data from hooks

### REQ-005: Core UI Pages
- **REQ-005.1**: Dashboard — stat cards, recent sessions table, quick actions
- **REQ-005.2**: Session Manager — filterable/sortable table with stat cards row
- **REQ-005.3**: Session Detail — tabs (Timeline/Messages/Memories/Metadata)
- **REQ-005.4**: Memory Explorer — search + filters + card grid/list toggle
- **REQ-005.5**: Memory Detail — content, metadata sidebar, version history
- **REQ-005.6**: Agent Registry — card grid with search/filter
- **REQ-005.7**: Agent Detail — tabs (Overview/Sessions/Skills/Version History)
- **REQ-005.8**: Skill Registry — card grid with search/filter
- **REQ-005.9**: Skill Detail — tabs (Overview/Usage/Versions)
- **REQ-005.10**: Efficiency Mapper — stat cards + 3x2 metric grid with sparklines

### REQ-006: Analytics Pages
- **REQ-006.1**: Analytics Overview — aggregated metrics with Recharts
- **REQ-006.2**: System Health — uptime, component status
- **REQ-006.3**: Performance Trends — line charts over time
- **REQ-006.4**: Resource Usage — memory, CPU, storage gauges
- **REQ-006.5**: Cost & Token Analytics — cost breakdowns
- **REQ-006.6**: Model Detail — per-model performance
- **REQ-006.7**: Service Status — live service indicators

### REQ-007: Settings Pages
- **REQ-007.1**: 8 settings sections with sidebar navigation
- **REQ-007.2**: General, Storage, MCP Server, LLM Providers, Notifications, Agents & Skills, Analytics, Data Management
- **REQ-007.3**: Read/write from API, with save confirmation

### REQ-008: Standalone Feature Pages
- **REQ-008.1**: Global Search — search results page
- **REQ-008.2**: Data Exports — Scheduled/Generated/Templates tabs
- **REQ-008.3**: Notification Center — read/unread list
- **REQ-008.4**: Feedback — Bug Report/Feature Request/Changelog tabs
- **REQ-008.5**: Onboarding — welcome wizard flow
- **REQ-008.6**: API Playground — tabbed REST/MCP/Schema Explorer
- **REQ-008.7**: Cross-Session Correlation — 3 tabs
- **REQ-008.8**: Versioning & Audit Trail — 3 tabs with diff viewer

### REQ-009: Testing
- **REQ-009.1**: Component tests for all shared UI components
- **REQ-009.2**: Hook tests for all React Query hooks
- **REQ-009.3**: MSW handlers mocking all API endpoints
- **REQ-009.4**: Route integration tests for all pages
- **REQ-009.5**: Minimum 80% line coverage

### CON-001: Constraints
- No Redux, Zustand, or alternative state managers — TanStack Query + local state only
- No CSS-in-JS — Tailwind v4 with CSS custom properties only
- No axios — native `fetch()` wrapper only
- Dark mode only — no light mode in v1
- Mobile-responsive but desktop-first (1440px max content width)

## 4. Interfaces & Data Contracts

### API Base Configuration
```
Base URL: http://localhost:8051/api/v1
Headers: Content-Type: application/json, X-API-Key: <key>
```

### Session Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/sessions` | List sessions (filter by project, status) |
| POST | `/sessions` | Create session |
| GET | `/sessions/{id}` | Get session detail |
| PUT | `/sessions/{id}` | Update session |
| DELETE | `/sessions/{id}` | Delete session |
| POST | `/sessions/{id}/resume` | Resume session |

### Memory Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/memories` | List memories |
| POST | `/memories` | Create memory |
| GET | `/memories/search` | Search memories |
| GET | `/memories/{id}` | Get memory detail |
| PUT | `/memories/{id}` | Update memory |
| DELETE | `/memories/{id}` | Delete memory |
| POST | `/memories/{id}/versions` | Create version snapshot |

### Agent Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/agents` | List agents |
| POST | `/agents` | Create agent |
| GET | `/agents/{id}` | Get agent detail |
| PUT | `/agents/{id}` | Update agent |
| DELETE | `/agents/{id}` | Delete agent |

### Skill Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/skills` | List skills |
| POST | `/skills` | Create skill |
| GET | `/skills/{id}` | Get skill detail |
| PUT | `/skills/{id}` | Update skill |
| DELETE | `/skills/{id}` | Delete skill |

### Analytics Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/analytics/overview` | Analytics overview |
| GET | `/analytics/health` | System health |
| GET | `/analytics/performance` | Performance metrics |

### Efficiency Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/efficiency/overview` | Efficiency overview |
| GET | `/efficiency/memory` | Memory usage |
| GET | `/efficiency/sessions` | Session activity |
| GET | `/efficiency/agents` | Agent performance |
| GET | `/efficiency/skills` | Skill effectiveness |
| GET | `/efficiency/tokens` | Token usage |
| GET | `/efficiency/correlation` | Correlation matrix |

### Settings Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/settings/{section}` | Get settings section |
| PUT | `/settings/{section}` | Update settings section |

### Other Endpoints
| Method | Path | Description |
|--------|------|-------------|
| GET | `/search?q=...` | Global search |
| GET | `/notifications` | List notifications |
| PUT | `/notifications/{id}/read` | Mark as read |
| POST | `/notifications/read-all` | Mark all as read |
| POST | `/feedback/bug` | Submit bug report |
| POST | `/feedback/suggest` | Submit feature request |
| GET | `/changelog` | Get changelog |
| GET | `/export/history` | Export history |
| POST | `/export/submit` | Submit export |
| GET | `/export/status/{id}` | Export status |
| GET | `/export/download/{id}` | Download export |
| GET | `/correlation/overview` | Correlation overview |
| GET | `/correlation/timeline` | Correlation timeline |
| GET | `/correlation/compare` | Correlation compare |
| GET | `/audit` | Audit trail |
| GET | `/onboarding/status` | Onboarding status |
| POST | `/onboarding/wizard` | Submit wizard step |
| GET | `/onboarding/progress` | Onboarding progress |
| GET | `/files` | List files |
| GET | `/files/{hash}/diff` | File diff |

## 5. Acceptance Criteria

See `ACCEPTANCE.md` for the complete list of 30+ Given/When/Then acceptance criteria covering all pages and features.

## 6. Test Automation Strategy

| Test Level | Framework | Scope | Target Coverage |
|---|---|---|---|
| Unit/Component | Vitest + Testing Library | Shared components, page components, custom hooks | 80%+ line coverage |
| Integration | Vitest + MSW | Route rendering, API integration, data flow | All routes |
| E2E | Playwright (future) | Critical user journeys | Post-MVP |

**MSW Handlers:** One handler file per domain (sessions, memories, agents, skills, analytics, settings, etc.) returning mock data matching the API contracts.

**CI/CD Integration:** `vitest run --coverage` runs on every build. Coverage threshold: 80%.

## 7. Rationale & Context

The decision to use a **minimal dependency stack** (no Redux, no CSS-in-JS, no axios) is intentional:
- **TanStack Query** eliminates the need for global state management — all UI state derives from server state
- **Tailwind v4 + CSS custom properties** provides design tokens without runtime CSS-in-JS overhead
- **Native fetch()** avoids an extra dependency for HTTP — the API surface is well-known and typed
- **Dark-only** simplifies the design system to a single theme, reducing token maintenance

Implementation follows the **dependency chain**: scaffold → design tokens → shared components → AppShell → API client → pages → tests. Each step unlocks the next.

## 8. Dependencies & External Integrations

### External Systems
- **API Server**: Contexter REST API at `http://localhost:8051/api/v1` — primary data source

### Technology Platform Dependencies
- **Node.js** >= 20.x
- **npm** >= 10.x
- **React 19**: Latest stable with concurrent features
- **Tailwind CSS v4**: Utility-first CSS framework
- **TypeScript 5.x**: Strict mode required

## 9. Validation Criteria

- All 30+ acceptance criteria pass
- All pages render without console errors
- All API hooks return typed data correctly
- All loading/empty/error states render correctly for each page
- Sidebar navigation resolves all routes
- Design system tokens match V2-DEEP specification
- MSW test suite passes with 80%+ coverage

## 10. Related Specifications

- [V2-DEEP Design System](../../design/V2-DEEP-design-system.md)
- [UI Design Specification](../../design/specs/2026-07-23-contexter-ui-design.md)
- [Phase 2 Core UI Spec](../../design/specs/2026-07-23-phase-2-core-ui.md)
- [System Analytics Spec](../../design/specs/2026-07-23-system-analytics.md)
- [Settings Configuration Spec](../../design/specs/2026-07-23-settings-configuration.md)
- [Standalone Features Spec](../../design/specs/2026-07-23-standalone-features.md)
- [Efficiency Mapper Details Spec](../../design/specs/2026-07-23-efficiency-mapper-details.md)
