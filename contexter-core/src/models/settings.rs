//! Settings and configuration types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregate storage size information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSize {
    /// Per-column-family size breakdown.
    pub per_cf: HashMap<String, u64>,
    /// Write-ahead log size.
    pub wal_size: u64,
    /// Total storage consumed.
    pub total: u64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_size_serialization() {
        let size = StorageSize {
            per_cf: HashMap::from([("default".into(), 1024u64)]),
            wal_size: 512,
            total: 1536,
        };

        let json = serde_json::to_value(&size).expect("serialize StorageSize");
        assert_eq!(json["total"], 1536);
        assert_eq!(json["perCf"]["default"], 1024);
    }
}
