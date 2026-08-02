# SPEC — Bridge Double-Encode on Bytes Path (PF-02)

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-bridge-double-encode

## Problem
For content ≥102400 bytes, the bridge bytes path double-encodes content (bridge.py:214/238) — the engine already returns bytes, and the bridge encodes them again, corrupting payloads (Performance LOW PF-02).

## Requirements
- REQ-BD-001: Bytes path decodes engine bytes exactly once; large payloads round-trip byte-identical.
- REQ-BD-002: `encode()`/`decode()` boundaries in the bridge are single-owner (each direction exactly one encode/decode) and documented.
- REQ-BD-003: TDD: round-trip test with content ≥102400 bytes asserting byte-identical content; full suite green.

## Constraints
Auth unchanged. DDD applies. Do not alter the string path semantics.
