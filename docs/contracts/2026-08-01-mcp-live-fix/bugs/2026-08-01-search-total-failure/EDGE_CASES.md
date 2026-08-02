# EDGE_CASES — Search Total Failure Silencing
- EC-STF-001: results call fails, count succeeds → existing error path (structured HandlerError).
- EC-STF-002: both calls fail → error path, no crash.
- EC-STF-003: empty results, count 0 → total 0 legitimately (not a failure signal).
- EC-STF-004: count returns but results truncated by limit → total ≥ results len (documented semantics).
