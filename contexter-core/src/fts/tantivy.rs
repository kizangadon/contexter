//! Tantivy-backed full-text search implementation.
//!
//! Wraps a tantivy [`Index`] and provides the [`FullTextSearch`] trait
//! interface. Per-field boosts are defined by each entity type's
//! [`EntitySchema::default_search_fields`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{Index, IndexWriter, ReloadPolicy, TantivyDocument};

use crate::fts::error::FtsError;
use crate::fts::schema::{schema_for_entity, EntitySchema};
use crate::fts::{FieldValue, FtsResult, FullTextSearch};

/// Tantivy-backed full-text search index.
///
/// # Thread safety
///
/// The inner [`IndexWriter`] is protected by an `RwLock` so that multiple
/// readers can search concurrently while writes are serialised. The
/// cached [`QueryParser`] is `Send + Sync` and safe to share across
/// threads.
pub struct TantivyIndex {
    index: Index,
    writer: RwLock<IndexWriter>,
    schema: &'static EntitySchema,
    /// Cached query parser (constructed once, reused across `search()` calls).
    query_parser: QueryParser,
    /// Simple in-memory alias map: alias name → entity type.
    /// Used for `add_alias` / `list_aliases` / `switch_index` support.
    aliases: RwLock<HashMap<String, String>>,
}

