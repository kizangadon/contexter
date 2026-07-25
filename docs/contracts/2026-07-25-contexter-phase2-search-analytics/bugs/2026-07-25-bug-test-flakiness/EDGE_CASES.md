# Edge Cases

1. **UUID collision** — astronomically unlikely with UUID v4; no special handling needed
2. **Cleanup failure** — if `remove_dir_all` fails, log a warning (as current code does)
3. **Long temp dir names** — UUID is 36 chars, should be well within filesystem limits
