# Bug: Tantivy QueryParser Rebuilt Per Search

**Severity:** LOW  
**Root Cause:** `search()` in `TantivyIndex` constructs a new `QueryParser` on every call, causing per-query allocation overhead.

## Requirements

### REQ-FIX-001: Cache QueryParser
Store a cached `QueryParser` instance in `TantivyIndex` and reuse it across `search()` calls.
