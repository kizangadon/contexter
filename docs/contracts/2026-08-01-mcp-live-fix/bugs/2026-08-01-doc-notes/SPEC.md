# SPEC — Documentation Notes (docs + INFO findings)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-doc-notes

## Problem
Documentation-level findings across validators: Security INFO-1 (env typo documented/preserved), Design INFO-1 (env typo preserved), OBS-1 (`_safe_get` masks key mismatches — now fixed by B3, document), PF-05..08 INFO notes (architecture choices, hard wheel dependency, type shadowing), CR-7 INFO (hard wheel dependency in setup.py), Dev-4 (run_in_executor vs asyncio.to_thread decision), Dev-6.

## Requirements
- REQ-DN-001: Update project docs (README/ARCHITECTURE/design preview notes where appropriate) to reflect: canonical `CONTEXTER_*` env vars, engine as hard dependency (Rust wheel), thread-pool bridge design decision, `_safe_get` behavior.
- REQ-DN-002: Document known INFO findings as accepted decisions with rationale (no code change needed for pure-doc items).
- REQ-DN-003: No stale references to the typo env var in docs.
- REQ-DN-004: Full suite green (≥647/1 pre-existing).

## Constraints
Auth unchanged. DDD applies. Docs only — no production code changes in this contract.
