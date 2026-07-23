# Settings & Configuration — Pages

**Parent Spec:** [2026-07-23-contexter-ui-design.md](2026-07-23-contexter-ui-design.md)
**Design System:** [V2-DEEP-design-system.md](../V2-DEEP-design-system.md)

---

## 1. Pages Covered

| Section | Route | Mockup Ref |
|---|---|---|
| General | `/settings/general` | `settings-v2-general.html` |
| Storage | `/settings/storage` | `settings-v2-storage.html` |
| MCP Server | `/settings/mcp` | `settings-v2-mcp-server.html` |
| LLM Providers | `/settings/llm` | `settings-v2-llm-providers.html` |
| Notifications | `/settings/notifications` | `settings-v2-notifications.html` |
| Agents & Skills | `/settings/agents-skills` | `settings-v2-agents-skills.html` |
| Analytics | `/settings/analytics` | `settings-v2-analytics.html` |
| Data Management | `/settings/data-management` | `settings-v2-data-management.html` |

---

## 2. Common Pattern (All Settings)

All settings sections share:
- **Left nav:** 8-section vertical list. Active section highlighted with purple accent. Sticky on scroll.
- **Content area:** Section title, description, form groups.
- **Form groups:** Label + description + input. Save buttons per group or section.
- **Path fields** (Storage, etc.): Read-only display with permission badge (✅ Read/Write) + Browse... button. Changing path opens "Move Data" confirmation flow.
- **Save behavior:** Inline save per group (no global Save button). Toast on success. Error inline on field.

---

## 3. General

**Mockup:** `content/settings-v2-general.html`

- **Fields:** Project Name, Default Project, Language, Theme (Dark/Light/System), Telemetry toggle.
- **Edge cases:**
  - Empty project name: Validation error inline
  - Theme change: Preview immediately in a theme sample block

---

## 4. Storage

**Mockup:** `content/settings-v2-storage.html`

- **Display:** Storage location (read-only path), space used / total, file count.
- **Path field:** Read-only with ✅ Read/Write badge. "Browse..." button to change location.
- **Change path:** "Move Data" modal — shows source and destination paths, estimates data size, warns about time. "Start Migration" button. Progress bar during move.
- **Edge cases:**
  - Destination has insufficient space: Pre-flight check, show error before migration starts
  - Migration interrupted: Resume capability, show last checkpoint
  - Path not writable: Show ❌ badge with error message

---

## 5. MCP Server

**Mockup:** `content/settings-v2-mcp-server.html`

- **Fields:** Host, Port, Auth Token (masked), Allowed Origins, Enable/Disable toggle.
- **Status indicator:** Running / Stopped / Error with restart button.
- **Connection test:** "Test Connection" button with result toast.
- **Edge cases:**
  - Port conflict: Error on save, suggest available port
  - Token regeneration: Confirm dialog "This will invalidate existing clients"

---

## 6. LLM Providers

**Mockup:** `content/settings-v2-llm-providers.html`

- **Provider list:** Stacked cards — Provider name, status dot (Connected/Disconnected/Error), model count, last check.
- **Add provider:** Modal with Provider select (OpenAI, Anthropic, Ollama, etc.), API key field (masked), endpoint URL, models.
- **Per-provider:** Connection test button, model list with enable/disable toggles.
- **Edge cases:**
  - Invalid API key: Error on test with specific message
  - Rate limited: Show "Rate limited" status with retry-after hint
  - Provider removed: Cascade warning if models are in use

---

## 7. Notifications

**Mockup:** `content/settings-v2-notifications.html`

- **Channel toggles:** In-app, Email, Webhook. Each with enable/disable.
- **Event types table:** Rows — Session Complete, Error, Memory Created, Agent Updated, Budget Alert, System Alert. Columns — per-channel toggle.
- **Webhook URL:** Shown only when Webhook channel enabled. Test button.
- **Edge cases:**
  - Email not configured: Disable email column with tooltip "Configure email in General settings"
  - Too many notifications: "Quiet hours" setting with start/end time

---

## 8. Agents & Skills

**Mockup:** `content/settings-v2-agents-skills.html`

- **Tabs:** Agents / Skills
- **Agents tab:** List of installed agents with enable/disable toggles. "Add Agent" opens file picker for agent definition file.
- **Skills tab:** List of installed skills with enable/disable. Each shows name, version, file path (read-only), load count.
- **Refresh all button:** Reloads all agent/skill definitions.
- **Edge cases:**
  - Invalid agent definition file: Parsing error displayed inline, file not loaded
  - Agent in use during disable: Warning "This agent has active sessions"
  - Duplicate name: Error on add, prompt to rename

---

## 9. Analytics

**Mockup:** `content/settings-v2-analytics.html`

- **Toggles:** Enable Analytics Collection, Enable Token Tracking, Enable Cost Tracking.
- **Retention:** Data retention period (dropdown: 30d / 90d / 180d / 1 year / Forever).
- **Export:** "Export Analytics Data" button with format selector (JSON/CSV).
- **Edge cases:**
  - Disabling collection: Warning "This will delete existing data" with confirmation
  - Large export: Background job, notification when ready

---

## 10. Data Management

**Mockup:** `content/settings-v2-data-management.html`

- **Database status:** Engine type, size, connection status.
- **Actions:**
  - **Compact Database:** Runs VACUUM/optimization. Progress bar.
  - **Export All Data:** Format selector (JSON/CSV). Background job.
  - **Import Data:** File picker, preview of records to import, conflict resolution (Skip/Overwrite/Merge).
  - **Reset All Data:** Triple confirmation flow. Requires typing "RESET".
- **Edge cases:**
  - Database locked: Show "Database busy" with retry
  - Import format mismatch: Validation error with details
  - Large import: Background job with progress notification
