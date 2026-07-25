//! Per-entity-type Tantivy schema definitions.
//!
//! Different entity types (memories, agents, skills, sessions) expose different
//! indexed fields. This module provides lazy-initialised [`EntitySchema`]
//! bundles keyed by entity type name.

use std::sync::OnceLock;

use tantivy::schema::*;

/// A bundle of a [`Schema`] and its pre-resolved [`Field`] handles.
///
/// # Design
///
/// Fields are resolved once at construction time so that callers never need to
/// call `schema.get_field_name(name)` at runtime. Optional fields are
/// `Option<Field>` — not every entity type has them.
pub struct EntitySchema {
    pub schema: Schema,
    pub id_field: Field,
    pub content_field: Field,
    pub default_search_fields: Vec<(Field, f32)>,
    pub tags_field: Option<Field>,
    pub entity_type_field: Field,
    // Entity-type-specific optional fields
    pub name_field: Option<Field>,
    pub description_field: Option<Field>,
    pub capabilities_field: Option<Field>,
    pub category_field: Option<Field>,
    pub project_field: Option<Field>,
    pub status_field: Option<Field>,
    pub metadata_field: Option<Field>,
}

// ---------------------------------------------------------------------------
// Memory schema
// ---------------------------------------------------------------------------

fn memory_schema() -> EntitySchema {
    let mut sb = Schema::builder();
    let id_field = sb.add_text_field("id", STRING | STORED);
    let content_field = sb.add_text_field("content", TEXT | STORED);
    let tags_field = sb.add_text_field("tags", STRING | STORED);
    let entity_type_field = sb.add_text_field("entity_type", STRING | STORED);
    EntitySchema {
        default_search_fields: vec![(content_field, 1.0), (tags_field, 1.5)],
        schema: sb.build(),
        id_field,
        content_field,
        tags_field: Some(tags_field),
        entity_type_field,
        name_field: None,
        description_field: None,
        capabilities_field: None,
        category_field: None,
        project_field: None,
        status_field: None,
        metadata_field: None,
    }
}

// ---------------------------------------------------------------------------
// Session schema
// ---------------------------------------------------------------------------

fn session_schema() -> EntitySchema {
    let mut sb = Schema::builder();
    let id_field = sb.add_text_field("id", STRING | STORED);
    let content_field = sb.add_text_field("content", TEXT | STORED);
    let project_field = sb.add_text_field("project", STRING | STORED);
    let status_field = sb.add_text_field("status", STRING | STORED);
    let entity_type_field = sb.add_text_field("entity_type", STRING | STORED);
    EntitySchema {
        default_search_fields: vec![(content_field, 1.0), (project_field, 1.0)],
        schema: sb.build(),
        id_field,
        content_field,
        tags_field: None,
        entity_type_field,
        name_field: None,
        description_field: None,
        capabilities_field: None,
        category_field: None,
        project_field: Some(project_field),
        status_field: Some(status_field),
        metadata_field: None,
    }
}

// ---------------------------------------------------------------------------
// Agent schema
// ---------------------------------------------------------------------------

fn agent_schema() -> EntitySchema {
    let mut sb = Schema::builder();
    let id_field = sb.add_text_field("id", STRING | STORED);
    let content_field = sb.add_text_field("content", TEXT | STORED);
    let name_field = sb.add_text_field("name", TEXT | STORED);
    let description_field = sb.add_text_field("description", TEXT | STORED);
    let capabilities_field = sb.add_text_field("capabilities", STRING | STORED);
    let status_field = sb.add_text_field("status", STRING | STORED);
    let entity_type_field = sb.add_text_field("entity_type", STRING | STORED);
    EntitySchema {
        default_search_fields: vec![
            (content_field, 1.0),
            (name_field, 2.0),
            (description_field, 1.0),
            (capabilities_field, 1.0),
        ],
        schema: sb.build(),
        id_field,
        content_field,
        tags_field: None,
        entity_type_field,
        name_field: Some(name_field),
        description_field: Some(description_field),
        capabilities_field: Some(capabilities_field),
        category_field: None,
        project_field: None,
        status_field: Some(status_field),
        metadata_field: None,
    }
}

// ---------------------------------------------------------------------------
// Skill schema
// ---------------------------------------------------------------------------

fn skill_schema() -> EntitySchema {
    let mut sb = Schema::builder();
    let id_field = sb.add_text_field("id", STRING | STORED);
    let content_field = sb.add_text_field("content", TEXT | STORED);
    let name_field = sb.add_text_field("name", TEXT | STORED);
    let description_field = sb.add_text_field("description", TEXT | STORED);
    let category_field = sb.add_text_field("category", STRING | STORED);
    let entity_type_field = sb.add_text_field("entity_type", STRING | STORED);
    EntitySchema {
        default_search_fields: vec![
            (content_field, 1.0),
            (name_field, 2.0),
            (description_field, 1.0),
            (category_field, 1.0),
        ],
        schema: sb.build(),
        id_field,
        content_field,
        tags_field: None,
        entity_type_field,
        name_field: Some(name_field),
        description_field: Some(description_field),
        capabilities_field: None,
        category_field: Some(category_field),
        project_field: None,
        status_field: None,
        metadata_field: None,
    }
}

// ---------------------------------------------------------------------------
// Default (fallback) schema
// ---------------------------------------------------------------------------

