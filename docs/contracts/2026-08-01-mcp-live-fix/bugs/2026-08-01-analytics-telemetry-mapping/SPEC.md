# SPEC — Analytics/Health Telemetry Mapping Repair

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-analytics-telemetry-mapping

## Problem
`contexter://analytics/overview` and `get_system_health` counters are structurally always 0 with real data present. Engine returns camelCase telemetry (`totalOps`, `entriesByType`, `total`, `perCf`/`walSize`) but Python reads snake_case keys (`total_sessions`, `total_bytes`, `uptime_seconds`) — mismatches silently masked by `_safe_get`.

## Requirements
- REQ-AN-001: Analytics overview returns real counts (sessions, memories, agents, skills) from the same engine store.
- REQ-AN-002: `get_system_health` maps real engine telemetry (storage size, ops, uptime where available).
- REQ-AN-003: `_safe_get` no longer masks key mismatches — either correct mapping or explicit key-error logging.
- REQ-AN-004: TDD reproduction tests: seed data → counters reflect seeded data over live stdio; full suite green.

## Constraints
Auth unchanged. DDD applies. Analytics remains a domain service; translation at boundary.
