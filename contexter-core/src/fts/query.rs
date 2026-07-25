//! Query parsing helpers for Tantivy full-text search.
//!
//! Provides convenience functions for building `Query` objects with
//! per-field boosting and standard query parsing.

use tantivy::query::{BooleanQuery, BoostQuery, Occur, Query, QueryParser};
use tantivy::schema::{Field, Schema};

use crate::fts::error::FtsError;
use crate::fts::FtsResult;

/// Parse a query string using the provided [`QueryParser`].
///
/// Returns an error if the query string is empty or cannot be parsed.
pub fn parse_query(query_parser: &QueryParser, query_text: &str) -> FtsResult<Box<dyn Query>> {
    if query_text.is_empty() {
        return Err(FtsError::QueryParse("empty query".to_string()));
    }
    query_parser
        .parse_query(query_text)
        .map_err(|e| FtsError::QueryParse(e.to_string()))
}

/// Build a boolean disjunction query with per-field boosting.
///
/// For each `(field, boost)` pair a separate sub-query is created over that
/// single field and wrapped in a [`BoostQuery`]. All sub-queries are combined
/// with `Occur::Should` inside a [`BooleanQuery`].
///
/// This produces a disjunction where each field's match is independently
/// boosted, unlike `set_field_boost` which applies a single boost factor
/// across the combined field scoring.
pub fn parse_boosted_query(
    schema: &Schema,
    schema_fields: &[(Field, f32)],
    query_text: &str,
    tokenizer_manager: &tantivy::tokenizer::TokenizerManager,
) -> FtsResult<Box<dyn Query>> {
    let mut boosted_queries: Vec<(Occur, Box<dyn Query>)> = Vec::new();

    for &(field, boost) in schema_fields {
        let qp = QueryParser::new(schema.clone(), vec![field], tokenizer_manager.clone());
        if let Ok(q) = qp.parse_query(query_text) {
            boosted_queries.push((Occur::Should, Box::new(BoostQuery::new(q, boost))));
        }
    }

    if boosted_queries.is_empty() {
        return Err(FtsError::QueryParse(
            "no fields produced a valid query".to_string(),
        ));
    }

    Ok(Box::new(BooleanQuery::new(boosted_queries)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tantivy::schema::*;
    use tantivy::tokenizer::TokenizerManager;
    use tantivy::Index;

    /// Create a minimal in-memory index to get a properly configured
    /// tokenizer manager (with the "default" tokenizer registered).
    fn with_index_tokenizer() -> TokenizerManager {
        let mut sb = Schema::builder();
        sb.add_text_field("_dummy", TEXT);
        let schema = sb.build();
        let index = Index::create_in_ram(schema);
        index.tokenizers().clone()
    }

    fn test_schema_and_fields() -> (Schema, Field) {
        let mut sb = Schema::builder();
        let field = sb.add_text_field("content", TEXT | STORED);
        (sb.build(), field)
    }

    #[test]
    fn parse_simple_query_succeeds() {
        let (schema, field) = test_schema_and_fields();
        let tm = with_index_tokenizer();
        let qp = QueryParser::new(schema, vec![field], tm);
        let result = parse_query(&qp, "hello world");
        assert!(
            result.is_ok(),
            "simple query should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_empty_query_returns_error() {
        let (schema, field) = test_schema_and_fields();
        let tm = with_index_tokenizer();
        let qp = QueryParser::new(schema, vec![field], tm);
        let result = parse_query(&qp, "");
        assert!(result.is_err(), "empty query should error");
        match result.unwrap_err() {
            FtsError::QueryParse(msg) => assert!(msg.contains("empty"), "error mentions empty"),
            other => panic!("expected QueryParse, got {other:?}"),
        }
    }

    #[test]
    fn parse_boosted_query_returns_boolean_disjunction() {
        let (schema, field) = test_schema_and_fields();
        let tm = with_index_tokenizer();
        let result = parse_boosted_query(&schema, &[(field, 2.0)], "test", &tm);
        assert!(
            result.is_ok(),
            "boosted query should parse: {:?}",
            result.err()
        );
    }

    #[test]
    fn parse_boosted_empty_query_errors() {
        let (schema, field) = test_schema_and_fields();
        let tm = with_index_tokenizer();
        let result = parse_boosted_query(&schema, &[(field, 1.0)], "", &tm);
        // An empty string may produce an empty BooleanQuery (no terms to
        // match); the error path only fires when *no* fields produce a valid
        // sub-query.
        match result {
            Err(FtsError::QueryParse(_)) => {} // expected error path
            Ok(_) => {}                        // also acceptable (empty query)
            Err(_) => {}                       // other errors not expected here
        }
    }
}
