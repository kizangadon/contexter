//! Feedback entity for user-provided feedback on memories and interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User feedback on a memory or interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    /// Unique feedback identifier.
    pub id: Uuid,
    /// The memory or entity this feedback pertains to.
    pub target_id: Uuid,
    /// Rating value (e.g. 1-5).
    pub rating: u8,
    /// Optional free-text comment.
    pub comment: Option<String>,
    /// Optional actor that provided the feedback.
    pub actor: Option<String>,
    /// Timestamp when the feedback was recorded.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Verify Feedback serialization round-trip.
    #[test]
    fn feedback_serialization_round_trip() {
        let feedback = Feedback {
            id: Uuid::now_v7(),
            target_id: Uuid::now_v7(),
            rating: 5,
            comment: Some("Great memory!".into()),
            actor: Some("user123".into()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&feedback).expect("serialize Feedback");
        assert_eq!(json["rating"], 5);
        assert_eq!(json["comment"], "Great memory!");
        assert!(json.get("actor").is_some());

        let deserialized: Feedback = serde_json::from_value(json).expect("deserialize Feedback");
        assert_eq!(deserialized.id, feedback.id);
        assert_eq!(deserialized.rating, feedback.rating);
    }

    /// Verify that Feedback with no comment or actor still serializes.
    #[test]
    fn feedback_minimal() {
        let feedback = Feedback {
            id: Uuid::now_v7(),
            target_id: Uuid::now_v7(),
            rating: 3,
            comment: None,
            actor: None,
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&feedback).expect("serialize Feedback");
        assert_eq!(json["rating"], 3);
    }
}
