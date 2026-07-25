# Edge Cases

1. **Schema changes** — If schema changes between calls (unlikely), the cached parser may reference stale schema. Lock step with schema version.
2. **First call** — Lazy-init the parser on first search (or in constructor).
3. **Thread safety** — QueryParser is Send + Sync; use Arc or construct once.
