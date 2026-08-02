# ACCEPTANCE — Search Total Failure Silencing
- AC-STF-001: GIVEN count engine call fails, WHEN search runs, THEN total is NOT silently 0 — explicit signal present (error response OR flag OR logged-and-distinguishable value).
- AC-STF-002: GIVEN healthy engine, WHEN search runs, THEN total equals real count.
- AC-STF-003: WHEN full suite runs, THEN 0 failures.
