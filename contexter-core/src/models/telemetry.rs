//! Telemetry event entity for self-observability.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A telemetry event recorded by the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryEvent {
    /// Unique event identifier.
    pub id: Uuid,
    /// Event type (e.g. "cache_hit", "storage_write").
    pub event_type: String,
    /// Scope of the event (e.g. "session", "memory").
    pub scope: String,
    /// Numeric value associated with the event.
    pub value: f64,
    /// Arbitrary labels for filtering.
    #[serde(default)]
    pub labels: HashMap<String, String>,
    /// Timestamp when the event was recorded.
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    /// Verify TelemetryEvent serialization round-trip.
    #[test]
    fn telemetry_event_serialization_round_trip() {
        let event = TelemetryEvent {
            id: Uuid::now_v7(),
            event_type: "cache_hit".into(),
            scope: "memory".into(),
            value: 42.0,
            labels: HashMap::from([("cf".into(), "agents".into())]),
            timestamp: Utc::now(),
        };

        let json = serde_json::to_value(&event).expect("serialize TelemetryEvent");
        assert_eq!(json["eventType"], "cache_hit");
        assert_eq!(json["scope"], "memory");
        assert_eq!(json["value"], 42.0);

        let deserialized: TelemetryEvent =
            serde_json::from_value(json).expect("deserialize TelemetryEvent");
        assert_eq!(deserialized.id, event.id);
        assert_eq!(deserialized.event_type, event.event_type);
    }
}
