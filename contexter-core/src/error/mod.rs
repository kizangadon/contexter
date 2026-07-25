//! Unified error type for the Contexter storage engine.
//!
//! All storage operations return `EngineError`, which is convertible
//! from `rocksdb::Error` and `serde_json::Error` via `From` impls.

use thiserror::Error;

/// Top-level error type for all Contexter storage engine operations.
#[derive(Debug, Error)]
pub enum EngineError {
    /// An error originating from the underlying storage layer.
    #[error("Storage error: {0}")]
    Storage(String),

    /// The requested entity was not found.
    #[error("Entity not found: {entity_type} {id}")]
    NotFound {
        /// The type name of the entity (e.g. "Session", "Memory").
        entity_type: String,
        /// The identifier that was looked up.
        id: String,
    },

    /// A validation constraint was violated.
    #[error("Validation error: {0}")]
    Validation(String),

    /// Serialisation or deserialisation failed.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Compression or decompression failed.
    #[error("Compression error: {0}")]
    Compression(String),

    /// An error in the caching layer.
    #[error("Cache error: {0}")]
    Cache(String),

    /// An unexpected internal error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// An invalid configuration was supplied.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// A feature that has not been implemented yet (Phase 2 placeholder).
    #[error("Not implemented: {0}")]
    Unimplemented(String),

    /// An operation was attempted on a disabled tier or unsupported feature.
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
}

/// Convenience alias for `Result<T, EngineError>`.
pub type EngineResult<T> = Result<T, EngineError>;

impl EngineError {
    /// Returns a sanitized version of the error suitable for network transmission.
    ///
    /// Strips entity IDs and internal details from error messages to prevent
    /// information disclosure (e.g. leaking `NotFound` entity IDs, stack traces,
    /// or internal paths).
    pub fn sanitized(&self) -> String {
        match self {
            EngineError::NotFound { .. } => "Resource not found".to_string(),
            EngineError::Validation(msg) => format!("Validation error: {msg}"),
            EngineError::Storage(_) => "Storage error".to_string(),
            EngineError::Serialization(_) => "Serialization error".to_string(),
            EngineError::Compression(_) => "Compression error".to_string(),
            EngineError::Cache(_) => "Cache error".to_string(),
            EngineError::Internal(_) => "Internal error".to_string(),
            EngineError::InvalidConfig(msg) => format!("Invalid configuration: {msg}"),
            EngineError::Unimplemented(ref feature) => {
                format!("Not implemented: {feature}")
            }
            EngineError::UnsupportedOperation(ref msg) => {
                format!("Unsupported operation: {msg}")
            }
        }
    }
}

impl From<rocksdb::Error> for EngineError {
    fn from(err: rocksdb::Error) -> Self {
        EngineError::Storage(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that every variant's Display output is meaningful.
    #[test]
    fn engine_error_display_storage() {
        let err = EngineError::Storage("disk full".into());
        assert_eq!(err.to_string(), "Storage error: disk full");
    }

    #[test]
    fn engine_error_display_not_found() {
        let err = EngineError::NotFound {
            entity_type: "Session".into(),
            id: "abc-123".into(),
        };
        assert_eq!(err.to_string(), "Entity not found: Session abc-123");
    }

    #[test]
    fn engine_error_display_validation() {
        let err = EngineError::Validation("missing field".into());
        assert_eq!(err.to_string(), "Validation error: missing field");
    }

    #[test]
    fn engine_error_display_serialization() {
        let inner = serde_json::from_str::<i32>("not-a-number").unwrap_err();
        let err = EngineError::Serialization(inner);
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn engine_error_display_compression() {
        let err = EngineError::Compression("invalid header".into());
        assert_eq!(err.to_string(), "Compression error: invalid header");
    }

    #[test]
    fn engine_error_display_cache() {
        let err = EngineError::Cache("capacity exceeded".into());
        assert_eq!(err.to_string(), "Cache error: capacity exceeded");
    }

    #[test]
    fn engine_error_display_internal() {
        let err = EngineError::Internal("unexpected null".into());
        assert_eq!(err.to_string(), "Internal error: unexpected null");
    }

    #[test]
    fn engine_error_display_invalid_config() {
        let err = EngineError::InvalidConfig("embedding_dim must be >= 1".into());
        assert_eq!(
            err.to_string(),
            "Invalid configuration: embedding_dim must be >= 1"
        );
    }

    /// Verify that NotFound renders with both entity_type and id.
    #[test]
    fn not_found_renders_correctly() {
        let err = EngineError::NotFound {
            entity_type: "Memory".into(),
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Memory"), "should contain entity type");
        assert!(msg.contains("550e8400"), "should contain id");
    }

    /// Verify conversion from rocksdb::Error compiles.
    #[test]
    fn from_rocksdb_error_conversion_compiles() {
        // Compile-time check: EngineError implements From<rocksdb::Error>.
        fn assert_from()
        where
            EngineError: From<rocksdb::Error>,
        {
        }
        _ = assert_from;
    }

    /// Verify conversion from serde_json::Error via #[from].
    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("").unwrap_err();
        let engine_err: EngineError = json_err.into();
        assert!(matches!(engine_err, EngineError::Serialization(_)));
    }

    /// Verify that `sanitized()` strips sensitive details.
    #[test]
    fn sanitized_not_found_strips_ids() {
        let err = EngineError::NotFound {
            entity_type: "Session".into(),
            id: "550e8400-e29b-41d4-a716-446655440000".into(),
        };
        let sanitized = err.sanitized();
        assert_eq!(sanitized, "Resource not found");
        // Ensure the UUID is NOT present in the sanitized output.
        assert!(!sanitized.contains("550e8400"));
        assert!(!sanitized.contains("Session"));
    }

    #[test]
    fn sanitized_validation_preserves_message() {
        let err = EngineError::Validation("missing field".into());
        assert_eq!(err.sanitized(), "Validation error: missing field");
    }

    #[test]
    fn sanitized_storage_is_generic() {
        let err = EngineError::Storage("disk full at /var/data".into());
        assert_eq!(err.sanitized(), "Storage error");
    }

    #[test]
    fn sanitized_serialization_is_generic() {
        let inner = serde_json::from_str::<i32>("x").unwrap_err();
        let err = EngineError::Serialization(inner);
        assert_eq!(err.sanitized(), "Serialization error");
    }

    #[test]
    fn sanitized_compression_is_generic() {
        let err = EngineError::Compression("lz4 header corrupt".into());
        assert_eq!(err.sanitized(), "Compression error");
    }

    #[test]
    fn sanitized_cache_is_generic() {
        let err = EngineError::Cache("capacity exceeded".into());
        assert_eq!(err.sanitized(), "Cache error");
    }

    #[test]
    fn sanitized_internal_is_generic() {
        let err = EngineError::Internal("unexpected null in thread pool".into());
        assert_eq!(err.sanitized(), "Internal error");
    }

    #[test]
    fn sanitized_invalid_config_preserves_message() {
        let err = EngineError::InvalidConfig("embedding_dim must be >= 1".into());
        assert_eq!(
            err.sanitized(),
            "Invalid configuration: embedding_dim must be >= 1"
        );
    }

    #[test]
    fn engine_error_display_unsupported_operation() {
        let err = EngineError::UnsupportedOperation("L3 tier disabled".into());
        assert_eq!(err.to_string(), "Unsupported operation: L3 tier disabled");
    }

    #[test]
    fn sanitized_unsupported_operation_preserves_message() {
        let err = EngineError::UnsupportedOperation("vector index not enabled".into());
        assert_eq!(err.sanitized(), "Unsupported operation: vector index not enabled");
    }
}
