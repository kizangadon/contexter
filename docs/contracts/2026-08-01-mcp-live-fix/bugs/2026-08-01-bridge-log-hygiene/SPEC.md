# SPEC — Bridge Log Hygiene (unbounded content in args_summary)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-bridge-log-hygiene

## Problem
Bridge `_truncated_args_summary` logs up to 97-char prefixes of `content`/query args (Security LOW). This risks leaking user content to logs; the truncation limit is arbitrary and not token-budgeted.

## Requirements
- REQ-BH-001: Log summaries of content-bearing args are capped at a small, documented bound (e.g., ≤64 chars) and never log full content.
- REQ-BH-002: The summary function is unit-tested for the cap and for absence of full-content leakage.
- REQ-BH-003: No secret/auth material ever appears in bridge summaries (already true — keep it true; add regression test if feasible).
- REQ-BH-004: Full suite green (≥647/1 pre-existing).

## Constraints
Auth unchanged. DDD applies. Logs remain useful for debugging (args_summary still emitted, just bounded).
