//! Integration tests for agent and skill CRUD — lifecycle, cache invalidation,
//! and file_path validation.

use contexter_core::{
    AgentFilter, AgentPatch, AgentStatus, NewAgent, NewSkill, SkillFilter, SkillPatch,
};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Agent CRUD
// ---------------------------------------------------------------------------

#[test]
fn test_agent_skill_roundtrip() {
    let (engine, _dir) = common::setup_engine();

    let agent = engine
        .create_agent(NewAgent {
            name: "test-agent".into(),
            agent_type: "chat".into(),
            description: "A test agent".into(),
            capabilities: Some(vec!["code".into(), "search".into()]),
            status: Some(AgentStatus::Active),
            config: Some(serde_json::json!({"model": "gpt-4"})),
        })
        .expect("create agent");
    assert_eq!(agent.name, "test-agent");
    assert_eq!(agent.version, 1);
    assert!(agent.capabilities.contains(&"code".to_string()));

    // Get agent (cache-aside — should hit L1 after write-through).
    let fetched = engine
        .get_agent(agent.id)
        .expect("get agent")
        .expect("agent exists");
    assert_eq!(fetched.name, "test-agent");

    // Update agent (write-around).
    let updated = engine
        .update_agent(
            agent.id,
            &AgentPatch {
                name: Some("updated-agent".into()),
                ..AgentPatch::default()
            },
        )
        .expect("update agent");
    assert_eq!(updated.name, "updated-agent");

    // List agents.
    let agents = engine
        .list_agents(&AgentFilter::default())
        .expect("list agents");
    assert!(agents.iter().any(|a| a.name == "updated-agent"));

    // Delete agent.
    engine.delete_agent(agent.id).expect("delete agent");
    assert!(engine
        .get_agent(agent.id)
        .expect("get after delete")
        .is_none());
}

#[test]
fn test_agent_delete_invalidates_cache() {
    let (engine, _dir) = common::setup_engine();
    let agent = engine
        .create_agent(NewAgent {
            name: "del-test".into(),
            agent_type: "test".into(),
            description: "delete test".into(),
            capabilities: None,
            status: None,
            config: None,
        })
        .expect("create");

    // Warm cache.
    let _ = engine.get_agent(agent.id).expect("warm");

    engine.delete_agent(agent.id).expect("delete");
    assert!(engine
        .get_agent(agent.id)
        .expect("get after delete")
        .is_none());
}

// ---------------------------------------------------------------------------
// Skill CRUD
// ---------------------------------------------------------------------------

#[test]
fn test_skill_roundtrip() {
    let (engine, _dir) = common::setup_engine();

    let skill = engine
        .create_skill(NewSkill {
            name: "code-review".into(),
            description: "Review code changes".into(),
            category: "dev".into(),
            file_path: Some("/skills/review.py".into()),
        })
        .expect("create skill");
    assert_eq!(skill.name, "code-review");
    assert_eq!(skill.version, 1);

    // Get skill (cache-aside).
    let fetched = engine
        .get_skill(skill.id)
        .expect("get skill")
        .expect("skill exists");
    assert_eq!(fetched.name, "code-review");

    // Update skill (write-around).
    let updated = engine
        .update_skill(
            skill.id,
            &SkillPatch {
                name: Some("super-review".into()),
                ..SkillPatch::default()
            },
        )
        .expect("update");
    assert_eq!(updated.name, "super-review");

    // List skills.
    let skills = engine
        .list_skills(&SkillFilter::default())
        .expect("list skills");
    assert!(skills.iter().any(|s| s.name == "super-review"));

    // Delete skill.
    engine.delete_skill(skill.id).expect("delete skill");
    assert!(engine
        .get_skill(skill.id)
        .expect("get after delete")
        .is_none());
}

// ---------------------------------------------------------------------------
// Skill file_path validation
// ---------------------------------------------------------------------------

#[test]
fn test_create_skill_with_valid_file_path() {
    let (engine, _dir) = common::setup_engine();
    let skill = engine
        .create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: Some("/home/skills/test.py".into()),
        })
        .expect("create skill with valid file_path");
    assert_eq!(skill.file_path, Some("/home/skills/test.py".into()));
}

#[test]
fn test_create_skill_with_no_file_path() {
    let (engine, _dir) = common::setup_engine();
    let skill = engine
        .create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: None,
        })
        .expect("create skill without file_path");
    assert!(skill.file_path.is_none());
}

#[test]
fn test_create_skill_empty_file_path_rejected() {
    let (engine, _dir) = common::setup_engine();
    let result = engine.create_skill(NewSkill {
        name: "test".into(),
        description: "desc".into(),
        category: "code".into(),
        file_path: Some(String::new()),
    });
    assert!(result.is_err(), "empty file_path should be rejected");
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("file_path"),
        "error should mention file_path: {err}"
    );
}

#[test]
fn test_update_skill_empty_file_path_rejected() {
    let (engine, _dir) = common::setup_engine();
    let skill = engine
        .create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: None,
        })
        .expect("create skill");

    let result = engine.update_skill(
        skill.id,
        &SkillPatch {
            file_path: Some(String::new()),
            ..SkillPatch::default()
        },
    );
    assert!(result.is_err(), "empty file_path on update should be rejected");
}

#[test]
fn test_update_skill_valid_file_path() {
    let (engine, _dir) = common::setup_engine();
    let skill = engine
        .create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: None,
        })
        .expect("create skill");

    let updated = engine
        .update_skill(
            skill.id,
            &SkillPatch {
                file_path: Some("/new/path.py".into()),
                ..SkillPatch::default()
            },
        )
        .expect("update with valid file_path");
    assert_eq!(updated.file_path, Some("/new/path.py".into()));
}

#[test]
fn test_validate_file_path_too_long_rejected() {
    let (engine, _dir) = common::setup_engine();

    let skill = engine
        .create_skill(NewSkill {
            name: "test".into(),
            description: "desc".into(),
            category: "code".into(),
            file_path: Some("a".repeat(4097)),
        })
        .expect_err("file_path over 4096 chars should be rejected");
    let err = skill.to_string();
    assert!(err.contains("4096"), "error should mention 4096 limit: {err}");
}