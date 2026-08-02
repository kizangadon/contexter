# Preview — Input Validation Gaps Repair

## Approach
```mermaid
flowchart TD
  A[store_memory] --> B{content empty/whitespace?}
  B -->|yes| C[structured error]
  B -->|no| D{content &gt; cap?}
  D -->|yes| E[structured error]
  D -->|no| F[persist]
  G[export_data] --> H{format in supported set?}
  H -->|no| I[structured error]
  H -->|yes| J[export]
  K[limit] --> L[clamp to 0..max]
```
Central validation helpers: non-empty content, size caps (content/query), export format allowlist, limit clamping. Errors never echo unbounded input.

## Fix boundary
Handlers store_memory/export_data/list_recent_sessions/search_memories + shared validators + TDD tests.

## Acceptance mapping
AC-IV-001..005, EC-IV-001..009.
