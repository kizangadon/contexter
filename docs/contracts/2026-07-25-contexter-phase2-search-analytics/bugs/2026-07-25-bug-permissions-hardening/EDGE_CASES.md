# Edge Cases

1. **TempDir already exists** — `set_permissions` on an existing dir should not fail; handle `set_permissions` errors gracefully (log + continue)
2. **Platform without permissions** — Windows may not support `from_mode` — use conditional compilation or fallback
3. **Nested Tantivy dir** — Tantivy may create subdirectories; ensure only the root directory has `0o700`
4. **Snapshot file overwrite** — `save_snapshot_data` writes to a new temp file then renames; apply permissions to the temp file before rename
5. **Test regression** — Changing permissions may break other tests that rely on specific permission behavior
