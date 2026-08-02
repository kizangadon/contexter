# ACCEPTANCE — Success-Path / Launch Log Hygiene

## AC-SH-001 — No WARNINGs on success path
- **Given** a live stdio server at default log level (open mode and key mode)
- **When** success-path calls run, including the analytics overview resource (both with and without `_api_key`)
- **Then** stderr shows INFO lifecycle events only — ZERO WARNING-level records

## AC-SH-002 — Signal preserved at DEBUG
- **Given** DEBUG logging enabled
- **When** the analytics resource is called without a key
- **Then** the `analytics.missing_key` event is visible (at DEBUG), so operators can still trace auth-missing calls

## AC-SH-003 — Clean launch stderr
- **Given** a launch without API key configured (open mode)
- **When** the server starts
- **Then** stderr contains NO warning preamble — clean start
- **And** for a corrupt-engine launch (rc=2), stderr contains exactly the failure line with NO pre-failure warning preamble

## AC-SH-004 — Auth enforcement unchanged
- **Given** the auth matrix tests
- **When** they run
- **Then** missing-key and wrong-key behavior is unchanged (client-visible frames byte-identical); only log levels changed

## AC-SH-005 — Suite green
- **Given** the full suite
- **Then** `python -m pytest -q` shows 881 + new tests passed, 0 failures (any existing tests asserting the old WARNING level updated accordingly)
