# Task 1 — Rename HybridSearchQuery fields

**File:** `contexter-core/src/engine/search.rs`

## Changes

### Struct field renames
- `text_query` → `query_text`
- `vector_query` → `query_vector`
- `limit` → `top_k`
- Remove `sort_field: Option<String>` entirely
- Remove `agent_id: Option<Uuid>` entirely
- Add `text_weight: f32` (default 0.5)

### Updated Default impl
```rust
impl Default for HybridSearchQuery {
    fn default() -> Self {
        Self {
            query_text: None,
            query_vector: None,
            text_weight: 0.5,
            vector_weight: 0.5,
            top_k: 20,
            memory_type: None,
            tags: None,
            session_id: None,
        }
    }
}
```

### RRF weight logic
The RRF weighting currently computes `1.0 - vector_weight` for the text side. Remove this computation and use `text_weight` directly instead. Update the error message on line 118 to say `query_text`/`query_vector` instead of `text_query`/`vector_query`.

### Remove sort_field handling (lines 135-141)
Remove the entire block that checks `query.sort_field` for empty/whitespace.

### Remove agent_id filtering (lines 290-293)
Remove the `agent_id` filter check from the in-memory filtering section.

### Update test call sites
All tests using `HybridSearchQuery` must use the new field names:
- `text_query: Some(...)` → `query_text: Some(...)`
- `vector_query: Some(...)` → `query_vector: Some(...)`
- `limit: N` → `top_k: N`
- `sort_field: ...` → remove entirely

### Remove 3 sort_field tests
- `test_hybrid_search_sort_field_empty`
- `test_hybrid_search_sort_field_whitespace`
- `test_hybrid_search_sort_field_none`

### Update Error message line 118
Change error string from `"Hybrid search requires text_query, vector_query, or both"` to `"Hybrid search requires query_text, query_vector, or both"` (already semantic match since this is error text, but update for consistency).

## Verification
```bash
cargo build --workspace && cargo test --workspace
```
