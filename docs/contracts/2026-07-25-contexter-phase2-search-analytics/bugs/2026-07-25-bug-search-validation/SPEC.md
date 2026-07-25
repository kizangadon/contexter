# Bug: Missing Search Input Validation

**Severity:** HIGH  
**Root Cause:** `hybrid_search()` in `engine/search.rs` doesn't clamp vector_weight, cap limit, or handle NaN/Inf sort.

## Requirements

### REQ-FIX-001: Clamp vector_weight to [0.0, 1.0]
If `vector_weight < 0.0`, clamp to 0.0. If `vector_weight > 1.0`, clamp to 1.0.

### REQ-FIX-002: Cap limit to reasonable maximum
If `limit > 1000`, cap to 1000. If `limit == 0`, return empty results or cap to default (10).

### REQ-FIX-003: Handle NaN/Inf in sort_filed
If `sort_field` contains empty or whitespace-only string, treat as no sort and fall through cleanly. Document that NaN/Inf handling is the caller's responsibility for custom sort fields.

### REQ-FIX-004: Add unit tests for validation
Test: weight clamped, limit capped, empty sort_field handled, limit=0 returns empty.
