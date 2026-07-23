# Efficiency Mapper — Detail Pages

**Parent Spec:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)
**Parent Page:** Efficiency Mapper (`/efficiency`)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Pages Covered

| Page | Route | Mockup Ref |
|---|---|---|
| Memory Usage | `/efficiency/memory` | `efficiency-mapper-v2-memory-usage.html` |
| Session Activity | `/efficiency/sessions` | `efficiency-mapper-v2-session-activity.html` |
| Agent Performance | `/efficiency/agents` | `efficiency-mapper-v2-agent-performance.html` |
| Skill Effectiveness | `/efficiency/skills` | `efficiency-mapper-v2-skill-effectiveness.html` |
| Token Usage | `/efficiency/tokens` | `efficiency-mapper-v2-token-usage.html` |
| Correlation Matrix | `/efficiency/correlation` | `efficiency-mapper-v2-correlation-matrix.html` |

---

## 2. Common Pattern (All Pages)

Every detail page follows the same structure:

```
[Efficiency Mapper breadcrumb]  [Timeframe filter]
┌─────────────────────────────────────────────────┐
│ Stat Card  │ Stat Card  │ Stat Card  │ Stat Card │
├─────────────────────────────────────────────────┤
│                                                 │
│         Main visualization / chart               │
│                                                 │
├─────────────────────────────────────────────────┤
│         Supporting detail table / list           │
└─────────────────────────────────────────────────┘
```

- **Timeframe filter** (top right): Dropdown with Last 7d / 30d / 90d / All / Custom.
- **Stat cards:** 4 context-specific metrics per page.
- **Main area:** Chart or visualization dominant.
- **Supporting area:** Table or list with drill-down rows.

---

## 3. Memory Usage

**Mockup:** `content/efficiency-mapper-v2-memory-usage.html`

- **Stat cards:** Total Memories, Avg Memory Size, Memory Growth Rate, Unique Tags.
- **Main chart:** Line/area chart showing memory count over time. Overlay for memory type breakdown (stacked areas).
- **Supporting table:** Top memory producers — columns: Agent, Memories Created, Total Size, Avg Size, % of Total. Sortable.
- **Edge cases:**
  - No memories in period: Chart shows flat line, table is empty with message
  - Memory size spike: Tooltip on chart point shows exact value and timestamp

---

## 4. Session Activity

**Mockup:** `content/efficiency-mapper-v2-session-activity.html`

- **Stat cards:** Total Sessions, Active Now, Avg Duration, Avg Turn Count.
- **Main chart:** Bar chart — sessions per day. Color-coded by status (completed/active/error).
- **Supporting table:** Session list — columns: Session ID, Agent, Duration, Turn Count, Status, Start Time. Sortable. Row click → Session Detail.
- **Edge cases:**
  - Zero sessions: Chart empty, stat cards show 0, message "No session activity"
  - All-day session: Duration capped display with "(>24h)" label

---

## 5. Agent Performance

**Mockup:** `content/efficiency-mapper-v2-agent-performance.html`

- **Stat cards:** Active Agents, Avg Efficiency, Top Agent (name + score), Agents Below Threshold.
- **Main chart:** Horizontal bar chart — efficiency by agent. Color-coded (green ≥90%, amber ≥75%, red <75%).
- **Supporting table:** Agent list — columns: Agent Name, Sessions, Avg Turns, Efficiency %, Trend. Sortable.
- **Edge cases:**
  - Agent with 0 sessions: Show in table with "No sessions" label (don't hide)
  - Efficiency >100%: Cap display at 100%, flag in tooltip

---

## 6. Skill Effectiveness

**Mockup:** `content/efficiency-mapper-v2-skill-effectiveness.html`

- **Stat cards:** Total Skills, Avg Effectiveness, Most Used Skill, Skills Below Threshold.
- **Main chart:** Scatter or bubble chart — usage frequency vs effectiveness. Bubble size = sessions using it.
- **Supporting table:** Skill list — columns: Skill Name, Usage Count, Effectiveness %, Avg Token Impact, Trend. Sortable.
- **Edge cases:**
  - Skill used once with 100% effectiveness: Show normally, mark as "low sample" in table
  - Skill deprecated mid-period: Show usage up to deprecation date, grey out after

---

## 7. Token Usage

**Mockup:** `content/efficiency-mapper-v2-token-usage.html`

- **Stat cards:** Total Tokens, Avg Tokens/Session, Avg Cost/Session, Token Efficiency (tokens per meaningful output).
- **Main chart:** Stacked area chart — input tokens vs output tokens over time.
- **Supporting table:** Top token consumers — columns: Session ID, Agent, Input Tokens, Output Tokens, Total, Cost. Sortable.
- **Edge cases:**
  - Token overflow (context window exceeded): Highlight row in red with warning icon
  - Zero-cost period (local models): Show "N/A" for cost columns

---

## 8. Correlation Matrix

**Mockup:** `content/efficiency-mapper-v2-correlation-matrix.html`

- **Stat cards:** Metrics Correlated, Strongest Correlation (pair + value), Weakest Correlation, Data Points.
- **Main chart:** Heatmap matrix — grid of colored cells showing correlation coefficient (-1 to 1) between metric pairs. Color scale: red (negative) → white (neutral) → green (positive).
- **Supporting table:** Metric pair list — columns: Metric A, Metric B, Correlation Coefficient, Sample Size, Significance. Sortable by coefficient.
- **Edge cases:**
  - Insufficient data: Show "Not enough data points" on affected cells with strikethrough
  - Perfect correlation (1.0): Show as expected, note if sample is small
