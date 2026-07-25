//! Integration tests for the models module.
//!
//! Tests that all domain entity types can be created, serialized, and
//! deserialized correctly at the integration level. Each model type
//! has its own unit tests in `src/models/*.rs`; this file provides
//! cross-module integration coverage.

use std::collections::HashMap;

use chrono::Utc;
use uuid::Uuid;

use contexter_core::*;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Session model
// ---------------------------------------------------------------------------

#[test]
fn test_session_serialization() {
    let session = Session {
        id: Uuid::now_v7(),
        project: "integration-test".into(),
        agent_id: Uuid::now_v7(),
        status: SessionStatus::Active,
        turn_count: 42,
        duration_ms: 12345,
        metadata: serde_json::json!({"key": "value"}),
        efficiency_score: Some(0.95),
        created_at: Utc::now(),
        last_active: Utc::now(),
    };

    let json = serde_json::to_value(&session).expect("serialize Session");
    assert_eq!(json["project"], "integration-test");
    assert_eq!(json["turnCount"], 42);
    assert_eq!(json["durationMs"], 12345);
    assert_eq!(json["status"], "active");
    assert_eq!(json["efficiencyScore"], 0.95);

    let deserialized: Session = serde_json::from_value(json).expect("deserialize Session");
    assert_eq!(deserialized.id, session.id);
    assert_eq!(deserialized.project, session.project);
    assert_eq!(deserialized.turn_count, session.turn_count);
}

#[test]
fn test_new_session_serialization() {
    let agent_id = Uuid::now_v7();
    let new = NewSession {
        project: "new-project".into(),
        agent_id,
        status: Some(SessionStatus::Active),
        metadata: Some(serde_json::json!({"env": "prod"})),
    };

    let json = serde_json::to_value(&new).expect("serialize NewSession");
    assert_eq!(json["project"], "new-project");
    assert_eq!(json["status"], "active");

    let deserialized: NewSession = serde_json::from_value(json).expect("deserialize NewSession");
    assert_eq!(deserialized.project, new.project);
    assert_eq!(deserialized.agent_id, agent_id);
}

#[test]
fn test_session_patch_serialization() {
    let patch = SessionPatch {
        status: Some(SessionStatus::Completed),
        turn_count: Some(10),
        duration_ms: Some(5000),
        metadata: Some(serde_json::json!({"updated": true})),
    };

    let json = serde_json::to_value(&patch).expect("serialize SessionPatch");
    assert_eq!(json["status"], "completed");
    assert_eq!(json["turnCount"], 10);
    assert_eq!(json["durationMs"], 5000);

    let deserialized: SessionPatch =
        serde_json::from_value(json).expect("deserialize SessionPatch");
    assert_eq!(deserialized.status, Some(SessionStatus::Completed));
}

#[test]
fn test_session_filter_serialization() {
    let filter = SessionFilter {
        project: Some("my-proj".into()),
        agent_id: Some(Uuid::now_v7()),
        status: Some(SessionStatus::Active),
        limit: 50,
        offset: 10,
    };

    let json = serde_json::to_value(&filter).expect("serialize SessionFilter");
    assert_eq!(json["project"], "my-proj");
    assert_eq!(json["limit"], 50);
    assert_eq!(json["offset"], 10);

    let deserialized: SessionFilter =
        serde_json::from_value(json).expect("deserialize SessionFilter");
    assert_eq!(deserialized.project, filter.project);
    assert_eq!(deserialized.limit, 50);
}

#[test]
fn test_session_status_deserialize() {
    for (json_str, expected) in [
        ("\"active\"", SessionStatus::Active),
        ("\"completed\"", SessionStatus::Completed),
        ("\"error\"", SessionStatus::Error),
    ] {
        let status: SessionStatus =
            serde_json::from_str(json_str).expect("deserialize SessionStatus");
        assert_eq!(status, expected);
    }
}

// ---------------------------------------------------------------------------
// Memory model
// ---------------------------------------------------------------------------

