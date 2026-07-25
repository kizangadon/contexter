# Acceptance Criteria — Bug-FTS

- AC-01: Tantivy schema includes `title`, `content`, `tags` fields
- AC-02: `MemoryEntry` implements `TextContent` trait (title + content + tags concatenated)
- AC-03: `TantivyIndex::new()` uses the path argument (not hardcoded)
- AC-04: `TantivyIndex.index(entry)` indexes title + content + tags as text
- AC-05: `TantivyIndex.search(query)` returns results based on title, content, tags
- AC-06: `add_alias()` / `list_aliases()` / `switch_index()` are implemented (even as stubs)
- AC-07: All existing tests continue to pass
