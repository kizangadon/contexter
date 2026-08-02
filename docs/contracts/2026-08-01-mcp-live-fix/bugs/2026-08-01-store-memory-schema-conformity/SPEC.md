# SPEC — store_memory Schema Conformity

**Parent:** 2026-08-01-mcp-live-fix · **Bug:** 2026-08-01-store-memory-schema-conformity

## Problem
`store_memory` registers 3 extra optional parameters (`tokens`, `tokenizer`, `model`) beyond the frozen contract table (which declares only `session_id`, `role`, `content`, `_api_key`).

## Requirements
- REQ-SM-001: Registered schema for `store_memory` matches the frozen contract table exactly (extra params removed OR explicitly justified and contract-aligned — frozen contract wins).
- REQ-SM-002: Handler signature aligns with the registered schema (no drift either direction).
- REQ-SM-003: TDD: schema-registration test asserting exact param set; full suite green.

## Constraints
Auth unchanged. DDD applies. The frozen API contract table is authoritative.
