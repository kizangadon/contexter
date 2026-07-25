# Bug: Efficiency Cache TTL Check Still O(n)

**Severity:** LOW  
**Root Cause:** `get_efficiency_scores()` uses `HashMap::retain()` which iterates all entries even when only one session is queried.

## Requirements

### REQ-FIX-001: Per-entry TTL check
Replace the full cache scan with per-entry lazy TTL check: when `get_efficiency_scores("session-X")` is called, only check and remove the entry for "session-X", not all entries.
