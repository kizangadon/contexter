# Edge Cases

- `api_key` is `None` → handled before comparison, raises `MCPAuthError`
- `api_key` is empty string `""` → handled before comparison, raises `MCPAuthError`
- `CONtexTER_API_KEY` env var is not set → early return, comparison skipped
- `api_key` and `expected` are equal strings → `compare_digest` returns `True`
- `api_key` and `expected` differ → `compare_digest` returns `False`, `MCPAuthError` raised
