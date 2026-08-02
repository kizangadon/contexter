# EDGE_CASES — CLI Status Test Alignment

- EC-CST-001: status payload missing `cacheTelemetry` → formatter shows default/safe value, no crash.
- EC-CST-002: status payload missing `version` → no crash; version field renders as safe fallback.
- EC-CST-003: engine raises on `status()` → CLI error path (existing behavior) unchanged.
- EC-CST-004: mock shape change does not break other CLI tests (`tests/cli/` suite green).
- EC-CST-005: no changes leak into `tests/mcp/`, `tests/core/`, or `tests/services/`.
