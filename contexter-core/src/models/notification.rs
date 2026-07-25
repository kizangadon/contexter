//! Notification entity for system and user-facing notifications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A notification event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// Unique notification identifier.
    pub id: Uuid,
    /// Notification type (e.g. "memory_expired", "sync_conflict").
    pub notification_type: String,
    /// Human-readable message.
    pub message: String,
    /// Optional target entity ID.
    pub target_id: Option<Uuid>,
    /// Whether the notification has been read.
    #[serde(default)]
    pub read: bool,
    /// Timestamp when the notification was created.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Verify Notification serialization round-trip.
    #[test]
    fn notification_serialization_round_trip() {
        let notification = Notification {
            id: Uuid::now_v7(),
            notification_type: "memory_expired".into(),
            message: "A memory has expired".into(),
            target_id: Some(Uuid::now_v7()),
            read: false,
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&notification).expect("serialize Notification");
        assert_eq!(json["notificationType"], "memory_expired");
        assert_eq!(json["message"], "A memory has expired");
        assert_eq!(json["read"], false);

        let deserialized: Notification =
            serde_json::from_value(json).expect("deserialize Notification");
        assert_eq!(deserialized.id, notification.id);
        assert_eq!(deserialized.notification_type, notification.notification_type);
    }
}
