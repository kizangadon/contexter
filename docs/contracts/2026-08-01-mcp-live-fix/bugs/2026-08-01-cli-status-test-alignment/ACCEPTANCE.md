# ACCEPTANCE — CLI Status Test Alignment

- AC-CST-001: GIVEN a mocked engine `status()` returning `{cacheTelemetry: {...}, status: "...", version: "..."}`,
  WHEN the CLI status command runs, THEN it exits 0 and prints the interpolated values without error.
- AC-CST-002: GIVEN the real engine status payload shape, WHEN `test_status_shows_interpolated_values` runs,
  THEN it passes.
- AC-CST-003: GIVEN a status payload missing optional keys, WHEN the CLI formatter runs,
  THEN it degrades gracefully (no crash, no raw exception).
- AC-CST-004: WHEN the full suite runs, THEN 0 failures (all tests green).
