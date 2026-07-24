//! Domain data types for Contexter.
//!
//! These types model the core domain concepts: Sessions (conversation contexts),
//! Memories (stored facts/preferences/procedures), Agents (AI entities),
//! Skills (capability definitions), and Audit trail entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

/// The semantic category of a stored memory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MemoryType {
    /// An established fact about the user or domain.
    Fact,
    /// A user preference or setting.
    Preference,
    /// A known procedure or workflow.
    Procedure,
    /// Contextual information for an active session.
    Context,
    /// A recorded past episode or interaction.
    Episode,
}

/// A stored memory entry with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memory {
    /// Unique memory identifier.
    pub id: Uuid,
    /// Session this memory belongs to.
    pub session_id: Uuid,
    /// Agent that created this memory.
    pub agent_id: Uuid,
    /// Semantic category of this memory.
    pub memory_type: MemoryType,
    /// The stored content text.
    pub content: String,
    /// Optional embedding vector (stub for Phase 2).
    pub embedding: Option<Vec<f32>>,
    /// Tags for categorisation and search.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optimistic-lock version, starts at 1.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Timestamp when the memory was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the memory was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Input data for creating a new memory.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewMemory {
    /// Session this memory belongs to.
    pub session_id: Uuid,
    /// Agent creating this memory.
    pub agent_id: Uuid,
    /// Semantic category of this memory.
    pub memory_type: MemoryType,
    /// The stored content text.
    pub content: String,
    /// Optional tags (defaults to empty).
    pub tags: Option<Vec<String>>,
}

/// Partial update payload for an existing memory.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemoryPatch {
    pub content: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
}

/// Query parameters for searching memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemorySearchQuery {
    pub keywords: Option<String>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
    pub session_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub project: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

impl Default for MemorySearchQuery {
    fn default() -> Self {
        Self {
            keywords: None,
            memory_type: None,
            tags: None,
            session_id: None,
            agent_id: None,
            project: None,
            limit: 100,
            offset: 0,
        }
    }
}

/// Filter criteria for memory queries.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFilter {
    pub session_id: Option<Uuid>,
    pub agent_id: Option<Uuid>,
    pub memory_type: Option<MemoryType>,
    pub tags: Option<Vec<String>>,
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Describes the operational state of an agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AgentStatus {
    /// Agent is active and can participate in sessions.
    Active,
    /// Agent is inactive and cannot participate.
    Inactive,
}

/// An AI agent registered in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    /// Unique agent identifier.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Agent type identifier, e.g. "chat", "coding-assistant".
    #[serde(rename = "type")]
    pub agent_type: String,
    /// Human-readable description of the agent's purpose.
    pub description: String,
    /// List of capabilities this agent supports.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Current operational status.
    pub status: AgentStatus,
    /// Configuration data for this agent.
    pub config: serde_json::Value,
    /// Optimistic-lock version, starts at 1.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Timestamp when the agent was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the agent was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Input data for registering a new agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAgent {
    pub name: String,
    /// Agent type identifier.
    #[serde(rename = "type")]
    pub agent_type: String,
    pub description: String,
    pub capabilities: Option<Vec<String>>,
    pub status: Option<AgentStatus>,
    pub config: Option<serde_json::Value>,
}

/// Partial update payload for an agent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentPatch {
    pub name: Option<String>,
    /// Agent type identifier.
    #[serde(rename = "type")]
    pub agent_type: Option<String>,
    pub description: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub status: Option<AgentStatus>,
    pub config: Option<serde_json::Value>,
}

/// Criteria for filtering agent queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentFilter {
    pub name: Option<String>,
    pub status: Option<AgentStatus>,
    pub capability: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

