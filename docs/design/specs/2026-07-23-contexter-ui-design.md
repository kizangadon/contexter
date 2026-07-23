# Contexter — UI Design Specification

**Date:** 2026-07-23
**Status:** Draft
**Parent Hub:** [2026-07-23-contexter-specification-hub.md](2026-07-23-contexter-specification-hub.md)
**Architecture:** [2026-07-23-contexter-system-architecture.md](2026-07-23-contexter-system-architecture.md)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Purpose

This document is the **entry point and source of truth** for the Contexter user interface design. It defines the project-wide UI architecture, navigation structure, shared patterns, and the relationship between all sub-specs. Each major feature cluster has its own detailed sub-spec.

---

## 2. Project Overview

Contexter is a RAG-like memory, agent, skill, and session management platform for AI coding agents. It provides both a REST API and MCP Server interface over a Rust+Python modular monolith. The UI is a React single-page application serving as the primary human interface for managing sessions, memories, agents, skills, analytics, and system configuration.

---

## 3. Navigation Architecture

### 3.1 Global Navigation

The app shell follows a **left sidebar + top bar** layout:

- **Left sidebar** — primary navigation (collapsible):
  - Dashboard (home icon)
  - Sessions (message icon)
  - Memories (database icon)
  - Agents (robot icon)
  - Skills (puzzle icon)
  - Analytics (chart icon)
  - Settings (gear icon)
- **Top bar** — context-aware:
  - Page title / breadcrumbs
  - Global search trigger (`⌘K`)
  - Notification bell (unread badge)
  - User avatar / profile

### 3.2 Page Hierarchy

Contexter pages are organized into a shallow hierarchy with consistent patterns:

```
Dashboard                    /dashboard
├── Efficiency Mapper        /efficiency
│   ├── Memory Usage         /efficiency/memory
│   ├── Session Activity     /efficiency/sessions
│   ├── Agent Performance    /efficiency/agents
│   ├── Skill Effectiveness  /efficiency/skills
│   ├── Token Usage          /efficiency/tokens
│   └── Correlation Matrix   /efficiency/correlation
Session Manager              /sessions
├── Session Detail           /sessions/:id
Memory Explorer              /memories
├── Memory Detail            /memories/:id
Agent Registry               /agents
├── Agent Detail             /agents/:id
Skill Registry               /skills
├── Skill Detail             /skills/:id
Analytics                    /analytics
├── System Health            /analytics/health
├── Performance Trends       /analytics/performance
├── Resource Usage           /analytics/resources
├── Cost & Token Analytics   /analytics/costs
│   └── Model Detail         /analytics/costs/models/:id
├── Service Status           /analytics/services
Notification Center          /notifications
Feedback & Bug Reporter      /feedback
API Playground               /playground
Global Search                /search
Data Export & Reports        /exports
Cross-Session Correlation    /correlation
Versioning & Audit Trail     /audit
Settings                     /settings
├── General                  /settings/general
├── Storage                  /settings/storage
├── MCP Server               /settings/mcp
├── LLM Providers            /settings/llm
├── Notifications            /settings/notifications
├── Agents & Skills          /settings/agents-skills
├── Analytics                /settings/analytics
└── Data Management          /settings/data-management
Onboarding                   /onboarding
```

---

## 4. Shared UI Patterns (Global)

These patterns apply across ALL pages and are defined in the [design system](../V2-DEEP-design-system.md):

