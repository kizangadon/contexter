# SPEC — Search Total Failure Silencing (iter-1 finding PERF-PF04, LOW re-stated)

## Context
Search performs 2 engine calls: results + count (full-scan). `return_exceptions=True` swallows
count-call exceptions → silent `total=0`. Pre-existing, but re-stated by Performance validator
this iteration — must be resolved.

## Requirements
- REQ-STF-001: The count failure path in the search handler/service SHALL NOT silently report
  `total=0` when the count engine call fails. Either: (a) propagate the failure as a structured
  error, or (b) compute total from the results call when it is authoritative, or (c) log an
  explicit error AND surface a distinguishing signal (e.g. total=-1 or a dedicated flag) while
  still returning results.
- REQ-STF-002: Happy path unchanged: total equals real count, results identical.
- REQ-STF-003: Tests: (a) count call fails → behavior is explicit (no silent 0), (b) happy path
  total correct.
- REQ-STF-004: Full suite passes.
