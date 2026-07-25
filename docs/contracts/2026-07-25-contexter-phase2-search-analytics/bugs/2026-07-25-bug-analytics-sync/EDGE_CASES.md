# Edge Cases

1. **All records missing created_at** — sync should complete with zero records; no crash
2. **Some records valid, some invalid** — valid records should be synced, invalid skipped
3. **Created_at is non-empty but malformed** — let DuckDB CAST handle the error; log a warning