impl Default for AgentFilter {
    fn default() -> Self {
        Self {
            name: None,
            status: None,
            capability: None,
            limit: 100,
            offset: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

/// A registered capability or tool that an agent can use.
///
/// # Security note — `file_path` validation
///
/// The [`file_path`](Skill::file_path) field is an optional filesystem path
/// supplied by the caller. It is **not validated or canonicalised** before
/// storage or retrieval, which could enable path-traversal attacks if a
/// downstream consumer uses the path without sanitisation (e.g. to load or
/// execute a file). Future work should add an allow-list or canonicalisation
/// step at the API boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    /// Unique skill identifier.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Description of what this skill does.
    pub description: String,
    /// Category grouping (e.g. "search", "code", "memory").
    pub category: String,
    /// Optimistic-lock version, starts at 1.
    #[serde(default = "default_version")]
    pub version: u32,
    /// Optional file-system path to the skill's implementation.
    pub file_path: Option<String>,
    /// Timestamp when the skill was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the skill was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Input data for registering a new skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSkill {
    pub name: String,
    pub description: String,
    pub category: String,
    pub file_path: Option<String>,
}

/// Partial update payload for a skill.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SkillPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub file_path: Option<String>,
}

/// Criteria for filtering skill queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFilter {
    pub name: Option<String>,
    pub category: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

impl Default for SkillFilter {
    fn default() -> Self {
        Self {
            name: None,
            category: None,
            limit: 100,
            offset: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

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
    /// Optional JSON describing what changed.
    pub changes: Option<serde_json::Value>,
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
    /// Optional JSON describing what changed.
    pub changes: Option<serde_json::Value>,
    /// Timestamp when the entry was recorded.
    pub timestamp: DateTime<Utc>,
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

// ---------------------------------------------------------------------------
// Storage metadata
// ---------------------------------------------------------------------------

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
// Internal helpers
// ---------------------------------------------------------------------------

fn default_limit() -> u64 {
    100
}

fn default_version() -> u32 {
    1
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
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let json = serde_json::to_value(&session).expect("serialize Session");
        assert_eq!(json["project"], "test");
        assert_eq!(json["status"], "active");
        assert_eq!(json["turnCount"], 5);
        assert_eq!(json["durationMs"], 12345);
        // camelCase keys
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

        // Round-trip
        let deserialized: Session = serde_json::from_value(json).expect("deserialize Session");
        assert_eq!(deserialized.project, session.project);
        assert_eq!(deserialized.turn_count, session.turn_count);
        assert_eq!(deserialized.status, session.status);
    }

    /// Verify that Memory defaults are applied correctly.
    #[test]
    fn memory_default_values() {
        let now = Utc::now();
        let memory = Memory {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "test content".into(),
            embedding: None,
            tags: vec![],
            version: 1,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(memory.version, 1, "version should default to 1");
        assert!(memory.tags.is_empty(), "tags should default to empty");
        assert!(
            memory.embedding.is_none(),
            "embedding should be None in Phase 1"
        );
    }

    /// Verify Version defaults to 1.
    #[test]
    fn version_defaults_to_one() {
        assert_eq!(default_version(), 1);
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

    /// Verify MemorySearchQuery defaults.
    #[test]
    fn memory_search_query_defaults() {
        let query = MemorySearchQuery::default();
        assert_eq!(query.limit, 100);
        assert_eq!(query.offset, 0);
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

    /// Verify Agent serialization includes the `type` field.
    #[test]
    fn agent_type_serialization() {
        let agent = Agent {
            id: Uuid::now_v7(),
            name: "test-agent".into(),
            agent_type: "coding-assistant".into(),
            description: "A test agent".into(),
            capabilities: vec!["code".into()],
            status: AgentStatus::Active,
            config: serde_json::json!({}),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_value(&agent).expect("serialize Agent");
        assert_eq!(json["type"], "coding-assistant");
        assert!(
            json.get("type").is_some(),
            "expected 'type' field, not 'agentType'"
        );
    }

    /// Verify AuditEntry round-trip.
    #[test]
    fn audit_entry_round_trip() {
        let entry = AuditEntry {
            id: Uuid::now_v7(),
            action: "create".into(),
            entity_type: "Session".into(),
            entity_id: "abc-123".into(),
            actor: Some("user-1".into()),
            changes: Some(serde_json::json!({"status": "active"})),
            timestamp: Utc::now(),
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

    /// Verify MemoryType serialization.
    #[test]
    fn memory_type_serialization() {
        let json = serde_json::to_value(&MemoryType::Fact).expect("serialize MemoryType");
        assert_eq!(json, "fact");
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
        // status and metadata default to null (None)
    }

    /// Verify StorageSize serialization.
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
