# ACCEPTANCE — store_memory Schema Conformity

## AC-SM-001
GIVEN the MCP server
WHEN `tools/list` is called
THEN `store_memory` schema declares exactly `session_id`, `role`, `content`, `_api_key`

## AC-SM-002
GIVEN a call to store_memory with `tokens`/`tokenizer`/`model` (previously extra params)
THEN behavior is consistent with the frozen contract (either cleanly rejected as unknown or accepted if retained — frozen contract table wins)

## AC-SM-003
GIVEN the fix
THEN full suite ≥647 passed / 1 known pre-existing; schema-registration test present
