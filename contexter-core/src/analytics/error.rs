use crate::error::EngineError;

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Table '{0}' does not exist. Call sync('{0}') first")]
    TableNotFound(String),
    #[error("Column family '{0}' does not exist")]
    ColumnFamilyNotFound(String),
    #[error("Query error: {0}")]
    QueryError(String),
    #[error("Sync error: {0}")]
    SyncError(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<AnalyticsError> for EngineError {
    fn from(e: AnalyticsError) -> Self {
        EngineError::Internal(format!("analytics error: {e}"))
    }
}
