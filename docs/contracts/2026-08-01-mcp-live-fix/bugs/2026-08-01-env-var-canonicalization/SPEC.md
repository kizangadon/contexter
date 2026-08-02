# SPEC — Env Var Canonicalization (typo `CONtexTER_BRIDGE_POOL_SIZE`)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-env-var-canonicalization

## Problem
Bridge reads `CONtexTER_BRIDGE_POOL_SIZE` (bridge.py:112) — a typo of the canonical `CONTEXTER_BRIDGE_POOL_SIZE`. Non-canonical env prefix `CONtexTER_` also appears in other code paths (Security LOW, Performance LOW PF-03). Users/validators documented `CONTEXTER_*` as canonical.

## Requirements
- REQ-EV-001: All env var reads use the canonical `CONTEXTER_` prefix only.
- REQ-EV-002: `CONTEXTER_BRIDGE_POOL_SIZE` is the single source for pool size; typo variant removed (no silent fallback read of the misspelled name).
- REQ-EV-003: Any legacy alias handling (if kept) is explicit, logged, and tested — otherwise removed entirely.
- REQ-EV-004: TDD: test asserting pool size honors `CONTEXTER_BRIDGE_POOL_SIZE`; full suite green.

## Constraints
Auth unchanged. DDD applies. No behavior regression for existing configs that used the typo (log a deprecation note if aliasing is retained).
