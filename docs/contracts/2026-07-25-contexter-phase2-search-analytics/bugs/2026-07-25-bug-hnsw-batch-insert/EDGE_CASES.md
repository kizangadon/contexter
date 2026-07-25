# Edge Cases

1. **Empty batch** — `insert_batch` with empty input should return Ok(())
2. **Single-item batch** — should behave identically to `insert()`
3. **Mismatched dimensions** — check all embeddings have the expected dimension before building
4. **Dead code warning** — if single `insert()` delegates to batch, it may still have the full rebuild — measure and optimize
