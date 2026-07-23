# Standalone Features — Pages

**Parent Spec:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Pages Covered

| Page | Route | Mockup Ref |
|---|---|---|
| Notification Center | `/notifications` | `settings-v2-notification-center.html` |
| Feedback & Bug Reporter | `/feedback` | `feedback-bug-reporter.html` |
| Onboarding / First-Run | `/onboarding` | `onboarding-first-run.html` |
| API Playground | `/playground` | `api-playground.html` |
| Global Search (palette) | (⌘K overlay) | `global-search.html` |
| Global Search (page) | `/search` | `global-search-dedicated.html` |
| Data Export & Reports | `/exports` | `data-export-reports.html` |
| Cross-Session Correlation | `/correlation` | `cross-session.html` |
| Versioning & Audit Trail | `/audit` | `versioning-audit.html` |

---

## 2. Notification Center

**Mockup:** `content/settings-v2-notification-center.html`

**Layout:** Tabs (Inbox / History) + filters.

- **Inbox tab:** Unread notifications at top with blue dot indicator. Each item: icon, title, message snippet, timestamp, action button ("View Session", "Dismiss", etc.). "Mark all read" button.
- **History tab:** All past notifications, same layout, grey dot for read items. Filterable by type and date.
- **Filters:** Category dropdown (Session, Error, Memory, System, Budget) + Timeframe filter.
- **Edge cases:**
  - 100+ unread: Show count badge, "Mark all read" at top
  - Empty inbox: "All caught up!" with celebration illustration
  - Notification clicked: Navigate to relevant page, mark as read

---

## 3. Feedback & Bug Reporter

**Mockup:** `content/feedback-bug-reporter.html`

**Layout:** 3 tabs — Report a Bug / Send Feedback / Changelog.

- **Report a Bug tab:** Form — Title, Severity (Low/Medium/High/Critical), Description, Steps to Reproduce, Expected vs Actual, Attach logs (checkbox), Include session ID (auto-populated if applicable). Submit button.
- **Send Feedback tab:** Form — Category (Feature Request / Improvement / Other), Title, Description. Optional email. Submit button.
- **Changelog tab:** Reverse-chronological list of releases. Each entry: version number, date, sectioned by Added/Changed/Fixed/Removed.
- **Edge cases:**
  - Bug submitted with session ID: Link back to session for debugging
  - Empty required fields: Inline validation
  - Success: Toast with confirmation and ticket ID

---

## 4. Onboarding / First-Run

**Mockup:** `content/onboarding-first-run.html`

**Layout:** Bottom sheet (first visit) → Wizard (optional) → Checklist card (dashboard).

- **First visit:** Compact bottom-sheet with logo, welcome text, two buttons: "Run Setup Wizard" and "Explore On My Own".
- **Setup Wizard (4 steps):**
  1. **Storage** — Choose data directory, test write permission.
  2. **MCP Server** — Configure host/port, test connection.
  3. **LLM Provider** — Add first provider, test API key.
  4. **First Project** — Create project name, optional description.
- **Each step:** Title, description, form inputs. Back/Next/Finish buttons. Step indicator (4 dots) at bottom.
- **Explore On My Own:** Dashboard shows a checklist card — "Setup checklist (3 of 8 complete)" with checkable items linked to Settings sections.
- **Re-access:** Help menu > Setup Wizard re-opens the wizard.
- **Edge cases:**
  - Wizard abandoned mid-way: Progress saved, resume from where left off
  - Storage path invalid in step 1: Block next, show error with suggestion
  - API key fails in step 3: Show specific error, allow retry or skip
  - Returning user re-run wizard: Pre-fill known values, offer to skip already-completed steps

---

## 5. API Playground

**Mockup:** `content/api-playground.html`

**Layout:** Tabs — REST API / MCP Tools / Schema Explorer.

- **REST API tab:**
  - Method dropdown (GET/POST/PUT/DELETE) + URL input + Send button.
  - Parameters panel: Query params table, Headers table, Request body (JSON editor).
  - Response panel: Status code badge, response body (syntax highlighted), latency, size.
  - Footer bar: Headers / Auth / Schema / Curl (toggle views for request).
- **MCP Tools tab:**
  - Tool selector dropdown → Tool description + input schema.
  - Arguments editor (JSON) + Call button.
  - Response panel with result display.
- **Schema Explorer tab:**
  - Entity selector (Sessions/Memories/Agents/Skills) → Field list with types, descriptions, required markers.
  - Example payload generator.
- **Rich parameters:**
  - Searchable project dropdown with live filter.
  - Memory type `select` for relevant endpoints.
  - Query, limit, offset fields where applicable.
- **Edge cases:**
  - No server connection: Banner "API server not reachable" with settings link
  - Large response: Truncate display with "Response truncated" notice
  - Invalid JSON body: Inline validation before send
  - Auth required: Show auth section in footer, unauthenticated badge

---

## 6. Global Search

**Mockup:** `content/global-search.html` (palette) + `global-search-dedicated.html` (page)

### 6.1 Quick Palette (⌘K)
- Triggered by `⌘K` or search icon in top bar.
- Modal overlay: Search input at top, grouped results below.
- **Groups:** Sessions (recent), Memories (relevant), Agents, Skills.
- **Each result:** Icon + name + type badge + score badge (relevance %).
- **Keyboard nav:** Up/Down arrows to select, Enter to navigate, Esc to close.
- **Edge cases:**
  - No query: Show recent items for each group
  - No results: "No results found" with suggestion to try dedicated search page

