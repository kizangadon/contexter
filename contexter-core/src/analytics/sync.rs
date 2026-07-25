//! Table schemas for RocksDB → DuckDB data synchronisation.
//!
//! Defines the schema of each synchronisable dataset and the column family
//! each table maps to in RocksDB.

/// Metadata about a synced table.
#[derive(Debug, Clone)]
pub struct TableSchema {
    /// Logical table name used in DuckDB.
    pub name: &'static str,
    /// Column definitions as `(name, type)` pairs (e.g. `("id", "VARCHAR")`).
    pub columns: Vec<(&'static str, &'static str)>,
    /// The RocksDB column family from which this table is populated.
    pub source_cf: &'static str,
}

/// Predefined table schemas for each synchronisable dataset.
pub fn table_schemas() -> Vec<TableSchema> {
    vec![
        TableSchema {
            name: "telemetry",
            columns: vec![
                ("id", "VARCHAR"),
                ("event_type", "VARCHAR"),
                ("scope", "VARCHAR"),
                ("value", "DOUBLE"),
                ("ts", "TIMESTAMP"),
            ],
            source_cf: "telemetry",
        },
        TableSchema {
            name: "memories",
            columns: vec![
                ("id", "VARCHAR"),
                ("session_id", "VARCHAR"),
                ("memory_type", "VARCHAR"),
                ("tags", "VARCHAR"),
                ("created_at", "TIMESTAMP"),
            ],
            source_cf: "memory_items",
        },
        TableSchema {
            name: "sessions",
            columns: vec![
                ("id", "VARCHAR"),
                ("project", "VARCHAR"),
                ("agent_id", "VARCHAR"),
                ("status", "VARCHAR"),
                ("turn_count", "BIGINT"),
                ("duration_ms", "BIGINT"),
                ("created_at", "TIMESTAMP"),
                ("last_active", "TIMESTAMP"),
            ],
            source_cf: "sessions",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_schemas_defined() {
        let schemas = table_schemas();
        assert_eq!(schemas.len(), 3, "expected three table schemas");

        // Check sessions schema
        let sessions = schemas.iter().find(|s| s.name == "sessions").unwrap();
        assert_eq!(sessions.source_cf, "sessions");
        assert!(sessions.columns.iter().any(|(n, _)| n == &"id"));
        assert!(sessions.columns.iter().any(|(n, _)| n == &"project"));
        assert!(sessions.columns.iter().any(|(n, _)| n == &"turn_count"));
        assert!(sessions.columns.iter().any(|(n, _)| n == &"duration_ms"));

        // Check memories schema
        let memories = schemas.iter().find(|s| s.name == "memories").unwrap();
        assert_eq!(memories.source_cf, "memory_items");
        assert!(memories.columns.iter().any(|(n, _)| n == &"memory_type"));
        assert!(memories.columns.iter().any(|(n, _)| n == &"session_id"));

        // Check telemetry schema
        let telemetry = schemas.iter().find(|s| s.name == "telemetry").unwrap();
        assert_eq!(telemetry.source_cf, "telemetry");
        assert!(telemetry.columns.iter().any(|(n, _)| n == &"event_type"));
        assert!(telemetry.columns.iter().any(|(n, _)| n == &"value"));
    }

    #[test]
    fn test_table_schemas_have_columns() {
        for schema in table_schemas() {
            assert!(
                !schema.columns.is_empty(),
                "schema '{}' must have at least one column",
                schema.name
            );
            // Every schema must have an id column
            assert!(
                schema.columns.iter().any(|(n, _)| n == &"id"),
                "schema '{}' must have an 'id' column",
                schema.name
            );
        }
    }
}
