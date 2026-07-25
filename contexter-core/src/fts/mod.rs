//! Full-text search index using tantivy.
//!
//! Provides the [`FullTextSearch`] trait and a tantivy-backed implementation
//! ([`TantivyIndex`]) for BM25 keyword search (L4 tier).

pub mod error;
pub mod query;
pub mod schema;
pub mod tantivy;

pub use tantivy::TantivyIndex;

pub type FtsResult<T> = Result<T, error::FtsError>;

/// A named field value for indexing.
///
/// Each entry pairs a field name (matching one of the fields defined in
/// [`schema::EntitySchema`]) with its text content.
#[derive(Debug, Clone)]
pub struct FieldValue {
    pub field_name: &'static str,
    pub value: String,
}

/// Provides the plain-text representation of an entity for FTS indexing.
///
/// Implementors return a single string that captures all searchable text
/// content (body, title, tags, etc.) so that consumers can index it into
/// a full-text search engine without knowing the entity's internal shape.
pub trait TextContent {
    /// Concatenated text content suitable for full-text indexing.
    fn text_content(&self) -> String;
}

/// Full-text search index trait.
///
/// Implementations provide indexed search over stored text fields using
/// BM25 ranking. All methods fallible — see [`FtsError`] for failure modes.
pub trait FullTextSearch: Send + Sync {
    /// Index a document with the given field values.
    fn index(&self, doc_id: &str, fields: &[FieldValue]) -> FtsResult<()>;

    /// Search the index and return up to `limit` `(doc_id, score)` pairs.
    fn search(&self, query: &str, limit: usize) -> FtsResult<Vec<(String, f32)>>;

    /// Delete a document by its doc ID.
    fn delete(&self, doc_id: &str) -> FtsResult<()>;

    /// Flush / commit pending writes so they are visible to new readers.
    fn flush(&self) -> FtsResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that `FullTextSearch` is object-safe (required for
    /// `Option<Arc<dyn FullTextSearch>>` in `Engine`).
    #[test]
    fn trait_is_object_safe() {
        fn _assert(_: &dyn FullTextSearch) {}
    }

    /// Verify that `TextContent` is object-safe.
    #[test]
    fn text_content_trait_is_object_safe() {
        fn _assert(_: &dyn TextContent) {}
    }
}
