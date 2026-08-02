# SPEC — CLI Status Test Alignment

## Context
`tests/cli/test_status_format.py::test_status_shows_interpolated_values` fails in the full suite.
Root cause (flagged by B11/B12 Worker): the test mocks `StorageEngine.status()` with keys
(`uptime_seconds`, `engine_name`, etc.) that diverge from the real engine status shape
(`{cacheTelemetry, status, version}` as returned by `status()` and consumed by
`analytics_service.get_overview`). The CLI status path (`status_format.py` / CLI formatter)
interpolates values that do not exist in the real engine payload.

## Requirements

- REQ-CST-001: The CLI status test SHALL mock `StorageEngine.status()` with the real engine
  status shape (keys the analytics/CLI code actually reads), not invented keys.
- REQ-CST-002: If the CLI formatter reads keys that the real `status()` payload does not
  provide, the test SHALL assert the formatter's graceful degradation (the CLI output stays
  sane — no crash, no misleading blank fields) and any such formatter divergence SHALL be
  fixed in the formatter so real-engine output renders correctly.
- REQ-CST-003: The full test suite SHALL pass (all tests in `tests/cli/` green, no
  regressions elsewhere).
- REQ-CST-004: Tests SHALL be written before the formatter fix (TDD), proving the failure.

## Out of scope
- No changes to analytics_service.py or bridge (they return the real shape; they are correct).
- No changes to other test files beyond `tests/cli/`.
