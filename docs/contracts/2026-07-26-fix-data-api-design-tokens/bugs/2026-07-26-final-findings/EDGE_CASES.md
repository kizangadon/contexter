# Edge Cases

- Search endpoint: session results don't have `embedding` field, so the `if k != 'embedding'` filter is a no-op for sessions
- Search endpoint: if `r` is empty dict, the filter produces empty dict — no crash
- UTC coercion: timezone-aware datetimes must pass through unchanged
- Status: only 'done' is normalized; 'active', 'paused', 'completed', 'archived' pass through unchanged
