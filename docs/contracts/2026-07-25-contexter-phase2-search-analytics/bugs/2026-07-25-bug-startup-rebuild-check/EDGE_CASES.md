# Edge Cases

1. **No vector index** — skip check if vector index is disabled
2. **No snapshot loaded** — HNSW entry count is 0; compare with L2 count
3. **Counts differ** — log warning, do not fail startup
4. **Zero counts match** — no warning needed
