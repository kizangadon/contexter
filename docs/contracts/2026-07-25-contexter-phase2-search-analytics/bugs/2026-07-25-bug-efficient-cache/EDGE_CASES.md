# Edge Cases

1. **Entry not in cache** — return None, fall through to DB query, then cache result
2. **Expired entry** — remove and return None, triggering DB query + cache refresh
3. **Fresh entry** — return cached value immediately
