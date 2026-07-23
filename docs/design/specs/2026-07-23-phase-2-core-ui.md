# Phase 2 — Core UI Pages

**Parent Spec:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Pages Covered

| Page | Route | Mockup Ref |
|---|---|---|
| Dashboard | `/dashboard` | `dashboard.html` |
| Memory Explorer | `/memories` | `memory-explorer.html` |
| Memory Detail | `/memories/:id` | `memory-detail.html` |
| Session Manager | `/sessions` | `session-manager-v5.html` |
| Session Detail | `/sessions/:id` | `session-detail.html` |
| Agent Registry | `/agents` | `agent-registry.html` |
| Agent Detail | `/agents/:id` | `agent-detail.html` |
| Skill Registry | `/skills` | `skill-registry.html` |
| Skill Detail | `/skills/:id` | `skill-detail.html` |
| Efficiency Mapper | `/efficiency` | `efficiency-mapper.html` |

---

## 2. Dashboard

**Layout:** Stat cards row (4-across) → Recent Sessions table → Quick Actions row.
**Mockup:** `content/dashboard.html`

- **Stat cards:** Total Sessions, Active Sessions, Total Memories, Avg Efficiency. Each has a trend arrow.
- **Recent Sessions:** Compact table with columns: Session ID, Agent, Status, Duration, Turn Count, Last Active. Row click → Session Detail.
- **Quick Actions:** Launch Session, Explore Memories, View Analytics. Card-style buttons with icons.
- **States:**
  - Loading: 4 skeleton stat cards + skeleton table rows
  - Empty: "No sessions yet. Launch your first one." + CTA button
  - Error: Banner with retry, cards/table show last known data

---

## 3. Memory Explorer

**Layout:** Search + filter bar → Results grid (card or list toggle).
**Mockup:** `content/memory-explorer.html`

- **Search:** Full-text search across memory content.
- **Filters:** Project dropdown, Agent dropdown, Memory type (fact/preference/procedure/context/episode), Date range, Tags.
- **Results:** Card view (default) with title, content snippet, type badge, timestamp. List view option. Sort by relevance/date.
- **Pagination:** Bottom of results, page counter.
- **States:**
  - Loading: 8 skeleton cards in grid
  - Empty: "No memories match your search" with suggestion to broaden filters
  - Error: Banner with retry

---

## 4. Memory Detail

**Layout:** Breadcrumb → Memory header (type badge, title) → Content section → Metadata sidebar.
**Mockup:** `content/memory-detail.html`

- **Header:** Memory type badge (colored), title, created/updated timestamps.
- **Content:** Full memory text with semantic highlighting.
- **Metadata sidebar:** Tags, project, agent, session, version history count.
- **Actions:** Edit, Delete, Copy, View in session context.
- **Version history:** Expandable list of previous versions with timestamps. Clicking opens diff view.
- **States:**
  - Loading: Skeleton block for content, skeleton sidebar
  - Not found: "Memory not found" with link back to explorer
  - Error: Banner with retry

---

## 5. Session Manager

**Layout:** Stat cards row → Filter row → Sessions table → Pagination.
**Mockup:** `content/session-manager-v5.html`

- **Stat cards:** Active Sessions, Completed Today, Avg Duration, Avg Turn Count.
- **Filter row:** Status (Active/Completed/Error), Project, Agent, Date range, Search by ID.
- **Table columns:** Session ID, Project, Agent, Status (badge), Duration, Turn Count, Created, Last Active. Sortable.
- **Actions per row:** Resume, View detail, Delete.
- **States:**
  - Loading: Skeleton cards + skeleton table
  - Empty: "No sessions found" with clear filters action
  - Error: Banner with retry

---

## 6. Session Detail

**Layout:** Session header → Tabs: Timeline / Messages / Memories / Metadata.
**Mockup:** `content/session-detail.html`

- **Header:** Session ID, status badge, agent, project, timestamps.
- **Timeline tab:** Chronological turn list with message bubbles, agent labels, latency indicators.
- **Messages tab:** Compact message log view, filterable by role (user/assistant/system).
- **Memories tab:** List of memories created during this session, clickable → Memory Detail.
- **Metadata tab:** Full session metadata, tags, token usage, cost breakdown.
- **Actions:** Resume, Export, Delete.
- **States:**
  - Loading: Skeleton header + skeleton tabs
  - Not found: "Session not found" with link back to manager
  - Error: Banner with retry

---

## 7. Agent Registry

**Layout:** Grid of agent cards with search/filter.
**Mockup:** `content/agent-registry.html`

- **Search:** By agent name or description.
- **Filters:** Status (Active/Inactive), Category, Capability.
- **Cards:** Agent icon/avatar, name, short description, status dot, session count, efficiency badge.
- **Click → Agent Detail.**
- **States:**
  - Loading: 8 skeleton cards
  - Empty: "No agents registered" with link to settings
  - Error: Banner with retry

---

## 8. Agent Detail

**Layout:** Agent header → Tabs: Overview / Sessions / Skills / Version History.
**Mockup:** `content/agent-detail.html`

- **Header:** Agent name, avatar, status, type, created/updated.
- **Overview tab:** Description, capability tags, performance stat cards (sessions, efficiency, avg turns).
- **Sessions tab:** Filterable list of sessions this agent ran.
- **Skills tab:** Skills the agent uses, with effectiveness ratings.
- **Version History tab:** Agent definition change log.
- **States:**
  - Loading: Skeleton header + skeleton tabs
  - Not found: "Agent not found" with link back to registry

---

## 9. Skill Registry

**Layout:** Grid of skill cards with search/filter. Mirrors Agent Registry pattern.
**Mockup:** `content/skill-registry.html`

- **Search:** By name or description.
- **Filters:** Category, Status, Effectiveness range.
- **Cards:** Skill icon, name, description, effectiveness bar, usage count, version.
- **States:** Same pattern as Agent Registry.

---

## 10. Skill Detail

**Layout:** Skill header → Tabs: Overview / Usage / Versions.
**Mockup:** `content/skill-detail.html`

- **Overview tab:** Description, category, effectiveness stat, agents using it, last used.
- **Usage tab:** Sessions that invoked this skill.
- **Versions tab:** Skill definition history with diffs.
- **States:** Same pattern as Agent Detail.

---

## 11. Efficiency Mapper

**Layout:** Stat cards row → 3×2 grid of metric cards → Links to detail pages.
**Mockup:** `content/efficiency-mapper.html`

- **Stat cards:** Avg Efficiency, Total Sessions Trend, Avg Tokens/Session, Avg Duration.
- **Grid cards (6):** Memory Usage, Session Activity, Agent Performance, Skill Effectiveness, Token Usage, Correlation Matrix. Each card shows a mini sparkline + current value + trend.
- **Click any card →** corresponding detail page in sub-spec [2026-07-23-efficiency-mapper-details.md](2026-07-23-efficiency-mapper-details.md).
- **Timeframe filter:** Controls all stat cards and sparklines.
- **States:**
  - Loading: Skeleton cards + skeleton grid
  - Empty: "No session data yet" with link to create a session
  - Error: Banner with retry

---

## 12. Edge Cases (All Pages)

- **Timeframe filter with no data:** Show "No data for this period" inline on affected cards, not a full-page empty state.
- **Long session lists (>1000):** Server-side pagination, 25 per page default.
- **Deleted entities:** Show "(deleted)" label in references (e.g., a session referencing a deleted agent).
- **Concurrent updates:** Toast notification when data refreshes from server while user is viewing.
- **Permission errors:** Grey out action buttons with tooltip explaining missing permission.
