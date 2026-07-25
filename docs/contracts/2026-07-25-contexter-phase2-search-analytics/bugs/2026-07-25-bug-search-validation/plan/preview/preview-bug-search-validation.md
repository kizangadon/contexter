# Design Preview — Bug-Search-Validation

## Fix Plan
In `engine/search.rs`, add validation at the top of `hybrid_search()`:
```rust
let vector_weight = query.vector_weight.clamp(0.0, 1.0);
let limit = match query.limit {
    0 => 10,          // default
    n if n > 1000 => 1000,
    n => n,
};
let sort_field = query.sort_field.as_ref()
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .map(|s| s.to_string());
```
Add 4+ unit tests in the search module's test mod.
