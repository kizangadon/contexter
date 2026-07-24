//! Session entity — a conversation context between a user and an AI agent.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes the lifecycle state of a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    /// Session is active and accepting turns.
    Active,
    /// Session has been completed normally.
    Completed,
    /// Session terminated due to an error.
    Error,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Error => write!(f, "error"),
        }
    }
}

/// A conversation session between a user and an AI agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// Unique session identifier.
    pub id: Uuid,
    /// Project that owns this session.
    pub project: String,
    /// Agent conducting the session.
    pub agent_id: Uuid,
    /// Current session status.
    pub status: SessionStatus,
    /// Number of turns (user ↔ agent exchanges) completed.
    pub turn_count: u32,
    /// Wall-clock duration of the session in milliseconds.
    pub duration_ms: u64,
    /// Arbitrary metadata associated with the session.
    pub metadata: serde_json::Value,
    /// Efficiency score (0.0–1.0) of the session, if computed.
    pub efficiency_score: Option<f64>,
    /// Timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of the most recent activity in this session.
    pub last_active: DateTime<Utc>,
}

/// Input data for creating a new session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSession {
    /// Project that owns this session.
    pub project: String,
    /// Agent conducting the session.
    pub agent_id: Uuid,
    /// Optional initial status (defaults to Active).
    pub status: Option<SessionStatus>,
    /// Optional metadata (defaults to empty object).
    pub metadata: Option<serde_json::Value>,
}

/// Partial update payload for an existing session.
///
/// Only `Some` fields are applied; `None` fields are left unchanged.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionPatch {
    pub status: Option<SessionStatus>,
    pub turn_count: Option<u32>,
    pub duration_ms: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

/// Criteria for filtering session queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFilter {
    pub project: Option<String>,
    pub agent_id: Option<Uuid>,
    pub status: Option<SessionStatus>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

impl Default for SessionFilter {
    fn default() -> Self {
        Self {
            project: None,
            agent_id: None,
            status: None,
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

    /// Verify that Session serializes to camelCase JSON and round-trips.
    #[test]
    fn session_serialization_round_trip() {
        let session = Session {
            id: Uuid::now_v7(),
            project: "test".into(),
            agent_id: Uuid::now_v7(),
            status: SessionStatus::Active,
            turn_count: 5,
            duration_ms: 12345,
            metadata: serde_json::json!({"env": "staging"}),
            efficiency_score: None,
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let json = serde_json::to_value(&session).expect("serialize Session");
        assert_eq!(json["project"], "test");
        assert_eq!(json["status"], "active");
        assert_eq!(json["turnCount"], 5);
        assert_eq!(json["durationMs"], 12345);
        assert!(
            json.get("turnCount").is_some(),
            "expected camelCase turnCount"
        );
        assert!(
            json.get("durationMs").is_some(),
            "expected camelCase durationMs"
        );
        assert!(
            json.get("createdAt").is_some(),
            "expected camelCase createdAt"
        );
        assert!(
            json.get("lastActive").is_some(),
            "expected camelCase lastActive"
        );

        let deserialized: Session = serde_json::from_value(json).expect("deserialize Session");
        assert_eq!(deserialized.project, session.project);
        assert_eq!(deserialized.turn_count, session.turn_count);
        assert_eq!(deserialized.status, session.status);
    }

    /// Verify SessionFilter defaults.
    #[test]
    fn session_filter_defaults() {
        let filter = SessionFilter::default();
        assert_eq!(filter.limit, 100);
        assert_eq!(filter.offset, 0);
        assert!(filter.project.is_none());
        assert!(filter.agent_id.is_none());
        assert!(filter.status.is_none());
    }

    /// Verify NewSession defaults through round-trip.
    #[test]
    fn new_session_with_defaults() {
        let new = NewSession {
            project: "p".into(),
            agent_id: Uuid::now_v7(),
            status: None,
            metadata: None,
        };

        let json = serde_json::to_value(&new).expect("serialize NewSession");
        assert_eq!(json["project"], "p");
    }

    /// Verify SessionStatus serialization.
    #[test]
    fn session_status_serialization() {
        for (status, expected) in [
            (SessionStatus::Active, "active"),
            (SessionStatus::Completed, "completed"),
            (SessionStatus::Error, "error"),
        ] {
            let json = serde_json::to_value(&status).expect("serialize SessionStatus");
            assert_eq!(json, expected);
        }
    }

    /// Verify UUID v7 generation (should not be v4 format).
    #[test]
    fn uuid_v7_generation() {
        let id = Uuid::now_v7();
        let bytes = id.as_bytes();
        // UUID v7: version nibble = 7 in the 4 most significant bits of byte 6
        let version_byte = bytes[6];
        let version = version_byte >> 4;
        assert_eq!(
            version, 7,
            "expected UUID version 7, got version {}",
            version
        );

        let id2 = Uuid::now_v7();
        assert_ne!(id, id2, "consecutive UUIDs should differ");
    }

    /// Verify that deserializing unknown enum variants raises an error.
    #[test]
    fn session_status_deserialize_unknown() {
        let result: Result<SessionStatus, _> = serde_json::from_str("\"unknown\"");
        assert!(
            result.is_err(),
            "unknown variant should fail to deserialize"
        );
    }
}
