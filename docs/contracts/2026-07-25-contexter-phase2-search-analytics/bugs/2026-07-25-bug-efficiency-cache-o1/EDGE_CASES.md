# Edge Cases

1. **Entry not present** — return None, query DB, cache result
2. **Entry expired** — remove single entry, return None, query DB, cache result
3. **Entry fresh** — return cached value immediately