impl TantivyIndex {
    /// Open (or create) a Tantivy index at the given directory path.
    ///
    /// The directory and any missing parents are created automatically.
    pub fn open(path: &Path, entity_type: &str) -> FtsResult<Self> {
        let entity_schema = schema_for_entity(entity_type);
        let schema = &entity_schema.schema;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| FtsError::Io(format!("create dir: {e}")))?;
        }

        let index = Index::create_in_dir(path, schema.clone())
            .map_err(|e| FtsError::Io(format!("create index: {e}")))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)) {
                eprintln!("Warning: could not set 0o700 on index dir: {e}");
            }
        }

        let writer = index
            .writer(50_000_000) // 50 MB memory budget
            .map_err(|e| FtsError::IndexError(e.to_string()))?;

        // Build the QueryParser once with field boosts.
        let query_parser = Self::build_query_parser(&index, entity_schema);

        Ok(Self {
            index,
            writer: RwLock::new(writer),
            schema: entity_schema,
            query_parser,
            aliases: RwLock::new(HashMap::new()),
        })
    }

    /// Create an in-memory Tantivy index (useful for testing).
    pub fn open_in_memory(entity_type: &str) -> FtsResult<Self> {
        let entity_schema = schema_for_entity(entity_type);
        let schema = &entity_schema.schema;

        let index = Index::create_in_ram(schema.clone());

        let writer = index
            .writer(50_000_000)
            .map_err(|e| FtsError::IndexError(e.to_string()))?;

        let query_parser = Self::build_query_parser(&index, entity_schema);

        Ok(Self {
            index,
            writer: RwLock::new(writer),
            schema: entity_schema,
            query_parser,
            aliases: RwLock::new(HashMap::new()),
        })
    }

    /// Build a [`QueryParser`] with default field boosts from the schema.
    ///
    /// Boosts are read from [`EntitySchema::default_search_fields`] so each
    /// entity type gets appropriate weighting automatically.
    fn build_query_parser(index: &Index, schema: &EntitySchema) -> QueryParser {
        let default_fields: Vec<Field> =
            schema.default_search_fields.iter().map(|(f, _)| *f).collect();

        let mut query_parser = QueryParser::for_index(index, default_fields);
        for (field, boost) in &schema.default_search_fields {
            query_parser.set_field_boost(*field, *boost);
        }
        query_parser
    }

    /// Register an alias name pointing to this index.
    ///
    /// Returns an error if `name` is empty.
    pub fn add_alias(&self, name: &str) -> FtsResult<()> {
        if name.is_empty() {
            return Err(FtsError::Internal("alias name must not be empty".into()));
        }
        self.aliases
            .write()
            .map_err(|e| FtsError::Internal(e.to_string()))?
            .insert(name.to_string(), "memory".to_string());
        Ok(())
    }

    /// Return all registered alias names.
    pub fn list_aliases(&self) -> Vec<String> {
        self.aliases
            .read()
            .map(|guard| guard.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Switch to a different index identified by `name`.
    ///
    /// Currently returns an error — full index-switching requires wiring a
    /// separate index path and is reserved for future use.
    pub fn switch_index(&self, name: &str) -> FtsResult<()> {
        let aliases = self
            .aliases
            .read()
            .map_err(|e| FtsError::Internal(e.to_string()))?;
        if !aliases.contains_key(name) {
            return Err(FtsError::Internal(format!("alias '{}' not found", name)));
        }
        // Stub: actual index switching would reload from the alias path.
        Ok(())
    }
}

impl FullTextSearch for TantivyIndex {
    fn index(&self, doc_id: &str, fields: &[FieldValue]) -> FtsResult<()> {
        let mut doc = TantivyDocument::new();
        doc.add_text(self.schema.id_field, doc_id);
        doc.add_text(self.schema.entity_type_field, "memory");

        for fv in fields {
            match fv.field_name {
                "content" => doc.add_text(self.schema.content_field, &fv.value),
                "tags" => {
                    if let Some(tf) = self.schema.tags_field {
                        doc.add_text(tf, &fv.value);
                    }
                }
                "name" => {
                    if let Some(f) = self.schema.name_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                "description" => {
                    if let Some(f) = self.schema.description_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                "capabilities" => {
                    if let Some(f) = self.schema.capabilities_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                "category" => {
                    if let Some(f) = self.schema.category_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                "project" => {
                    if let Some(f) = self.schema.project_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                "status" => {
                    if let Some(f) = self.schema.status_field {
                        doc.add_text(f, &fv.value);
                    }
                }
                _ => {}
            }
        }

        let writer = self.writer.write().unwrap_or_else(|e| e.into_inner());
        writer
            .add_document(doc)
            .map_err(|e| FtsError::IndexError(e.to_string()))?;

        Ok(())
    }

    fn search(&self, query_text: &str, limit: usize) -> FtsResult<Vec<(String, f32)>> {
        if query_text.is_empty() {
            return Ok(Vec::new());
        }

        let reader = self
            .index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e| FtsError::IndexError(e.to_string()))?;

        let searcher = reader.searcher();

        // Use the cached QueryParser (built once in the constructor).
        let query = self
            .query_parser
            .parse_query(query_text)
            .map_err(|e| FtsError::QueryParse(e.to_string()))?;

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| FtsError::IndexError(e.to_string()))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_address) in top_docs {
            let stored_doc = searcher
                .doc::<TantivyDocument>(doc_address)
                .map_err(|e| FtsError::IndexError(e.to_string()))?;

            let id_val = stored_doc
                .get_first(self.schema.id_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !id_val.is_empty() {
                results.push((id_val, score));
            }
        }

        Ok(results)
    }

    fn delete(&self, doc_id: &str) -> FtsResult<()> {
        let writer = self.writer.write().unwrap_or_else(|e| e.into_inner());
        let term = tantivy::Term::from_field_text(self.schema.id_field, doc_id);
        writer.delete_term(term);
        Ok(())
    }

    fn flush(&self) -> FtsResult<()> {
        let mut writer = self.writer.write().unwrap_or_else(|e| e.into_inner());
        writer
            .commit()
            .map_err(|e| FtsError::IndexError(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fts::FieldValue;

    /// Helper: create an in-memory TantivyIndex for the "memory" entity type.
    fn setup() -> TantivyIndex {
        TantivyIndex::open_in_memory("memory").expect("in-memory index")
    }

    // -----------------------------------------------------------------------
    // Basic index & search
    // -----------------------------------------------------------------------

    #[test]
    fn test_index_and_search() {
        let index = setup();

        index
            .index(
                "doc-1",
                &[FieldValue {
                    field_name: "content",
                    value: "the quick brown fox jumps over the lazy dog".into(),
                }],
            )
            .expect("index");
        index.flush().expect("flush");

        let results = index.search("fox", 10).expect("search");
        assert_eq!(results.len(), 1, "should find one result");
        assert_eq!(results[0].0, "doc-1", "should return the correct doc id");
        assert!(results[0].1 > 0.0, "BM25 score should be positive");
    }

    #[test]
    fn test_search_no_match() {
        let index = setup();

        index
            .index(
                "doc-1",
                &[FieldValue {
                    field_name: "content",
                    value: "hello world".into(),
                }],
            )
            .expect("index");
        index.flush().expect("flush");

        let results = index.search("nonexistent", 10).expect("search");
        assert!(results.is_empty(), "non-matching query should return empty");
    }

    // -----------------------------------------------------------------------
    // Delete
    // -----------------------------------------------------------------------

    #[test]
    fn test_delete_doc() {
        let index = setup();

        index
            .index(
                "doc-1",
                &[FieldValue {
                    field_name: "content",
                    value: "something to find".into(),
                }],
            )
            .expect("index");
        index.flush().expect("flush");

        // Confirm it is findable.
        let results = index.search("find", 10).expect("search");
        assert_eq!(results.len(), 1);

        // Delete and re-search.
        index.delete("doc-1").expect("delete");
        index.flush().expect("flush");

        let results = index.search("find", 10).expect("search");
        assert!(results.is_empty(), "doc should not be found after delete");
    }

    // -----------------------------------------------------------------------
    // Phrase search
    // -----------------------------------------------------------------------

    #[test]
    fn test_phrase_search() {
        let index = setup();

        index
            .index(
                "doc-1",
                &[FieldValue {
                    field_name: "content",
                    value: "the quick brown fox jumps over the lazy dog".into(),
                }],
            )
            .expect("index");
        index.flush().expect("flush");

        // Tantivy supports phrase queries via double quotes.
        let results = index.search("\"quick brown\"", 10).expect("search");
        assert_eq!(results.len(), 1, "phrase query should match the document");

        // Phrase that doesn't appear should not match.
        let results = index.search("\"brown quick\"", 10).expect("search");
        assert!(results.is_empty(), "reversed phrase should not match");
    }

    // -----------------------------------------------------------------------
    // Field boosting
    // -----------------------------------------------------------------------

    #[test]
    fn test_field_boosting() {
        // Use the agent schema which has both content (TEXT, 1.0) and
        // name (TEXT, 2.0) as default search fields.
        let index = TantivyIndex::open_in_memory("agent").expect("in-memory agent index");

        // Doc A matches "rust" in content only (boost 1.0).
        index
            .index(
                "doc-content",
                &[
                    FieldValue {
                        field_name: "content",
                        value: "I like rust programming".into(),
                    },
                    FieldValue {
                        field_name: "name",
                        value: "unrelated".into(),
                    },
                ],
            )
            .expect("index");

        // Doc B matches "rust" in name (boosted 2.0× vs content 1.0×).
        index
            .index(
                "doc-name",
                &[
                    FieldValue {
                        field_name: "content",
                        value: "about something else".into(),
                    },
                    FieldValue {
                        field_name: "name",
                        value: "rust is great".into(),
                    },
                ],
            )
            .expect("index");

        index.flush().expect("flush");

        let results = index.search("rust", 10).expect("search");

        // The name-match doc should rank higher.
        assert!(results.len() >= 2, "both documents should match");

        let top_doc = &results[0];
        assert_eq!(
            top_doc.0, "doc-name",
            "name-match should rank higher than content-only match (boost 2.0 vs 1.0)"
        );
    }

    // -----------------------------------------------------------------------
    // Empty query
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_query() {
        let index = setup();

        index
            .index(
                "doc-1",
                &[FieldValue {
                    field_name: "content",
                    value: "some content".into(),
                }],
            )
            .expect("index");
        index.flush().expect("flush");

        let results = index.search("", 10).expect("search");
        assert!(
            results.is_empty(),
            "empty query should return empty results"
        );
    }

    // -----------------------------------------------------------------------
    // Flush persistence
    // -----------------------------------------------------------------------

    #[test]
    fn test_flush_persistence() {
        // Use a temp directory so we can verify on-disk persistence.
        let dir = tempfile::TempDir::new().expect("temp dir");

        let index = TantivyIndex::open(dir.path(), "memory").expect("open index");

        index
            .index(
                "persist-doc",
                &[FieldValue {
                    field_name: "content",
                    value: "this should be persisted".into(),
                }],
            )
            .expect("index");

        // Before flush the new reader might not see the doc.
        // After flush it must be visible.
        index.flush().expect("flush");

        // Search on the same index (new reader picks up committed data).
        let results = index.search("persisted", 10).expect("search");
        assert_eq!(results.len(), 1, "committed doc must be searchable");
        assert_eq!(results[0].0, "persist-doc");
    }

    // -----------------------------------------------------------------------
    // Aliases
    // -----------------------------------------------------------------------

    #[test]
    fn add_alias_and_list_aliases() {
        let index = setup();

        index.add_alias("prod").expect("add alias");
        index.add_alias("staging").expect("add alias");

        let aliases = index.list_aliases();
        assert!(aliases.contains(&"prod".to_string()));
        assert!(aliases.contains(&"staging".to_string()));
    }

    #[test]
    fn add_empty_alias_returns_error() {
        let index = setup();
        let result = index.add_alias("");
        assert!(result.is_err(), "empty alias should error");
    }

    #[test]
    fn switch_to_existing_alias_succeeds() {
        let index = setup();
        index.add_alias("prod").expect("add alias");
        // switch_index is a stub — it only validates the alias exists
        let result = index.switch_index("prod");
        assert!(result.is_ok(), "switching to existing alias should succeed");
    }

    #[test]
    fn switch_to_nonexistent_alias_returns_error() {
        let index = setup();
        let result = index.switch_index("nonexistent");
        assert!(
            result.is_err(),
            "switching to non-existent alias should error"
        );
    }

    #[test]
    fn list_aliases_returns_empty_when_none_added() {
        let index = setup();
        let aliases = index.list_aliases();
        assert!(aliases.is_empty(), "no aliases should be registered");
    }
}