#[test]
fn test_memory_serialization() {
    let memory = Memory {
        id: Uuid::now_v7(),
        session_id: Uuid::now_v7(),
        agent_id: Uuid::now_v7(),
        memory_type: MemoryType::Fact,
        content: "test content".into(),
        embedding: None,
        tags: vec!["tag1".into(), "tag2".into()],
        version: 3,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_value(&memory).expect("serialize Memory");
    assert_eq!(json["memoryType"], "fact");
    assert_eq!(json["content"], "test content");
    assert_eq!(json["version"], 3);
    assert!(json["tags"].is_array());
    assert_eq!(json["tags"].as_array().unwrap().len(), 2);

    let deserialized: Memory = serde_json::from_value(json).expect("deserialize Memory");
    assert_eq!(deserialized.id, memory.id);
    assert_eq!(deserialized.version, 3);
    assert_eq!(deserialized.tags.len(), 2);
}

#[test]
fn test_new_memory_serialization() {
    let new = NewMemory {
        session_id: Uuid::now_v7(),
        agent_id: Uuid::now_v7(),
        memory_type: MemoryType::Fact,
        content: "new memory content".into(),
        tags: Some(vec!["important".into()]),
    };

    let json = serde_json::to_value(&new).expect("serialize NewMemory");
    assert_eq!(json["memoryType"], "fact");

    let deserialized: NewMemory = serde_json::from_value(json).expect("deserialize NewMemory");
    assert_eq!(deserialized.content, "new memory content");
}

#[test]
fn test_memory_search_query_serialization() {
    let query = MemorySearchQuery {
        keywords: Some("search term".into()),
        memory_type: Some(MemoryType::Fact),
        tags: Some(vec!["tag1".into()]),
        session_id: Some(Uuid::now_v7()),
        agent_id: Some(Uuid::now_v7()),
        project: None,
        limit: 20,
        offset: 5,
    };

    let json = serde_json::to_value(&query).expect("serialize MemorySearchQuery");
    assert_eq!(json["keywords"], "search term");
    assert_eq!(json["limit"], 20);
    assert_eq!(json["offset"], 5);

    let deserialized: MemorySearchQuery =
        serde_json::from_value(json).expect("deserialize MemorySearchQuery");
    assert_eq!(deserialized.keywords, query.keywords);
}

#[test]
fn test_memory_filter_serialization() {
    let filter = MemoryFilter {
        session_id: Some(Uuid::now_v7()),
        agent_id: Some(Uuid::now_v7()),
        memory_type: Some(MemoryType::Fact),
        tags: Some(vec!["important".into()]),
    };

    let json = serde_json::to_value(&filter).expect("serialize MemoryFilter");
    assert_eq!(json["memoryType"], "fact");

    let deserialized: MemoryFilter =
        serde_json::from_value(json).expect("deserialize MemoryFilter");
    assert_eq!(deserialized.memory_type, filter.memory_type);
}

#[test]
fn test_memory_patch_serialization() {
    let patch = MemoryPatch {
        content: Some("updated".into()),
        memory_type: Some(MemoryType::Procedure),
        tags: Some(vec!["new-tag".into()]),
    };

    let json = serde_json::to_value(&patch).expect("serialize MemoryPatch");
    assert_eq!(json["content"], "updated");
    assert_eq!(json["memoryType"], "procedure");

    let deserialized: MemoryPatch =
        serde_json::from_value(json).expect("deserialize MemoryPatch");
    assert_eq!(deserialized.content, Some("updated".into()));
}

#[test]
fn test_memory_type_deserialize() {
    for (json_str, expected) in [
        ("\"fact\"", MemoryType::Fact),
        ("\"preference\"", MemoryType::Preference),
        ("\"procedure\"", MemoryType::Procedure),
        ("\"context\"", MemoryType::Context),
        ("\"episode\"", MemoryType::Episode),
    ] {
        let mt: MemoryType = serde_json::from_str(json_str).expect("deserialize MemoryType");
        assert_eq!(mt, expected);
    }
}

// ---------------------------------------------------------------------------
// Agent model
// ---------------------------------------------------------------------------

#[test]
fn test_agent_serialization() {
    let agent = Agent {
        id: Uuid::now_v7(),
        name: "test-agent".into(),
        agent_type: "coding-assistant".into(),
        description: "A test agent".into(),
        capabilities: vec!["code".into(), "review".into()],
        status: AgentStatus::Active,
        config: serde_json::json!({"model": "gpt-4"}),
        version: 2,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_value(&agent).expect("serialize Agent");
    assert_eq!(json["name"], "test-agent");
    assert_eq!(json["type"], "coding-assistant");
    assert_eq!(json["status"], "active");
    assert_eq!(json["version"], 2);

    let deserialized: Agent = serde_json::from_value(json).expect("deserialize Agent");
    assert_eq!(deserialized.id, agent.id);
    assert_eq!(deserialized.agent_type, "coding-assistant");
}

#[test]
fn test_new_agent_serialization() {
    let new = NewAgent {
        name: "new-agent".into(),
        agent_type: "chat".into(),
        description: "desc".into(),
        capabilities: Some(vec!["chat".into()]),
        status: Some(AgentStatus::Active),
        config: Some(serde_json::json!({"key": "val"})),
    };

    let json = serde_json::to_value(&new).expect("serialize NewAgent");
    assert_eq!(json["name"], "new-agent");
    assert_eq!(json["type"], "chat");

    let deserialized: NewAgent = serde_json::from_value(json).expect("deserialize NewAgent");
    assert_eq!(deserialized.name, "new-agent");
}

#[test]
fn test_agent_filter_serialization() {
    let filter = AgentFilter {
        name: Some("test".into()),
        status: Some(AgentStatus::Active),
        capability: Some("code".into()),
        limit: 50,
        offset: 0,
    };

    let json = serde_json::to_value(&filter).expect("serialize AgentFilter");
    assert_eq!(json["name"], "test");
    assert_eq!(json["limit"], 50);

    let deserialized: AgentFilter =
        serde_json::from_value(json).expect("deserialize AgentFilter");
    assert_eq!(deserialized.name, filter.name);
}

// ---------------------------------------------------------------------------
// Skill model
// ---------------------------------------------------------------------------

#[test]
fn test_skill_serialization() {
    let skill = Skill {
        id: Uuid::now_v7(),
        name: "code-review".into(),
        description: "Review code quality".into(),
        category: "dev".into(),
        version: 1,
        file_path: Some("/path/to/skill.py".into()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let json = serde_json::to_value(&skill).expect("serialize Skill");
    assert_eq!(json["name"], "code-review");
    assert_eq!(json["category"], "dev");
    assert!(json.get("filePath").is_some());

    let deserialized: Skill = serde_json::from_value(json).expect("deserialize Skill");
    assert_eq!(deserialized.id, skill.id);
    assert_eq!(deserialized.file_path, skill.file_path);
}

#[test]
fn test_new_skill_serialization() {
    let new = NewSkill {
        name: "new-skill".into(),
        description: "desc".into(),
        category: "search".into(),
        file_path: None,
    };

    let json = serde_json::to_value(&new).expect("serialize NewSkill");
    assert_eq!(json["name"], "new-skill");

    let deserialized: NewSkill = serde_json::from_value(json).expect("deserialize NewSkill");
    assert_eq!(deserialized.name, "new-skill");
}

// ---------------------------------------------------------------------------
// Audit model
// ---------------------------------------------------------------------------

#[test]
fn test_audit_entry_serialization() {
    let entry = AuditEntry {
        id: Uuid::now_v7(),
        action: "create".into(),
        entity_type: "Session".into(),
        entity_id: Uuid::now_v7().to_string(),
        actor: Some("user-1".into()),
        summary: Some(serde_json::json!({"status": "active"})),
        metadata: HashMap::from([("key".into(), "value".into())]),
        created_at: Utc::now(),
    };

    let json = serde_json::to_value(&entry).expect("serialize AuditEntry");
    assert_eq!(json["action"], "create");
    assert_eq!(json["entityType"], "Session");
    assert!(json.get("metadata").is_some());

    let deserialized: AuditEntry = serde_json::from_value(json).expect("deserialize AuditEntry");
    assert_eq!(deserialized.action, entry.action);
}

#[test]
fn test_new_audit_entry_serialization() {
    let entry = NewAuditEntry {
        action: "update".into(),
        entity_type: "Memory".into(),
        entity_id: Uuid::now_v7().to_string(),
        actor: Some("test".into()),
        summary: Some(serde_json::json!({"field": "content"})),
    };

    let json = serde_json::to_value(&entry).expect("serialize NewAuditEntry");
    assert_eq!(json["action"], "update");

    let deserialized: NewAuditEntry =
        serde_json::from_value(json).expect("deserialize NewAuditEntry");
    assert_eq!(deserialized.action, entry.action);
}

#[test]
fn test_audit_filter_serialization() {
    let filter = AuditFilter {
        entity_type: Some("Session".into()),
        entity_id: Some("abc-123".into()),
        actor: Some("user".into()),
        limit: 10,
        offset: 0,
    };

    let json = serde_json::to_value(&filter).expect("serialize AuditFilter");
    assert_eq!(json["entityType"], "Session");
    assert_eq!(json["limit"], 10);

    let deserialized: AuditFilter =
        serde_json::from_value(json).expect("deserialize AuditFilter");
    assert_eq!(deserialized.entity_type, filter.entity_type);
}

// ---------------------------------------------------------------------------
// Other entity models
// ---------------------------------------------------------------------------

#[test]
fn test_notification_serialization() {
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

#[test]
fn test_feedback_serialization() {
    let feedback = Feedback {
        id: Uuid::now_v7(),
        target_id: Uuid::now_v7(),
        rating: 5,
        comment: Some("Great!".into()),
        actor: Some("user123".into()),
        created_at: Utc::now(),
    };

    let json = serde_json::to_value(&feedback).expect("serialize Feedback");
    assert_eq!(json["rating"], 5);
    assert_eq!(json["comment"], "Great!");

    let deserialized: Feedback = serde_json::from_value(json).expect("deserialize Feedback");
    assert_eq!(deserialized.id, feedback.id);
}

#[test]
fn test_correlation_serialization() {
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
}

#[test]
fn test_telemetry_event_serialization() {
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
    assert_eq!(deserialized.event_type, event.event_type);
}

// ---------------------------------------------------------------------------
// Engine CRUD integration (model-level)
// ---------------------------------------------------------------------------

#[test]
fn test_model_crud_roundtrips() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    // Create Session.
    let session = engine
        .create_session(NewSession {
            project: "model-crud".into(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: None,
        })
        .expect("create session");
    assert_eq!(session.status, SessionStatus::Active);

    // Create Agent.
    let agent = engine
        .create_agent(NewAgent {
            name: "crud-agent".into(),
            agent_type: "test".into(),
            description: "CRUD test agent".into(),
            capabilities: None,
            status: None,
            config: None,
        })
        .expect("create agent");
    assert_eq!(agent.name, "crud-agent");
    assert_eq!(agent.version, 1);

    // Create Skill.
    let skill = engine
        .create_skill(NewSkill {
            name: "crud-skill".into(),
            description: "CRUD test skill".into(),
            category: "test".into(),
            file_path: None,
        })
        .expect("create skill");
    assert_eq!(skill.name, "crud-skill");

    // Create Memory.
    let memory = engine
        .create_memory(NewMemory {
            session_id: session.id,
            agent_id,
            memory_type: MemoryType::Fact,
            content: "CRUD integration test".into(),
            tags: Some(vec!["test".into()]),
        })
        .expect("create memory");
    assert_eq!(memory.memory_type, MemoryType::Fact);

    // Fetch all by ID.
    let fetched_session = engine
        .get_session(session.id)
        .expect("get session")
        .expect("session exists");
    assert_eq!(fetched_session.project, "model-crud");

    let fetched_agent = engine
        .get_agent(agent.id)
        .expect("get agent")
        .expect("agent exists");
    assert_eq!(fetched_agent.name, "crud-agent");

    let fetched_skill = engine
        .get_skill(skill.id)
        .expect("get skill")
        .expect("skill exists");
    assert_eq!(fetched_skill.name, "crud-skill");

    let fetched_memory = engine
        .get_memory(memory.id)
        .expect("get memory")
        .expect("memory exists");
    assert_eq!(fetched_memory.content.to_lowercase(), "crud integration test");
}

// ---------------------------------------------------------------------------
// Default values and edge cases
// ---------------------------------------------------------------------------

#[test]
fn test_model_default_values() {
    // SessionFilter defaults.
    let filter = SessionFilter::default();
    assert_eq!(filter.limit, 100);
    assert_eq!(filter.offset, 0);

    // MemorySearchQuery defaults.
    let query = MemorySearchQuery::default();
    assert_eq!(query.limit, 100);
    assert_eq!(query.offset, 0);

    // AuditFilter defaults.
    let audit_filter = AuditFilter::default();
    assert_eq!(audit_filter.limit, 100);
    assert_eq!(audit_filter.offset, 0);

    // AgentFilter defaults.
    let agent_filter = AgentFilter::default();
    assert_eq!(agent_filter.limit, 100);
    assert_eq!(agent_filter.offset, 0);

    // SkillFilter defaults.
    let skill_filter = SkillFilter::default();
    assert_eq!(skill_filter.limit, 100);
    assert_eq!(skill_filter.offset, 0);

    // SessionPatch defaults.
    let patch = SessionPatch::default();
    assert!(patch.status.is_none());
    assert!(patch.turn_count.is_none());

    // MemoryPatch defaults.
    let mem_patch = MemoryPatch::default();
    assert!(mem_patch.content.is_none());
}

#[test]
fn test_model_unknown_variants_rejected() {
    // Unknown SessionStatus variant.
    let result: Result<SessionStatus, _> = serde_json::from_str("\"unknown_status\"");
    assert!(result.is_err(), "unknown SessionStatus variant should fail");

    // Unknown MemoryType variant.
    let result: Result<MemoryType, _> = serde_json::from_str("\"unknown_type\"");
    assert!(result.is_err(), "unknown MemoryType variant should fail");

    // Unknown AgentStatus variant.
    let result: Result<AgentStatus, _> = serde_json::from_str("\"unknown\"");
    assert!(result.is_err(), "unknown AgentStatus variant should fail");
}
