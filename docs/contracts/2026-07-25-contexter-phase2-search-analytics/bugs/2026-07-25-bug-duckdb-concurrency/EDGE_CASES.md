# Edge Cases

1. **First sync with no last-sync timestamp** — process all existing data
2. **Concurrent reads during write** — read connection should see consistent state (DuckDB handles this)
3. **Transactional integrity** — if upsert fails partway, partial data should not corrupt analytics
4. **Backward compat on StorageBackend trait** — existing implementations may need default method impl
