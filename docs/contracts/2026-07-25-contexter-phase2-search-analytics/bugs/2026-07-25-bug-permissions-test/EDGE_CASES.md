# Edge Cases

1. **Non-Unix** — the test should be cfg-gated to Unix only
2. **Permission bits** — use `mode & 0o777 == 0o700` to check exact bits
