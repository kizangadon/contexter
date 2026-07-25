# Bug: Analytics Sync Missing-Field Handling

**Severity:** MEDIUM  
**Root Cause:** `created_at` field in analytics sync may be missing/empty, which passes an empty string to `CAST(? AS TIMESTAMP)` causing a silent DuckDB error.

## Requirements

### REQ-FIX-001: Validate created_at before CAST
In the analytics sync logic (DuckDbEngine sync), check that `created_at` is non-empty before passing it to the DuckDB CAST. Skip records with invalid/missing timestamps and log a structured warning.