| Pattern | Description |
|---|---|
| **Timeframe filter** | Dropdown selector + Custom button (date range picker). Appears on all detail / data pages. |
| **Stat cards** | Grey-background cards with large number, small label, trend indicator. 4-across row typically. |
| **Data tables** | Minimal border, small font (11px), hover highlight, sortable headers. |
| **Toggle chips** | Grey capsule group for filter toggles. Active state: purple tint. |
| **Tag badges** | 8px font, 4px radius, colored backgrounds per category. |
| **Empty states** | Centered illustration + message + action button. |
| **Loading skeletons** | Grey pulsing rectangles matching card dimensions. |
| **Pagination** | Simple prev/next with page counter, centered at bottom. |
| **Entity links** | Purple (#7C5CFC) colored links to entity detail pages. |
| **Action icons** | 28×28px icon in colored background, placed on audit/log rows. |
| **Diff viewer** | GitHub-inspired: line-numbered, green additions, red deletions, neutral context. |

---

## 5. Sub-Specs

Each sub-spec is a standalone Markdown document in this directory. They follow a consistent structure: purpose, page-by-page breakdown, component sections, data flow, edge cases.

| # | Sub-Spec | Pages Covered | Status |
|---|---|---|---|
| 1 | [2026-07-23-phase-2-core-ui.md](2026-07-23-phase-2-core-ui.md) | Dashboard, Memory Explorer, Memory Detail, Session Manager, Session Detail, Agent Registry, Agent Detail, Skill Registry, Skill Detail, Efficiency Mapper | Draft |
| 2 | [2026-07-23-efficiency-mapper-details.md](2026-07-23-efficiency-mapper-details.md) | Memory Usage, Session Activity, Agent Performance, Skill Effectiveness, Token Usage, Correlation Matrix | Draft |
| 3 | [2026-07-23-system-analytics.md](2026-07-23-system-analytics.md) | Analytics Overview, System Health, Performance Trends, Resource Usage, Cost & Token Analytics, Model Detail, Service Status | Draft |
| 4 | [2026-07-23-settings-configuration.md](2026-07-23-settings-configuration.md) | All 8 Settings sections (General, Storage, MCP Server, LLM Providers, Notifications, Agents & Skills, Analytics, Data Management) | Draft |
| 5 | [2026-07-23-standalone-features.md](2026-07-23-standalone-features.md) | Notification Center, Feedback & Bug Reporter, Onboarding/First-Run, API Playground, Global Search, Data Export & Reports, Cross-Session Correlation, Versioning & Audit Trail | Draft |

---

## 6. Shared Component Library

The following components are shared across multiple pages and should be built once in the component library:

### Layout Components
- `AppShell` — sidebar + top bar + content area
- `PageHeader` — title + breadcrumbs + page-level actions
- `SidebarNav` — collapsible navigation with active state

### Data Display Components
- `StatCard` — number + label + optional trend indicator
- `DataTable` — sortable, filterable table with hover rows
- `EntityLink` — purple-colored link to entity detail
- `Tag` — colored label badge
- `ToggleChip` — filter toggle pill
- `EmptyState` — illustration + message + CTA
- `LoadingSkeleton` — pulsing placeholder

### Filter Components
- `TimeframeFilter` — dropdown + custom date picker
- `SearchInput` — text search with clear button
- `FilterBar` — row of selects + search inputs

### Navigation Components
- `TabBar` — horizontal tab navigation
- `Breadcrumb` — breadcrumb trail

### Feedback Components
- `NotificationBell` — top bar icon with unread count
- `NotificationToast` — slide-in notification
- `Modal` — overlay dialog

---

## 7. Mockup Files

Interactive mockup HTML files for all pages are located at:
```
.superpowers/brainstorm/1389270-1784815136/content/
```

Each sub-spec's page table references these by filename (e.g., `dashboard.html`). Open them in a browser to see the approved visual design.

---

## 8. Reading This Spec

1. Start here for the **global architecture** and navigation structure.
2. Read the [design system](../V2-DEEP-design-system.md) for tokens (colors, typography, spacing, shadows).
3. Read each sub-spec for page-level layout, components, data flow, and edge cases.
4. Sub-specs can be implemented independently and in any order, as long as the shared components (Section 6) exist first.

---

## 8. Implementation Priority (Recommended)

| Order | Cluster | Rationale |
|---|---|---|
| 1 | AppShell + Shared Components | Foundation — every page depends on these |
| 2 | Phase 2 Core UI | Core pages users interact with daily |
| 3 | Efficiency Mapper Details | Detail drill-downs from Phase 2 |
| 4 | System Analytics | Data-heavy, lower user frequency |
| 5 | Settings & Configuration | Needed early for setup flow |
| 6 | Standalone Features | Independent pages, lower priority |
