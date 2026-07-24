//! Agent entity — an AI agent registered in the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
}
