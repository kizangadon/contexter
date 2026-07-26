# Acceptance Criteria — Contexter Phase 4 React UI

> All criteria use Given/When/Then format and must pass for the feature to be considered complete.

---

## Foundation

### AC-001: Project scaffolds successfully
**Given** a fresh `npm create vite` with React + TypeScript template,
**When** `npm install` completes and `npm run dev` starts,
**Then** the dev server starts without errors, `npm run build` produces a production bundle, and `npm run test` runs Vitest.

### AC-002: Design tokens render correctly
**Given** `tokens.css` is imported into the app,
**When** any component references `var(--bg-base)`, `var(--accent)`, `var(--text-primary)`, etc.,
**Then** the correct V2-DEEP hex values are applied and visible in the browser.

### AC-003: Shared components render all states
**Given** a shared component (Button, Badge, DataTable, StatCard, etc.),
**When** it renders with loading data, empty data, normal data, and error props,
**Then** each state produces a distinct, correct visual output without console errors.

---

## AppShell & Navigation

### AC-004: AppShell renders with sidebar and top bar
**Given** the app loads at any route,
**When** the AppShell component mounts,
**Then** the left sidebar (240px) and top bar (56px) are visible with the content area filling the remaining space.

### AC-005: Sidebar collapses and expands
**Given** the sidebar is in expanded state,
**When** the user clicks the collapse toggle,
**Then** the sidebar shrinks to 60px showing only icons. Clicking toggle again restores 240px.

### AC-006: Navigation resolves all routes
**Given** the sidebar navigation,
**When** clicking each nav item (Dashboard, Sessions, Memories, Agents, Skills, Analytics, Settings),
**Then** the URL updates to the correct route and the corresponding page component renders.

### AC-007: Active route is highlighted
**Given** a nav item is clicked,
**When** the page loads,
**Then** the clicked nav item shows a purple accent left border and accent-muted background.

### AC-008: Unknown route shows 404 page
**Given** the user navigates to `/nonexistent-route`,
**When** the router resolves,
**Then** a 404 page is displayed with a link back to Dashboard.

---

## API Client & Hooks

### AC-009: API client makes correct requests
**Given** the API client is configured with base URL `http://localhost:8051/api/v1`,
**When** calling `api.get('/sessions')`,
**Then** it issues a `GET` request to `http://localhost:8051/api/v1/sessions` with JSON content-type header.

### AC-010: Hooks return typed data
**Given** a React Query hook `useSessions()`,
**When** the hook is called and data returns,
**Then** the data matches the expected TypeScript type (Session[]), and `isLoading`, `isError`, `error`, and `data` are all available.

### AC-011: Errors surface as toast notifications
**Given** an API call returns 4xx or 5xx,
**When** the error is caught by the hook,
**Then** a toast notification appears with the error message.

---

## Core Pages

### AC-012: Dashboard shows stat cards, recent sessions, quick actions
**Given** the user navigates to `/dashboard`,
**When** the page loads with data,
**Then** 4 stat cards (Total Sessions, Active Sessions, Total Memories, Avg Efficiency) are visible, a Recent Sessions table renders below, and 3 Quick Action buttons are displayed.

### AC-013: Dashboard handles empty state
**Given** the user navigates to `/dashboard` with zero sessions,
**When** the page loads,
**Then** an empty state message "No sessions yet" is shown with a CTA button.

### AC-014: Session Manager lists and filters sessions
**Given** the user navigates to `/sessions`,
**When** the page loads,
**Then** a stat cards row, filter bar (status, project, agent, date range), search input, and sortable sessions table are visible with pagination.

### AC-015: Session Detail shows tabbed content
**Given** the user clicks a session row or navigates to `/sessions/:id`,
**When** the detail page loads,
**Then** tabs render: Timeline, Messages, Memories, Metadata — each showing relevant content on click.

### AC-016: Memory Explorer searches and filters
**Given** the user navigates to `/memories`,
**When** the page loads,
**Then** a search bar, filter chips (project, agent, type, date range), and a results grid (card/list toggle) are visible with pagination.

### AC-017: Memory Detail shows content and metadata
**Given** the user clicks a memory card or navigates to `/memories/:id`,
**When** the detail page loads,
**Then** the memory content is displayed with a metadata sidebar (tags, project, agent, version count).

