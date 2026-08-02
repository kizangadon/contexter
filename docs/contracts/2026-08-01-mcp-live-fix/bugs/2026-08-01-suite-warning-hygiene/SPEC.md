# SPEC — Suite Warning Hygiene

> Parent: `2026-08-01-mcp-live-fix` · Auto Bug Loop Iteration 3
> Finding: **SPEC Compliance INFO** — `review-mcp-live-fix-spec-compliance-iter-3.md` (pre-existing starlette `PendingDeprecationWarning`)

## Problem

The full Python suite (`python -m pytest -q`) reports **1 warning**: a starlette `PendingDeprecationWarning` caused by python-multipart (infrastructure-level, 1 occurrence, pre-existing across iterations). It is the ONLY warning in an otherwise clean suite. A 0-warning suite is the quality bar; the warning must be deliberately resolved, not left as ambient noise.

## Requirements

### REQ-SW-001 — Zero-warning suite
`python -m pytest -q` SHALL report 0 warnings after the change.

### REQ-SW-002 — Deliberate, scoped resolution
The resolution SHALL be deliberate and NARROW: either (a) a targeted `filterwarnings` entry in `contexter-server/pyproject.toml` matching the specific `PendingDeprecationWarning` from python-multipart/starlette (with a justification comment and the library/version), or (b) a dependency pin/upgrade that resolves the deprecation at the source. A global `-W ignore` or blanket suppression is FORBIDDEN.

### REQ-SW-003 — Other warnings still surface
The change MUST NOT hide any other warnings — any future warning from a different source still appears.

## Non-Goals

- No change to application code behavior.
- No change to the 881-test suite content (unless a test itself triggers the warning).

## Artifacts

- Contract dir: `docs/contracts/2026-08-01-mcp-live-fix/bugs/2026-08-01-suite-warning-hygiene/`
- References: `plan/review/review-mcp-live-fix-spec-compliance-iter-3.md` (INFO finding)
