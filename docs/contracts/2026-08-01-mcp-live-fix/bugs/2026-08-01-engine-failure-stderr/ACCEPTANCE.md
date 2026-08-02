# ACCEPTANCE — Engine-Failure Stderr Hygiene
- AC-EFS-001: GIVEN mid-call engine failure, WHEN server runs, THEN stderr shows ≤512 chars total for that failure and no raw traceback.
- AC-EFS-002: GIVEN same failure, THEN full exception detail is available in the diagnostics log file.
- AC-EFS-003: GIVEN client, THEN stdout frames are pure JSON (no contamination).
- AC-EFS-004: WHEN full suite runs, THEN 0 failures.
