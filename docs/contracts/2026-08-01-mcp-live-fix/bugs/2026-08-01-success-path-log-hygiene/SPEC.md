# SPEC — Success-Path / Launch Log Hygiene

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Findings: User-Testing LOW (2 pre-existing `analytics.missing_key` WARNINGs on success path), User-Testing INFO (import-time API-key warning precedes every launch)

## Problem

AC-FL-005 requires: success-path stderr at default level shows **INFO lifecycle events only**. Two violations remain (both pre-existing, not new noise from iter-3 fixes, but they break the AC's letter):

1. **`analytics.missing_key` WARNINGs on the success path** — 2 observed. When the analytics overview resource is called without an `_api_key` (open mode or gated probe), a WARNING-level event is emitted per call. This is a per-call event → per the established policy (PF-05: per-call events at DEBUG, INFO reserved for lifecycle/errors), it belongs at DEBUG. It fires during NORMAL operation (open mode is a supported configuration), so default-level stderr carries WARNING noise.
2. **Import-time API-key warning precedes every launch** — a pre-existing warning fires at import time when the API key environment is unset, so even a clean launch failure (rc=2, ONE line) is preceded by a warning line on stderr. The launch stderr is not "clean" per the letter.

## Requirements

### REQ-SH-001 — `analytics.missing_key` at DEBUG
The per-call `analytics.missing_key` event SHALL be logged at DEBUG (not WARNING), consistent with the PF-05 per-call-at-DEBUG policy. The signal MUST NOT be lost — it remains visible at DEBUG level and in structured logs.

### REQ-SH-002 — Launch preamble removed
The import-time API-key warning that precedes every launch SHALL be removed or downgraded so launch stderr is clean: for a successful open-mode launch, no WARNING appears; for a failed launch (rc=2), stderr contains exactly the failure line (+ any INFO lifecycle events), no preamble noise.

### REQ-SH-003 — Letter met
At default log level, the success path SHALL emit INFO lifecycle events only (no WARNINGs), and launch stderr SHALL have no pre-failure warning preamble — AC-FL-005 letter satisfied end-to-end.

## Non-Goals

- No change to auth enforcement (missing key still rejected; open mode still works).
- No change to bridge/handler ERROR paths.
- No change to client-visible frames.

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-success-path-log-hygiene/`
- References: `plan/review/review-mcp-live-fix-user-testing-review-iter-3.md` (LOW + INFO findings), `bugs/2026-08-01-handler-observability/` (PF-05 policy)
