//! Skill entity — a registered capability or tool that an agent can use.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

fn default_limit() -> u64 {
    100
}

fn default_version() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    /// Verify Skill serialization round-trip.
    #[test]
    fn skill_serialization_round_trip() {
        let skill = Skill {
            id: Uuid::now_v7(),
            name: "test-skill".into(),
            description: "A test skill".into(),
            category: "code".into(),
            version: 1,
            file_path: Some("/path/to/skill.py".into()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_value(&skill).expect("serialize Skill");
        assert_eq!(json["name"], "test-skill");
        assert_eq!(json["category"], "code");
        assert!(json.get("filePath").is_some());

        let deserialized: Skill = serde_json::from_value(json).expect("deserialize Skill");
        assert_eq!(deserialized.id, skill.id);
        assert_eq!(deserialized.version, skill.version);
    }

    /// Verify NewSkill serialization.
    #[test]
    fn new_skill_serialization() {
        let new = NewSkill {
            name: "new-skill".into(),
            description: "desc".into(),
            category: "search".into(),
            file_path: None,
        };

        let json = serde_json::to_value(&new).expect("serialize NewSkill");
        assert_eq!(json["name"], "new-skill");
    }

    /// Verify SkillFilter defaults.
    #[test]
    fn skill_filter_defaults() {
        let filter = SkillFilter::default();
        assert_eq!(filter.limit, 100);
        assert_eq!(filter.offset, 0);
    }
}