fn default_schema() -> EntitySchema {
    let mut sb = Schema::builder();
    let id_field = sb.add_text_field("id", STRING | STORED);
    let content_field = sb.add_text_field("content", TEXT | STORED);
    let entity_type_field = sb.add_text_field("entity_type", STRING | STORED);
    EntitySchema {
        default_search_fields: vec![(content_field, 1.0)],
        schema: sb.build(),
        id_field,
        content_field,
        tags_field: None,
        entity_type_field,
        name_field: None,
        description_field: None,
        capabilities_field: None,
        category_field: None,
        project_field: None,
        status_field: None,
        metadata_field: None,
    }
}

// ---------------------------------------------------------------------------
// Lazy statics
// ---------------------------------------------------------------------------

static MEMORY_SCHEMA: OnceLock<EntitySchema> = OnceLock::new();
static SESSION_SCHEMA: OnceLock<EntitySchema> = OnceLock::new();
static AGENT_SCHEMA: OnceLock<EntitySchema> = OnceLock::new();
static SKILL_SCHEMA: OnceLock<EntitySchema> = OnceLock::new();
static DEFAULT_SCHEMA: OnceLock<EntitySchema> = OnceLock::new();

pub fn get_memory_schema() -> &'static EntitySchema {
    MEMORY_SCHEMA.get_or_init(memory_schema)
}

pub fn get_session_schema() -> &'static EntitySchema {
    SESSION_SCHEMA.get_or_init(session_schema)
}

pub fn get_agent_schema() -> &'static EntitySchema {
    AGENT_SCHEMA.get_or_init(agent_schema)
}

pub fn get_skill_schema() -> &'static EntitySchema {
    SKILL_SCHEMA.get_or_init(skill_schema)
}

pub fn get_default_schema() -> &'static EntitySchema {
    DEFAULT_SCHEMA.get_or_init(default_schema)
}

/// Return the [`EntitySchema`] appropriate for a given entity type.
pub fn schema_for_entity(entity_type: &str) -> &'static EntitySchema {
    match entity_type {
        "memory" | "memories" => get_memory_schema(),
        "session" | "sessions" => get_session_schema(),
        "agent" | "agents" => get_agent_schema(),
        "skill" | "skills" => get_skill_schema(),
        _ => get_default_schema(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_schema_has_tags_and_no_name() {
        let s = schema_for_entity("memory");
        assert!(s.tags_field.is_some(), "memory schema should have tags");
        assert!(s.name_field.is_none(), "memory schema should not have name");
        assert!(
            s.default_search_fields.len() == 2,
            "memory should have 2 default search fields"
        );
    }

    #[test]
    fn session_schema_has_project_and_status() {
        let s = schema_for_entity("session");
        assert!(s.project_field.is_some(), "session schema should have project");
        assert!(s.status_field.is_some(), "session schema should have status");
        assert!(s.tags_field.is_none(), "session schema should not have tags");
        assert!(
            s.default_search_fields.len() == 2,
            "session should have 2 default search fields"
        );
    }

    #[test]
    fn agent_schema_has_name_description_capabilities() {
        let s = schema_for_entity("agent");
        assert!(s.name_field.is_some(), "agent schema should have name");
        assert!(
            s.description_field.is_some(),
            "agent schema should have description"
        );
        assert!(
            s.capabilities_field.is_some(),
            "agent schema should have capabilities"
        );
        assert!(s.status_field.is_some(), "agent schema should have status");
        assert!(
            s.default_search_fields.len() == 4,
            "agent should have 4 default search fields"
        );
    }

    #[test]
    fn skill_schema_has_name_description_category() {
        let s = schema_for_entity("skill");
        assert!(s.name_field.is_some(), "skill schema should have name");
        assert!(
            s.description_field.is_some(),
            "skill schema should have description"
        );
        assert!(
            s.category_field.is_some(),
            "skill schema should have category"
        );
        assert!(
            s.default_search_fields.len() == 4,
            "skill should have 4 default search fields"
        );
    }

    #[test]
    fn default_schema_has_no_optional_fields() {
        let s = schema_for_entity("unknown");
        assert!(s.tags_field.is_none(), "default schema should not have tags");
        assert!(s.name_field.is_none(), "default schema should not have name");
        assert!(
            s.description_field.is_none(),
            "default schema should not have description"
        );
        assert!(
            s.capabilities_field.is_none(),
            "default schema should not have capabilities"
        );
        assert!(
            s.category_field.is_none(),
            "default schema should not have category"
        );
        assert!(
            s.project_field.is_none(),
            "default schema should not have project"
        );
        assert!(
            s.status_field.is_none(),
            "default schema should not have status"
        );
        assert!(
            s.metadata_field.is_none(),
            "default schema should not have metadata"
        );
        assert!(
            s.default_search_fields.len() == 1,
            "default should have 1 default search field"
        );
    }

    #[test]
    fn memories_alias_maps_to_memory_schema() {
        let memory = schema_for_entity("memory");
        let memories = schema_for_entity("memories");
        assert!(
            std::ptr::eq(memory, memories),
            "both should return same static"
        );
    }

    #[test]
    fn sessions_alias_maps_to_session_schema() {
        let s = schema_for_entity("session");
        let sessions = schema_for_entity("sessions");
        assert!(std::ptr::eq(s, sessions), "both should return same static");
    }

    #[test]
    fn agents_alias_maps_to_agent_schema() {
        let a = schema_for_entity("agent");
        let agents = schema_for_entity("agents");
        assert!(std::ptr::eq(a, agents), "both should return same static");
    }

    #[test]
    fn skills_alias_maps_to_skill_schema() {
        let s = schema_for_entity("skill");
        let skills = schema_for_entity("skills");
        assert!(std::ptr::eq(s, skills), "both should return same static");
    }
}
