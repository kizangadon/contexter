# System Analytics — Pages

**Parent Spec:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Pages Covered

| Page | Route | Mockup Ref |
|---|---|---|
| Analytics Overview | `/analytics` | `system-analytics-v2-overview.html` |
| System Health | `/analytics/health` | `system-analytics-v2-system-health.html` |
| Performance Trends | `/analytics/performance` | `system-analytics-v2-performance-trends.html` |
| Resource Usage | `/analytics/resources` | `system-analytics-v2-resource-usage.html` |
| Cost & Token Analytics | `/analytics/costs` | `system-analytics-v2-cost-token.html` |
| Model Detail | `/analytics/costs/models/:id` | `system-analytics-v2-model-detail.html` |
| Service Status | `/analytics/services` | `system-analytics-v2-service-status.html` |

---

## 2. Analytics Overview

**Mockup:** `content/system-analytics-v2-overview.html`

- **Stat bar (6-across):** Active Services, Avg Response Time, Uptime, Active Sessions, Total Memory, API Calls Today.
- **Grid (3×2):** Six clickable cards — System Health, Performance Trends, Resource Usage, Cost & Token Analytics, Service Status, LLM Usage.
  - Each card shows: title, mini sparkline, current value, trend arrow.
  - Click → corresponding detail page.
- **Timeframe filter:** Applies to all stat cards and sparklines.
- **States:**
  - Loading: Skeleton stat bar + skeleton grid
  - Empty: "No analytics data yet" with note about data collection requirements
  - Error: Per-card error state, non-affected cards remain live

---

## 3. System Health

**Mockup:** `content/system-analytics-v2-system-health.html`

- **Stat cards:** Uptime %, Active Services, Error Rate (24h), Avg Response Time.
- **Main chart:** Line chart — response time over time with p50/p95/p99 lines. Color-coded: green (normal), amber (warning), red (critical).
- **Error breakdown:** Donut chart — errors by type (Timeout, Rate Limit, Internal, Auth).
- **Recent incidents:** Compact list of recent service disruptions with timestamp, duration, status.
- **States:**
  - No incidents: "No recent incidents" message in incident list
  - Partial data outage: Show "Data gap" annotation on chart

---

## 4. Performance Trends

**Mockup:** `content/system-analytics-v2-performance-trends.html`

- **Stat cards:** Avg Latency, P95 Latency, P99 Latency, Throughput (req/min).
- **Main chart:** Dual-axis chart — latency (line) + throughput (bar) over time.
- **Breakdown table:** Performance by endpoint/service — columns: Endpoint, Avg Latency, P95, P99, Calls, Errors %. Sortable.
- **Annotation support:** Deployment markers, incident markers on timeline.
- **States:**
  - Insufficient data for percentile: Show "Insufficient data" for that metric
  - Zero-traffic period: Gap in chart with annotation

---

## 5. Resource Usage

**Mockup:** `content/system-analytics-v2-resource-usage.html`

- **Stat cards:** CPU Usage %, Memory Usage %, Disk Usage %, Network I/O.
- **Main chart:** Area chart — CPU + Memory over time (dual line).
- **Breakdown by service:** Table — columns: Service, CPU %, Memory MB, Disk MB, Network MB/s. Color-coded bars.
- **States:**
  - Resource collection not enabled: Banner with setup instructions
  - Process restart: Gap marker on chart

---

## 6. Cost & Token Analytics

**Mockup:** `content/system-analytics-v2-cost-token.html`

- **Stat cards:** Total Cost (period), Avg Cost/Day, Total Tokens, Avg Tokens/Session.
- **Main chart:** Stacked bar chart — daily cost by model/provider.
- **Breakdown table:** Model → Columns: Provider, Input Tokens, Output Tokens, Cost, Avg Cost/Call. Sortable.
- **Model names are clickable → Model Detail page.**
- **States:**
  - No cost data (self-hosted): Show "Cost tracking not configured" banner
  - Budget approaching limit: Amber highlight on stat card

---

## 7. Model Detail

**Mockup:** `content/system-analytics-v2-model-detail.html`

**Layout:** Breadcrumb (Analytics → Costs → Model Name) → Stat cards → 5 tabs.

- **Stat cards:** Total Cost, Total Calls, Avg Latency, Tokens/Call.
- **Tabs:**
  1. **Sessions** — Filterable list of sessions using this model
  2. **Bugs** — Bugs correlated with this model
  3. **Agents** — Agents using this model, with conversion rate column
  4. **Skills** — Skills used with this model, with effectiveness column
  5. **Insights** — Generated analysis: trends, anomalies, recommendations
- **Timeframe filter:** Controls all tabs.
- **States:**
  - Model not found: "Model not found" with link back to cost analytics
  - No session data for period: Tab shows empty state per tab

---

## 8. Service Status

**Mockup:** `content/system-analytics-v2-service-status.html`

- **Stat cards:** Services Online, Degraded, Offline, Uptime Avg.
- **Service cards:** Grid — each shows service name, status dot (green/amber/red), uptime %, response time, last incident.
  - Click → inline expansion with more metrics.
- **Legend:** Color-coded status definitions.
- **States:**
  - All services healthy: All green dots, "All systems operational" banner
  - Service flapping: Show "unstable" badge with warning count
