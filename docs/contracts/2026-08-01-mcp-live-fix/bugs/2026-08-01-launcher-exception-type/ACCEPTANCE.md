# ACCEPTANCE — Launcher Exception Type Pin
- AC-LET-001: GIVEN corrupt engine data dir, WHEN build_services runs, THEN a RuntimeError (pinned type) is raised.
- AC-LET-002: WHEN repo grep runs, THEN zero `pytest.raises(Exception)` in tests/.
- AC-LET-003: WHEN full suite runs, THEN 0 failures.
