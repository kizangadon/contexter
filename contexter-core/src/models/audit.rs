//! Audit entity — recorded audit-log entries.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input data for appending a new audit-log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAuditEntry {
    /// Action performed (e.g. "create_session", "update_memory").
    pub action: String,
    /// Type of entity affected.
    pub entity_type: String,
    /// Identifier of the affected entity.
    pub entity_id: String,
    /// Optional actor that performed the action.
    pub actor: Option<String>,
    /// Optional JSON summarising what changed.
    pub summary: Option<serde_json::Value>,
}

/// A recorded audit-log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    /// Unique entry identifier.
    pub id: Uuid,
    /// Action performed.
    pub action: String,
    /// Type of entity affected.
    pub entity_type: String,
    /// Identifier of the affected entity.
    pub entity_id: String,
    /// Optional actor that performed the action.
    pub actor: Option<String>,
    /// Optional JSON summarising what changed.
    pub summary: Option<serde_json::Value>,
    /// Arbitrary key-value metadata associated with this entry.
    pub metadata: HashMap<String, String>,
    /// Timestamp when the entry was recorded.
    pub created_at: DateTime<Utc>,
}

/// Criteria for filtering audit-log queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditFilter {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub actor: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            entity_type: None,
            entity_id: None,
            actor: None,
            limit: 100,
            offset: 0,
        }
    }
}

fn default_limit() -> u64 {
    100
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Verify AuditEntry round-trip.
    #[test]
    fn audit_entry_round_trip() {
        let entry = AuditEntry {
            id: Uuid::now_v7(),
            action: "create".into(),
            entity_type: "Session".into(),
            entity_id: "abc-123".into(),
            actor: Some("user-1".into()),
            summary: Some(serde_json::json!({"status": "active"})),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&entry).expect("serialize AuditEntry");
        assert_eq!(json["action"], "create");
        assert_eq!(json["entityType"], "Session");
        assert_eq!(json["entityId"], "abc-123");

        let deserialized: AuditEntry =
            serde_json::from_value(json).expect("deserialize AuditEntry");
        assert_eq!(deserialized.action, entry.action);
        assert_eq!(deserialized.actor, entry.actor);
    }
}
