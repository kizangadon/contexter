# SPEC — Engine-Open Failure Handling (raw traceback on launch)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-launch-error-handling

## Problem
When engine open fails (e.g., RocksDB LOCK error / unreadable data dir), the launcher path leaks raw stderr/traceback to the client (Security LOW; SPEC LOW EC-011) and process behavior on failure is undefined.

## Requirements
- REQ-LH-001: Engine-open failure produces a clean, structured error message (no raw traceback in client-visible output).
- REQ-LH-002: Full raw diagnostics go to server-side logs (observability preserved).
- REQ-LH-003: Process exits cleanly (nonzero code) OR serves a degraded-but-documented state — choose one, document it, test it.
- REQ-LH-004: TDD: reproduction test launching with an unwritable/locked data dir; assert clean error and no traceback leak; full suite green.

## Constraints
Auth unchanged. DDD applies. Must not break the working launcher path.
