# Bug: FullTextSearch Trait — Schemas, Path, TextContent, Indexing Gaps

**Severity:** HIGH  
**Root Cause:** `FullTextSearch` trait in `engine/mod.rs` is unimplemented; Tantivy schemas are incomplete (only text/content fields, missing title/tags); path/alias not wired; TextContent trait not implemented.

## Requirements

### REQ-FIX-001: Implement FullTextSearch trait on TantivyIndex
Implement `search()`, `index()`, `delete()`, `schema()` on `TantivyIndex` (already exists in `fts/tantivy.rs` — wire them).

### REQ-FIX-002: Add title and tags fields to Tantivy schema
Add `TITLE_FIELD` and `TAGS_FIELD` to the schema builder in `TantivyIndex::new()`.

### REQ-FIX-003: Implement TextContent on MemoryEntry
Implement `trait TextContent` with `fn text_content(&self) -> String` that returns concatenated title + content + tags for FTS indexing.

### REQ-FIX-004: Wire Tantivy path from EngineConfig
`Engine::with_config()` must pass the `tantivy_path` to `TantivyIndex::new(path)` correctly (currently hardcoded).

### REQ-FIX-005: Add alias support to TantivyIndex
Implement `add_alias(name)`, `list_aliases()`, `switch_index(name)` on TantivyIndex to support index switching.
