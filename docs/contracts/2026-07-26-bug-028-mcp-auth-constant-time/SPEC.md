# BUG-028: MCP Auth Uses Non-Constant-Time Comparison

## Problem
`mcp_tools/auth.py:55` uses `if api_key != expected:` instead of `hmac.compare_digest()`. The REST API layer (`api/deps.py:64`) already correctly uses `hmac.compare_digest()` after BUG-017.

## Fix
- Add `import hmac` to `mcp_tools/auth.py`
- Replace `if api_key != expected:` with `if not hmac.compare_digest(api_key, expected):`

## Files
- `contexter-server/src/contexter_server/mcp_tools/auth.py` — apply fix
- `contexter-server/tests/mcp/test_mcp_auth.py` — existing tests should validate the fix
