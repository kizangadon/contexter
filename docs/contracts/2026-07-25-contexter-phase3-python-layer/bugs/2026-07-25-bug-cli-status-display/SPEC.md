# Bug: CLI Status Display — Missing f-String Prefix

**Sources:** Code Reviewer P0 (Finding 1), Security LOW-03

**File:** `cli/status_commands.py` lines 43-53

**Problem:** Seven `click.echo()` calls use regular strings with `{variable.attr}` syntax that is never interpolated. CLI displays literal `{overview.total_sessions}` instead of actual values.

**Fix:** Prefix 7 affected click.echo calls with `f`. Also fix CLI exception reporting (Security LOW-03) — log full exception with `logger.exception()`, return generic message.

**Acceptance:** `contexter status` prints correct values, not literal template strings.