### AC-018: Agent Registry shows agent cards
**Given** the user navigates to `/agents`,
**When** the page loads,
**Then** a grid of agent cards is displayed with search, status filter, and category filter. Each card shows name, description, status dot, session count.

### AC-019: Agent Detail shows tabs
**Given** the user clicks an agent card or navigates to `/agents/:id`,
**When** the detail page loads,
**Then** tabs render: Overview, Sessions, Skills, Version History.

### AC-020: Skill Registry shows skill cards
**Given** the user navigates to `/skills`,
**When** the page loads,
**Then** a grid of skill cards with search, filter, effectiveness bar, and usage count is displayed.

### AC-021: Skill Detail shows tabs
**Given** the user clicks a skill card or navigates to `/skills/:id`,
**When** the detail page loads,
**Then** tabs render: Overview, Usage, Versions.

### AC-022: Efficiency Mapper shows metric grid
**Given** the user navigates to `/efficiency`,
**When** the page loads,
**Then** stat cards row + 3x2 grid of metric cards with sparklines (Memory Usage, Session Activity, Agent Performance, Skill Effectiveness, Token Usage, Correlation Matrix) are visible. Timeframe filter controls all cards.

---

## Analytics

### AC-023: Analytics Overview renders charts
**Given** the user navigates to `/analytics`,
**When** the page loads,
**Then** aggregated metrics and Recharts charts are visible with loading/empty/error states handled.

### AC-024: Analytics sub-pages render
**Given** the user navigates to `/analytics/health`, `/analytics/performance`, `/analytics/resources`, `/analytics/costs`,
**When** each page loads,
**Then** the appropriate health, performance, resource, or cost data is displayed with charts.

---

## Settings

### AC-025: Settings sidebar navigation works
**Given** the user navigates to `/settings`,
**When** the page loads,
**Then** a settings sidebar lists all 8 sections (General, Storage, MCP Server, LLM Providers, Notifications, Agents & Skills, Analytics, Data Management). Clicking each loads the corresponding form.

### AC-026: Settings save data correctly
**Given** a settings form is filled out,
**When** the user clicks Save,
**Then** a PUT request is sent to the API and a success confirmation toast appears.

---

## Standalone Features

### AC-027: Global Search renders results
**Given** the user navigates to `/search?q=test`,
**When** the page loads with results,
**Then** search results are displayed grouped by type.

### AC-028: API Playground shows REST/MCP tabs
**Given** the user navigates to `/playground`,
**When** the page loads,
**Then** tabs for REST API, MCP Tools, and Schema Explorer are visible with input fields and response panel.

### AC-029: Notification Center shows and marks read
**Given** the user navigates to `/notifications`,
**When** the page loads,
**Then** a list of notifications is shown with read/unread styling. Clicking marks as read.

### AC-030: Feedback shows 3 tabs
**Given** the user navigates to `/feedback`,
**When** the page loads,
**Then** Bug Report, Feature Request, and Changelog tabs are visible with appropriate forms.

### AC-031: Data Exports shows 3 tabs
**Given** the user navigates to `/exports`,
**When** the page loads,
**Then** Scheduled, Generated, and Templates tabs are visible.

### AC-032: Onboarding wizard shows steps
**Given** the user navigates to `/onboarding`,
**When** the page loads,
**Then** a multi-step wizard is displayed with progress indicator.

### AC-033: Correlation shows 3 tabs
**Given** the user navigates to `/correlation`,
**When** the page loads,
**Then** Overview, Timeline, and Compare tabs are visible.

### AC-034: Audit Trail shows entries with diff viewer
**Given** the user navigates to `/audit`,
**When** the page loads,
**Then** a list of audit entries is displayed with a GitHub-style diff viewer for changes.

---

## Testing

### AC-035: Component tests pass
**Given** shared components are implemented,
**When** `vitest run` executes component tests,
**Then** all component tests pass covering render, props, states, and user interactions.

### AC-036: Hook tests pass
**Given** all React Query hooks are implemented,
**When** `vitest run` executes hook tests with MSW,
**Then** all hook tests pass covering data fetch, loading, error, and mutation scenarios.

### AC-037: Route tests pass
**Given** all routes are defined,
**When** `vitest run` executes route integration tests,
**Then** each route renders its correct page component without errors.

### AC-038: Coverage threshold met
**Given** all tests pass,
**When** `vitest run --coverage` completes,
**Then** line coverage is at least 80%.
