//! Correlation types for linking related entities across bounded contexts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A correlation link between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Correlation {
    /// Unique correlation identifier.
    pub id: Uuid,
    /// Source entity type.
    pub source_type: String,
    /// Source entity ID.
    pub source_id: Uuid,
    /// Target entity type.
    pub target_type: String,
    /// Target entity ID.
    pub target_id: Uuid,
    /// Relationship label (e.g. "contains", "references").
    pub relation: String,
    /// Timestamp when the correlation was recorded.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Verify Correlation serialization round-trip.
    #[test]
    fn correlation_serialization_round_trip() {
        let correlation = Correlation {
            id: Uuid::now_v7(),
            source_type: "memory".into(),
            source_id: Uuid::now_v7(),
            target_type: "session".into(),
            target_id: Uuid::now_v7(),
            relation: "contains".into(),
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&correlation).expect("serialize Correlation");
        assert_eq!(json["sourceType"], "memory");
        assert_eq!(json["targetType"], "session");
        assert_eq!(json["relation"], "contains");

        let deserialized: Correlation =
            serde_json::from_value(json).expect("deserialize Correlation");
        assert_eq!(deserialized.id, correlation.id);
        assert_eq!(deserialized.relation, correlation.relation);
    }
}
