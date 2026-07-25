//! Memory entity — a stored fact, preference, procedure, context, or episode.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::fts::TextContent;

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

impl TextContent for Memory {
    fn text_content(&self) -> String {
        let tags_part = if self.tags.is_empty() {
            String::new()
        } else {
            format!(" {}", self.tags.join(" "))
        };
        format!("{}{}", self.content, tags_part)
    }
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
    /// Project filter — reserved for Phase 2 when Memory resolves project via Session join.
    #[serde(skip)]
    #[allow(dead_code)]
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
    use crate::fts::TextContent;
    use chrono::Utc;

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

    /// Verify MemorySearchQuery defaults.
    #[test]
    fn memory_search_query_defaults() {
        let query = MemorySearchQuery::default();
        assert_eq!(query.limit, 100);
        assert_eq!(query.offset, 0);
    }

    /// Verify Version defaults to 1.
    #[test]
    fn version_defaults_to_one() {
        let now = Utc::now();
        let memory = Memory {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "test".into(),
            embedding: None,
            tags: vec![],
            version: 1,
            created_at: now,
            updated_at: now,
        };
        assert_eq!(memory.version, 1);
    }

    /// Verify MemoryType serialization.
    #[test]
    fn memory_type_serialization() {
        let json = serde_json::to_value(&MemoryType::Fact).expect("serialize MemoryType");
        assert_eq!(json, "fact");
    }

    /// Verify that `project` field is silently ignored during deserialization.
    #[test]
    fn search_query_ignores_project_field_during_deserialization() {
        let json = r#"{"project": "some-project"}"#;
        let query: MemorySearchQuery =
            serde_json::from_str(json).expect("deserialize MemorySearchQuery");
        assert!(
            query.project.is_none(),
            "project should be ignored during deserialization"
        );
    }

    // -----------------------------------------------------------------------
    // TextContent
    // -----------------------------------------------------------------------

    #[test]
    fn text_content_concatenates_content_and_tags() {
        let now = Utc::now();
        let m = Memory {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "important fact".into(),
            embedding: None,
            tags: vec!["rust".into(), "async".into()],
            version: 1,
            created_at: now,
            updated_at: now,
        };
        let tc = m.text_content();
        assert!(tc.contains("important fact"));
        assert!(tc.contains("rust"));
        assert!(tc.contains("async"));
    }

    #[test]
    fn text_content_handles_empty_tags() {
        let now = Utc::now();
        let m = Memory {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "just content".into(),
            embedding: None,
            tags: vec![],
            version: 1,
            created_at: now,
            updated_at: now,
        };
        let tc = m.text_content();
        assert_eq!(tc, "just content");
    }

    #[test]
    fn text_content_handles_single_tag() {
        let now = Utc::now();
        let m = Memory {
            id: Uuid::now_v7(),
            session_id: Uuid::now_v7(),
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: "some data".into(),
            embedding: None,
            tags: vec!["unique".into()],
            version: 1,
            created_at: now,
            updated_at: now,
        };
        let tc = m.text_content();
        assert_eq!(tc, "some data unique");
    }
}
