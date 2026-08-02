# Preview — Analytics/Health Telemetry Mapping Repair

## Approach
```mermaid
flowchart LR
  A[Engine telemetry\ncamelCase: totalOps, entriesByType,\ntotal, perCf, walSize] --> B[Translation mapping\ncamelCase -> snake_case domain]
  B --> C[AnalyticsOverview\nreal counters]
  B --> D[SystemHealth\nreal storage/ops]
  E[_safe_get] -->|key mismatch| F[explicit log,\nnot silent zero]
```
Add explicit key mapping in analytics/health services; `_safe_get` logs missing keys at debug/warn instead of silent zero.

## Fix boundary
`services/analytics_service.py` (or health handler), `_safe_get` helper, + TDD tests (seed → non-zero counters).

## Acceptance mapping
AC-AN-001..003, EC-AN-001..005.
