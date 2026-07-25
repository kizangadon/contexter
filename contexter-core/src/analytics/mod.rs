//! Analytics aggregation (L5 tier).
//!
//! Provides an in-memory DuckDB-backed analytics engine that synchronises
//! data from RocksDB on demand and supports predefined analytical queries:
//! session counts, memory counts, telemetry aggregation, efficiency scores,
//! and metric correlation.

pub mod duckdb;
pub mod error;
pub mod queries;
pub mod sync;

pub use duckdb::DuckDbEngine;

/// Convenience alias for analytics results.
pub type AnalyticsResult<T> = Result<T, error::AnalyticsError>;

/// A runtime value for analytics query parameters and results.
///
/// DuckDB columns are dynamically typed and may be null, numeric, boolean,
/// or text. This enum provides a uniform representation across all query
/// result rows.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// SQL `NULL`.
    Null,
    /// SQL `BOOLEAN`.
    Bool(bool),
    /// SQL `INTEGER`, `BIGINT`, `UBIGINT`.
    Int(i64),
    /// SQL `FLOAT`, `DOUBLE`.
    Float(f64),
    /// SQL `VARCHAR`, `TEXT`, or any other string-like column.
    Text(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "NULL"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::Text(t) => write!(f, "{t}"),
        }
    }
}

/// Analytics engine trait.
///
/// Implementors provide aggregated queries over stored data for usage
/// statistics and time-series analysis. Data is synchronised from the
/// primary storage layer (RocksDB) into an in-memory DuckDB instance
/// before queries are executed.
pub trait AnalyticsEngine: Send + Sync {
    /// Execute an SQL query with the given parameters and return the result
    /// rows. Each outer `Vec` is a row; each inner `Vec` is the columnar
    /// values for that row.
    fn query(&self, sql: &str, params: &[Value]) -> AnalyticsResult<Vec<Vec<Value>>>;

    /// Synchronise a single column family (table) from RocksDB into the
    /// analytics engine's in-memory storage.
    fn sync(&self, cf_name: &str) -> AnalyticsResult<()>;

    /// Synchronise **all** known column families into the analytics engine.
    fn sync_all(&self) -> AnalyticsResult<()>;

}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that Value variants display correctly.
    #[test]
    fn test_value_display() {
        assert_eq!(Value::Null.to_string(), "NULL");
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Int(42).to_string(), "42");
        assert_eq!(Value::Float(std::f64::consts::PI).to_string(), "3.141592653589793");
        assert_eq!(Value::Text("hello".into()).to_string(), "hello");
    }

    /// Verify that Value cloning and partial equality work.
    #[test]
    fn test_value_clone_eq() {
        let a = Value::Int(10);
        let b = a.clone();
        assert_eq!(a, b);

        let c = Value::Text("foo".into());
        let d = Value::Text("bar".into());
        assert_ne!(c, d);
    }
}
