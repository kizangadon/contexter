# EDGE CASES — Success-Path / Launch Log Hygiene

## EC-SH-001 — Don't lose the missing-key signal
Downgrade to DEBUG, do NOT delete the event — operators debugging auth issues rely on it (AC-SH-002).

## EC-SH-002 — Open mode still valid
Open mode (no key configured) is a supported configuration — the fix must not warn at import in open mode, nor require a key.

## EC-SH-003 — Existing tests asserting WARNING
Any test asserting `analytics.missing_key` at WARNING level MUST be updated to DEBUG (search `missing_key` across tests).

## EC-SH-004 — Other per-call events consistency
Check sibling per-call events (e.g., other `*_missing_key` or similar warning-level per-call logs) and align them to the PF-05 policy in the same change set — but do NOT touch ERROR-path events.

## EC-SH-005 — Launch warning placement
If the import-time warning is needed for diagnostics (e.g., when a key IS misconfigured), keep it at DEBUG or move it to the diagnostics log channel — never a default-level preamble.

## EC-SH-006 — No framework suppression changes
This contract does NOT modify `fastmcp_logging.py` (that's `fastmcp-filter-coverage`'s scope) — only contexter's own log statements/wiring.
