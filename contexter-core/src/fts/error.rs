use crate::error::EngineError;

#[derive(Debug, thiserror::Error)]
pub enum FtsError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("Query parse error: {0}")]
    QueryParse(String),
    #[error("Index error: {0}")]
    IndexError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for FtsError {
    fn from(e: std::io::Error) -> Self {
        FtsError::Io(e.to_string())
    }
}

impl From<FtsError> for EngineError {
    fn from(e: FtsError) -> Self {
        EngineError::Internal(format!("fts error: {e}"))
    }
}

impl From<tantivy::TantivyError> for FtsError {
    fn from(e: tantivy::TantivyError) -> Self {
        FtsError::IndexError(e.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for FtsError {
    fn from(e: tantivy::query::QueryParserError) -> Self {
        FtsError::QueryParse(e.to_string())
    }
}
