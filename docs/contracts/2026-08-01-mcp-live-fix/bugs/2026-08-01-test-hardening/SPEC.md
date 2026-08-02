# SPEC — Test Hardening (broad pytest.raises, missing edge tests)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-test-hardening

## Problem
- `pytest.raises(Exception)` used broadly in resource-auth tests (Code LOW-3) — masks unexpected errors, can false-pass.
- Missing tests: AC-7 empty-engine (SPEC LOW), EC-006/EC-009/EC-011 edge tests absent (SPEC INFO).
- Test isolation concerns for the known-flaky lifespan test (see B15).

## Requirements
- REQ-TH-001: Replace broad `pytest.raises(Exception)` with specific exception types/assertions.
- REQ-TH-002: Add missing edge tests: empty-engine behavior, empty-content, limit edges, launch failure.
- REQ-TH-003: No test asserts on `Exception` generally; every raises is precise.
- REQ-TH-004: Full suite green (≥647/1 pre-existing).

## Constraints
Auth unchanged. DDD applies. Do not weaken any existing assertion.
