# SPEC — Camelization Live-Coverage Tests (MED-1)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-camelization-coverage-tests

## Problem
Camelization was live-verified for only 4 of 34 engine methods (Code Reviewer MED-1; Security INFO camelize dual-spelling). The remaining methods' response shapes are unverified against the real engine, leaving latent schema-drift risk.

## Requirements
- REQ-CM-001: Add live-engine tests (or a systematic harness) that exercise ALL 34 engine methods through the bridge against the real engine.
- REQ-CM-002: Every method's response is validated against its Python model (pydantic) — no unverified response shapes.
- REQ-CM-003: Methods that cannot be live-tested are enumerated with documented reasons and mocked-but-shape-locked tests.
- REQ-CM-004: Full suite green (≥647/1 pre-existing).

## Constraints
Auth unchanged. DDD applies. Tests must run against the real Rust engine wheel (no stub).
