# Tasks 4+5 — Cache policy + FTS title removal

**File:** `contexter-core/src/engine/memory.rs`

## Task 4 — Cache policy change in `create_memory`

Change `create_memory` from write-through (cache.store) to invalidate (cache.invalidate):

**Line 33**: Change:
```rust
self.cache.store(&key, CachedValue::Memory(memory.clone()));
```
to:
```rust
self.cache.invalidate(&key);
```

## Task 5 — Remove title from FTS index calls

### In `create_memory` (lines 50-69)

Remove the `FieldValue { field_name: "title", value: String::new() }` entry from the FTS index call.

Change from:
```rust
fts.index(
    &memory.id.to_string(),
    &[
        crate::fts::FieldValue {
            field_name: "content",
            value: memory.content.clone(),
        },
        crate::fts::FieldValue {
            field_name: "title",
            value: String::new(), // Memory has no title yet
        },
        crate::fts::FieldValue {
            field_name: "tags",
            value: tags_value,
        },
    ],
)
```

To:
```rust
fts.index(
    &memory.id.to_string(),
    &[
        crate::fts::FieldValue {
            field_name: "content",
            value: memory.content.clone(),
        },
        crate::fts::FieldValue {
            field_name: "tags",
            value: tags_value,
        },
    ],
)
```

### In `update_memory` (lines 127-153)

Same change — remove the `title` FieldValue entry from the FTS index call.

Change from:
```rust
fts.index(
    &memory.id.to_string(),
    &[
        crate::fts::FieldValue {
            field_name: "content",
            value: memory.content.clone(),
        },
        crate::fts::FieldValue {
            field_name: "title",
            value: String::new(),
        },
        crate::fts::FieldValue {
            field_name: "tags",
            value: tags_value,
        },
    ],
)
```

To:
```rust
fts.index(
    &memory.id.to_string(),
    &[
        crate::fts::FieldValue {
            field_name: "content",
            value: memory.content.clone(),
        },
        crate::fts::FieldValue {
            field_name: "tags",
            value: tags_value,
        },
    ],
)
```

## Verification
```bash
cargo build --workspace && cargo test --workspace
```
