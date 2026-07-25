# Bug: Efficiency Cache O(n) TTL Check

**Severity:** LOW  
**Root Cause:** The efficiency cache TTL check iterates ALL entries to find expired ones (O(n) per read). Should use lazy per-entry eviction.

## Requirements

### REQ-FIX-001: Lazy per-entry TTL check
Change the TTL expiry check from iterating the entire cache to checking only the requested entry's `cached_at` timestamp on each `get_efficiency_scores()` call. This is O(1) instead of O(n).