### 6.2 Dedicated Search Page (`/search`)
- **Layout:** Search input (larger) → Entity-type filter toolbar (Sessions/Memories/Agents/Skills/All) → Sort dropdown (Relevance/Date/Name) → Project filter → Results list.
- **Results:** Cards with more detail than palette. Pagination at bottom.
- **Edge cases:**
  - Empty query: Show recent activity across entities
  - No results: "No results" with broader search suggestion
  - Hundreds of results: Server-side pagination, 20 per page

---

## 7. Data Export & Reports

**Mockup:** `content/data-export-reports.html`

**Layout:** 3 tabs — Export Data / Report Builder / Export History.

### 7.1 Export Data
- **Source selector grid (2×3):** Sessions, Memories, Agents, Skills, Settings, Analytics. Each card: icon, name, checkbox. Multi-select.
- **Options row (appears when sources selected):** Date range, Project dropdown, Scope (All / Current project), Compression (None / GZIP / ZIP).
- **Export button →** Progress bar → Download link.
- **Edge cases:**
  - No sources selected: Export button disabled with "Select at least one source"
  - Large export (>1GB): Background job, email notification when ready
  - Empty export: Warning "No data matches your filters"

### 7.2 Report Builder
- **Preset list (5):** Session Summary, Memory Overview, Agent Performance Report, Cost Analysis, System Health Report.
- **Per preset:** Name, description, schedule toggle (One-time / Daily / Weekly / Monthly), last generated.
- **Generate now button** → Progress → Download.
- **Edge cases:**
  - Scheduled report with no data: Skip, note in history
  - Report generation fails: Error in history with retry button

### 7.3 Export History
- **Table:** Date, Export name, Source(s), Format, Size, Status (Completed/Failed/In Progress), Actions (Download / Re-generate / Delete).
- **Filterable:** By status, date range, source type.
- **Edge cases:**
  - Expired download (older than 7 days): "Expired" badge, re-generate required
  - Concurrent export: Queue position shown in status

---

## 8. Cross-Session Correlation

**Mockup:** `content/cross-session.html`

**Layout:** 3 tabs — Overview / Timeline / Comparison.

### 8.1 Overview Tab
- **Stat bar:** Total Sessions, Unique Agents, Avg Efficiency, Avg Sessions/Day.
- **Timeframe filter:** Last 7d / 30d / 90d / All / Custom.
- **Two-column layout:**
  - **Left — Recurring Topics:** Ranked list of topics/clusters across sessions. Each row: topic name, session count, percentage bar.
  - **Right — Agent Performance Trends:** Table — Agent, Sessions, Efficiency bar, Trend arrow. Mini horizontal bars.
- **States:**
  - No sessions: Empty stat cards + "No session data" message
  - Single session: Still shows topics and agent stats, but notes limited sample

### 8.2 Timeline Tab
- **Filters:** Project, Agent, Timeframe.
- **Timeline feed:** Chronological list. Each entry: colored dot, timestamp, session title, session ID, agent list, turn count, tag badges.
- **States:**
  - No sessions in period: "No activity in this period" with extend-timeframe suggestion

### 8.3 Comparison Tab
- **Dual selectors:** Session A dropdown, Session B dropdown.
- **Comparison grid:** 3-column layout (A | VS | B). Metrics: Duration, Turn Count, Agents, Memories Created, Efficiency, Spec Adherence, Bugs Found. Each metric shows visual indicator (↑↓~) for A vs B.
- **Metric category chips:** Basic metrics / Agents involved / Memory activity / Cost & tokens.
- **States:**
  - Same session selected for A and B: Disable comparison, prompt to select different sessions
  - Large session: Handle gracefully (no timeout on metrics)

---

## 9. Versioning & Audit Trail

**Mockup:** `content/versioning-audit.html`

**Layout:** 3 tabs — Audit Log / File Versions / Diff Viewer.

### 9.1 Audit Log Tab
- **Stat bar:** Total Events, File Versions, Files Tracked.
- **Filter row:** Entity type (Sessions/Memories/Agents/Skills/Settings/Files), Action (Created/Edited/Deleted/Versioned), Actor (agent or human), Search input.
- **Event feed:** Reverse-chronological list. Each entry: action icon (✏️ edit, ➕ create, 🗑️ delete, 📄 version), entity name (clickable link), action description, agent avatar + actor name, timestamp.
- **Pagination:** Bottom of feed.
- **States:**
  - No events: "No audit events yet" with note about tracking
  - Filtered to empty: "No events match filters" with clear-filters action

### 9.2 File Versions Tab
- **File selector:** Dropdown listing all tracked files (AGENTS.md, SPEC.md, etc.)
- **File info header:** File icon, filename, path, version count badge.
- **Version comparison:** Base version selector + Compare version selector. Side-by-side dropdowns.
- **Summary bar:** What changed (additions, deletions, files affected).
- **States:**
  - No tracked files: "No version-tracked files" with instructions
  - Same version selected for base and compare: Show message to select different versions

### 9.3 Diff Viewer Tab
- **File header:** File name, version comparison label, add/del count.
- **Diff body:** GitHub-inspired unified diff:
  - Line numbers (left = old, right = new).
  - Green background/color for additions.
  - Red background/color for deletions.
  - Neutral grey for unchanged context lines.
- **Auto-versioning:** AGENTS.md, SPEC.md, and tracked config files snapshotted automatically on every content change. Each version stores: timestamp, content hash, full content.
