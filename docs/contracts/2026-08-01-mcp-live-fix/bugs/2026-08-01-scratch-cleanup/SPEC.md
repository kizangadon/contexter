# SPEC — Scratch File Cleanup (docs/tests/)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-scratch-cleanup

## Problem
12+ leftover scratch files remain in `docs/tests/` and `contexter-server/docs/tests/` (Code Reviewer LOW-2). The directory is a temporary workspace and MUST be cleaned after validation sessions.

## Requirements
- REQ-SC-001: Remove ALL scratch files under `docs/tests/` and `contexter-server/docs/tests/` (they are gitignored; no legitimate committed content).
- REQ-SC-002: Verify nothing referenced by the suite or docs depends on those files.
- REQ-SC-003: Verify the directories themselves remain gitignored.
- REQ-SC-004: Full suite green (≥647/1 pre-existing) after cleanup.

## Constraints
Auth unchanged. DDD applies. This is a cleanup task — no production code changes unless required for hygiene.
