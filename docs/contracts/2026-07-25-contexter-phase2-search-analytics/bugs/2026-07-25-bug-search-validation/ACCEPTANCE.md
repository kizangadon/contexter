# Acceptance Criteria — Bug-Search-Validation

- AC-01: `vector_weight = -0.5` is clamped to `0.0`
- AC-02: `vector_weight = 2.0` is clamped to `1.0`
- AC-03: `limit = 5000` is capped to `1000`
- AC-04: `limit = 0` returns empty results
- AC-05: `sort_field = ""` falls through without sort (no error)
- AC-06: Unit tests added for all clamping behavior
- AC-07: All existing tests continue to pass
