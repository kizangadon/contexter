# Edge Cases — Contexter Phase 4 React UI

## Feature Scope

React SPA with 22+ pages covering session management, memory management, agent/skill registries, analytics, settings, and standalone features.

---

## Edge Case Categories

### E1: Network & API Failures

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-001 | API server is unreachable (connection refused) | Toast: "Cannot connect to server. Ensure the API is running on port 8051." All pages show loading state indefinitely with retry option. | High |
| EC-002 | API returns 401/403 | Show auth error toast. Redirect to settings for API key configuration. | High |
| EC-003 | API returns 404 for detail page | Show "Resource not found" page with link back to listing. | High |
| EC-004 | API returns 500 | Toast: "Server error. Please try again." Graceful degradation — show cached/empty data. | High |
| EC-005 | API request times out (30s+) | Toast: "Request timed out." Retry button on failed section. | Medium |
| EC-006 | WebSocket connection fails (notifications) | Polling fallback every 30s. | Medium |

### E2: Data & State Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-007 | Session list has 1000+ entries | Server-side pagination (25/page). Prev/Next navigation with page counter. | High |
| EC-008 | Memory search returns 0 results | "No memories match your search" with suggestion to broaden filters. | High |
| EC-009 | Dashboard has zero data (first run) | Empty states for each section. CTA: "Launch your first session." | High |
| EC-010 | Session has 100+ turns in timeline | Virtual scroll or paginated timeline. Default show 20, "Load more" button. | Medium |
| EC-011 | Memory content extremely long (100K+ chars) | Truncate to 500 chars in explorer. Full content in detail page with expand/collapse. | Medium |
| EC-012 | Agent/skill name is extremely long | Truncate with ellipsis in card and table views. Full name on hover tooltip. | Low |
| EC-013 | Entity has been deleted (referenced elsewhere) | Show "(deleted)" label on references (e.g., session referencing deleted agent). | Medium |

### E3: UI & Interaction Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-014 | User rapidly clicks navigation items | React Router cancels pending navigations. No duplicate page loads. | Medium |
| EC-015 | User resizes browser below 1024px | Sidebar auto-collapses. Tables show horizontal scroll. Cards stack vertically. | High |
| EC-016 | User opens sidebar and resizes to mobile | Sidebar overlays content (doesn't push it). Backdrop click closes sidebar. | Medium |
| EC-017 | Double-click on delete action | Button disabled after first click. Confirmation modal prevents double-execution. | High |
| EC-018 | Tab switch while data is loading | Keep loading state in new tab. Cancel in-flight requests for unmounted components. | Medium |
| EC-019 | Timeframe filter with no data in range | Show "No data for this period" inline on affected cards, not a full-page empty state. | Medium |
| EC-020 | Browser back/forward navigation | React Router handles history correctly. Query params preserved in filters. | High |

### E4: Settings-Specific Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-021 | Save settings with invalid data | Inline validation errors. Form not submitted until valid. | High |
| EC-022 | Concurrent settings saves | Last write wins. Toast on conflict notification. | Medium |
| EC-023 | API key field visibility | Toggle show/hide with eye icon. Key masked by default. | High |

### E5: Analytics & Data Display Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-024 | Chart has single data point | Render as dot, not line. Label with value. | Low |
| EC-025 | Chart has 1000+ data points | Downsample to ~200 points for rendering. Show "data sampled" indicator. | Medium |
| EC-026 | All metric values are zero | Show "0" prominently. Don't hide the card. | Low |
| EC-027 | Efficiency metric is exactly 100% | Show "100%" with green color. Full bar. | Low |

### E6: Export & Correlation Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-028 | Export takes longer than 5s | Show progress bar with estimated time. Cancel export button. | Medium |
| EC-029 | Export download fails mid-stream | Toast: "Download failed. Try again." | Medium |
| EC-030 | Correlation metrics are all identical | Show "Insufficient variance for correlation" message instead of misleading chart. | Medium |
| EC-031 | Audit trail has 10000+ entries | Server-side pagination (50/page). Date range filter strongly recommended. | Medium |

### E7: Notification & Feedback Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-032 | 100+ unread notifications | Show "99+" badge on bell icon. List paginated. | Medium |
| EC-033 | Bug report submitted with large attachment | Show progress indicator. 10MB max with error message if exceeded. | Medium |
| EC-034 | Changelog empty | Show "No changelog entries yet" empty state. | Low |

### E8: Onboarding Edge Cases

| ID | Scenario | Expected Behavior | Priority |
|----|----------|-------------------|----------|
| EC-035 | User refreshes during onboarding | Resume at current step (progress saved server-side). | High |
| EC-036 | User navigates away from onboarding | Return to dashboard. Onboarding accessible from settings. | Medium |

## Recovery Paths

| Scenario | User Message | Recovery Action |
|----------|-------------|-----------------|
| API unreachable | "Cannot connect to Contexter server" | "Check that the server is running on port 8051" with retry button |
| 404 on detail page | "[Entity] not found" | "Return to [entity] list" link |
| 500 on any request | "Something went wrong" | "Try again" button with auto-retry (max 3) |
| Mutation conflict | "This was just updated" | "Reload to see latest changes" |
| Network timeout | "Request timed out" | "Retry" button on affected component |
| Form validation error | Inline field errors | Scroll to first error field |
| Download fails | "Download failed" | "Try downloading again" link |

## Priority Classification

| Priority | Definition | Target Resolution |
|----------|------------|-------------------|
| High | Data loss, broken flow, blocked task | Fix before feature ships |
| Medium | Degraded UX, edge case, minor glitch | Fix if time permits or document known issue |
| Low | Cosmetic, rare edge, nice-to-have | Document and defer |
