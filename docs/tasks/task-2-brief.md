# Task 2 — FTS entity schemas

**Files:** `contexter-core/src/fts/schema.rs`, `contexter-core/src/fts/tantivy.rs`

## Schema Changes (`fts/schema.rs`)

### Modify `EntitySchema`
- Remove `title_field: Option<Field>`
- Add `default_search_fields: Vec<(Field, f32)>`
- Add the following optional fields:
  - `name_field: Option<Field>`
  - `description_field: Option<Field>`
  - `capabilities_field: Option<Field>`
  - `category_field: Option<Field>`
  - `project_field: Option<Field>`
  - `status_field: Option<Field>`
  - `metadata_field: Option<Field>`

### Create entity-specific schema functions

**`memory_schema()`** (existing, update):
- Keep: `id`, `content` (TEXT | STORED), `tags` (STRING | STORED), `entity_type` (STRING | STORED)
- Remove: `title` field
- `default_search_fields`: `[(content_field, 1.0), (tags_field, 1.5)]`
- Set `name_field = None`, `description_field = None`, etc.

**`session_schema()`** (new):
- Fields: `id` (STRING | STORED), `content` (TEXT | STORED), `project` (STRING | STORED), `status` (STRING | STORED), `entity_type` (STRING | STORED)
- No tags field
- `default_search_fields`: `[(content_field, 1.0), (project_field, 1.0)]`
- Populate: `project_field = Some(project)`, `status_field = Some(status)`

**`agent_schema()`** (new):
- Fields: `id` (STRING | STORED), `content` (TEXT | STORED), `name` (TEXT | STORED), `description` (TEXT | STORED), `capabilities` (STRING | STORED), `status` (STRING | STORED), `entity_type` (STRING | STORED)
- `default_search_fields`: `[(content_field, 1.0), (name_field, 1.5), (description_field, 1.0), (capabilities_field, 1.0)]`
- Populate: `name_field`, `description_field`, `capabilities_field`, `status_field`

**`skill_schema()`** (new):
- Fields: `id` (STRING | STORED), `content` (TEXT | STORED), `name` (TEXT | STORED), `description` (TEXT | STORED), `category` (STRING | STORED), `entity_type` (STRING | STORED)
- `default_search_fields`: `[(content_field, 1.0), (name_field, 1.5), (description_field, 1.0), (category_field, 1.0)]`
- Populate: `name_field`, `description_field`, `category_field`

**`default_schema()`** (update):
- Remove `title_field: None`

### Update `schema_for_entity()`
```rust
pub fn schema_for_entity(entity_type: &str) -> &'static EntitySchema {
    match entity_type {
        "memory" | "memories" => get_memory_schema(),
        "session" | "sessions" => get_session_schema(),
        "agent" | "agents" => get_agent_schema(),
        "skill" | "skills" => get_skill_schema(),
        _ => get_default_schema(),
    }
}
```

### Add lazy statics for new schemas
- `SESSION_SCHEMA: OnceLock<EntitySchema>`
- `AGENT_SCHEMA: OnceLock<EntitySchema>`
- `SKILL_SCHEMA: OnceLock<EntitySchema>`

And getter functions:
- `get_session_schema()`
- `get_agent_schema()`
- `get_skill_schema()`

## Tantivy Changes (`fts/tantivy.rs`)

### Add `entity_type` field to `TantivyIndex`
```rust
pub struct TantivyIndex {
    index: Index,
    writer: RwLock<IndexWriter>,
    schema: &'static EntitySchema,
    query_parser: QueryParser,
    aliases: RwLock<HashMap<String, String>>,
    entity_type: &'static str,  // NEW
}
```

Store it in `open()` and `open_in_memory()` — set `entity_type` to the `entity_type` string parameter.

### Update `build_query_parser()`
Replace the hardcoded title:2.0 boosting with `schema.default_search_fields`:
```rust
fn build_query_parser(index: &Index, schema: &EntitySchema) -> QueryParser {
    let mut default_fields: Vec<Field> = Vec::new();
    
    for (field, _boost) in &schema.default_search_fields {
        default_fields.push(*field);
    }
    
    let mut query_parser = QueryParser::for_index(index, default_fields);
    for (field, boost) in &schema.default_search_fields {
        query_parser.set_field_boost(*field, *boost);
    }
    query_parser
}
```

### Update `index()` method
- Replace hardcoded `"memory"` for `entity_type_field` with `self.entity_type`
- Update field handling:
  - `"content"` → uses `self.schema.content_field`
  - Remove `"title"` handling entirely (no more title_field)
  - `"tags"` → uses `self.schema.tags_field`
  - Add new field handlers: `"name"`, `"description"`, `"capabilities"`, `"category"`, `"project"`, `"status"`, `"metadata"` — each looking up the corresponding `Option<Field>` and adding if `Some`
  - Use wildcard `_ => {}` for unknown fields

### Update tests

**Remove test: `test_field_boosting`** — this tests the old title boosting logic which is being removed.

**Add new schema tests in `schema.rs`:**
- `test_session_schema_fields` — verify session schema has `project_field.is_some()` and `status_field.is_some()`
- `test_agent_schema_fields` — verify agent schema has `name_field`, `description_field`, `capabilities_field`
- `test_skill_schema_fields` — verify skill schema has `name_field`, `description_field`, `category_field`
- `test_default_schema` — verify none of the new optional fields are present in default schema

**Update existing test `memory_schema_has_title_and_tags`:**
- Change `s.title_field.is_some()` to check `name_field.is_none()` (memory has no name field)
- Keep `s.tags_field.is_some()` check

**Update existing test `default_schema_has_no_title_or_tags`:**
- Remove `s.title_field.is_none()` check (no longer exists)
- Keep `s.tags_field.is_none()` check

**Update tantivy tests in `tantivy.rs`:**
- `test_field_boosting` → remove entirely (no more title field to boost)

## Verification
```bash
cargo build --workspace && cargo test --workspace
```
