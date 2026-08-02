# SPEC — Error-Shape Drift Repair

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-error-shape-drift

## Problem
Handler error paths return `{"error": ...}` inside a **success** result (isError=False) instead of the frozen contract's structured MCP error shape. Confirmed live: not-found returns success payload `{"error": "not found"}`; message ≠ `Resource not found: <id>` (REQ-007/AC-6/EC-001 deviation).

## Requirements
- REQ-ES-001: All handler error paths (not-found, invalid params, engine failures) produce structured MCP errors per the frozen contract (isError=True / MCP error shape; `Resource not found: <id>` message convention for missing entities).
- REQ-ES-002: Auth errors keep their existing MCPAuthError serialization (verified working — do not regress).
- REQ-ES-003: No success-result smuggling: a failed operation never returns a 200-style success frame.
- REQ-ES-004: TDD: reproduction tests for each error path asserting the frozen error shape; full suite green.
- REQ-ES-005: Process survives every error path; stdout purity preserved.

## Constraints
Auth model unchanged. DDD applies. Do not modify SPEC.md of parent contract — this contract aligns implementation to the frozen parent contract.
