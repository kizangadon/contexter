# SPEC — Analytics Agent/Skill Count Endpoints (iter-1 finding PERF-PF09, MEDIUM)

## Context
`AnalyticsService.get_overview` counts agents/skills by FULL-STORE scans: `list_agents({}, 1_000_000, 0)`
+ `list_skills({}, 1_000_000, 0)` materialize entire tables in Python just for `len()`. Engine has
`count_sessions`/`count_memories` but NO `count_agents`/`count_skills`. O(store size) on every
`contexter://analytics/overview` read + CLI status.

## Requirements
- REQ-ACE-001: Add engine-side `count_agents` and `count_skills` to the Rust engine core
  (contexter-core) mirroring the existing `count_sessions`/`count_memories` pattern.
- REQ-ACE-002: Expose both via the bridge (contexter-server/src/contexter_server/core/bridge.py)
  with the same sync/async pattern as existing counts.
- REQ-ACE-003: `AnalyticsService.get_overview` SHALL use the count endpoints instead of
  full-store `list_agents`/`list_skills` scans for the agent/skill counts.
- REQ-ACE-004: Live tests prove counts match seeded data and no full-store list call is made
  (spy/mock or call-count assertion).
- REQ-ACE-005: Full suite passes.
