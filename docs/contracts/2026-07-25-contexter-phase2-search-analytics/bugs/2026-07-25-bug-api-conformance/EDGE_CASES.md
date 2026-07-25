# Edge Cases

1. **Backward compatibility** — If any external code references the old field names, it will break. This is expected — the design is the contract.
2. **FTS schema migration** — Existing Tantivy indexes with the old schema may need reindexing. Handle gracefully.
3. **text_weight semantics** — Ensure `text_weight + vector_weight == 1.0` invariant is maintained.
