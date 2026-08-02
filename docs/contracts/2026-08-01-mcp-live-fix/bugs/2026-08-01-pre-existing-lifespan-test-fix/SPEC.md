# SPEC — Pre-Existing Lifespan Test Fix

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-pre-existing-lifespan-test-fix

## Problem
`test_lifespan_shutdown_joins_thread` is the single failing test (647/1). Root cause: RocksDB LOCK contention when multiple engine instances share a data dir in tests, and/or the shutdown-join race. Proven pre-existing (not introduced by Fix A/B), but it blocks a fully green suite.

## Requirements
- REQ-LS-001: The test passes reliably: isolate per-test data dirs (unique temp dirs) so no two engine instances contend for one RocksDB LOCK.
- REQ-LS-002: Preserve the original test intent (shutdown joins the engine thread cleanly).
- REQ-LS-003: No global test-state coupling introduced; other tests unaffected.
- REQ-LS-004: Full suite fully green (≥648 passed, 0 failures) after fix.

## Constraints
Auth unchanged. DDD applies. This is a test-infrastructure fix — no production behavior change.
