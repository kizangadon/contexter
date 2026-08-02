//! Integration tests for agent and skill CRUD — lifecycle, cache invalidation,
//! and file_path validation.

use contexter_core::{
    AgentFilter, AgentPatch, AgentStatus, NewAgent, NewSession, NewSkill, SessionFilter,
    SessionStatus, SkillFilter, SkillPatch,
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
// Count agents / skills (REQ-ACE-001: dedicated store-backed counters)
// ---------------------------------------------------------------------------

#[test]
fn test_count_agents_matches_store() {
    let (engine, _dir) = common::setup_engine();

    for i in 0..3 {
        engine
            .create_agent(NewAgent {
                name: format!("count-agent-{i}"),
                agent_type: "chat".into(),
                description: "count test".into(),
                capabilities: None,
                status: Some(AgentStatus::Active),
                config: None,
            })
            .expect("create agent");
    }

    let count = engine
        .count_agents(&AgentFilter::default())
        .expect("count agents");
    assert_eq!(count, 3, "unfiltered count must match the store");
}

#[test]
fn test_count_agents_with_status_filter() {
    let (engine, _dir) = common::setup_engine();

    for i in 0..2 {
        engine
            .create_agent(NewAgent {
                name: format!("active-{i}"),
                agent_type: "chat".into(),
                description: "count test".into(),
                capabilities: None,
                status: Some(AgentStatus::Active),
                config: None,
            })
            .expect("create active agent");
    }
    engine
        .create_agent(NewAgent {
            name: "inactive".into(),
            agent_type: "chat".into(),
            description: "count test".into(),
            capabilities: None,
            status: Some(AgentStatus::Inactive),
            config: None,
        })
        .expect("create inactive agent");

    let active = engine
        .count_agents(&AgentFilter {
            status: Some(AgentStatus::Active),
            ..AgentFilter::default()
        })
        .expect("count active agents");
    assert_eq!(active, 2, "status-filtered count must match the store");

    let all = engine
        .count_agents(&AgentFilter::default())
        .expect("count all agents");
    assert_eq!(all, 3);
}

#[test]
fn test_count_skills_matches_store() {
    let (engine, _dir) = common::setup_engine();

    for i in 0..2 {
        engine
            .create_skill(NewSkill {
                name: format!("count-skill-{i}"),
                description: "count test".into(),
                category: "dev".into(),
                file_path: None,
            })
            .expect("create skill");
    }

    let count = engine
        .count_skills(&SkillFilter::default())
        .expect("count skills");
    assert_eq!(count, 2, "unfiltered count must match the store");
}

#[test]
fn test_count_skills_with_category_filter() {
    let (engine, _dir) = common::setup_engine();

    engine
        .create_skill(NewSkill {
            name: "review".into(),
            description: "count test".into(),
            category: "dev".into(),
            file_path: None,
        })
        .expect("create dev skill");
    engine
        .create_skill(NewSkill {
            name: "translate".into(),
            description: "count test".into(),
            category: "language".into(),
            file_path: None,
        })
        .expect("create language skill");

    let dev = engine
        .count_skills(&SkillFilter {
            category: Some("dev".into()),
            ..SkillFilter::default()
        })
        .expect("count dev skills");
    assert_eq!(dev, 1, "category-filtered count must match the store");

    let all = engine
        .count_skills(&SkillFilter::default())
        .expect("count all skills");
    assert_eq!(all, 2);
}

// ---------------------------------------------------------------------------
// Count sessions (REQ-CS-001..004: unfiltered estimate fast path parity)
// ---------------------------------------------------------------------------

#[test]
fn test_count_sessions_matches_store() {
    let (engine, _dir) = common::setup_engine();

    // Interleave agents/skills with sessions to prove the sessions CF count
    // is independent of other entity stores (AC-CS-001).
    engine
        .create_agent(NewAgent {
            name: "unrelated-agent".into(),
            agent_type: "chat".into(),
            description: "count test".into(),
            capabilities: None,
            status: Some(AgentStatus::Active),
            config: None,
        })
        .expect("create agent");

    let agent_id = Uuid::now_v7();
    for i in 0..3 {
        engine
            .create_session(NewSession {
                project: format!("count-project-{i}"),
                agent_id,
                status: Some(SessionStatus::Active),
                metadata: None,
            })
            .expect("create session");
    }

    let count = engine
        .count_sessions(&SessionFilter::default())
        .expect("count sessions");
    assert_eq!(count, 3, "unfiltered count must match the store");
}

#[test]
fn test_count_sessions_empty_store_returns_zero() {
    let (engine, _dir) = common::setup_engine();

    let count = engine
        .count_sessions(&SessionFilter::default())
        .expect("count sessions");
    assert_eq!(count, 0, "empty store must count zero");
}

#[test]
fn test_count_sessions_with_project_filter() {
    let (engine, _dir) = common::setup_engine();
    let agent_id = Uuid::now_v7();

    for i in 0..3 {
        engine
            .create_session(NewSession {
                project: "alpha".into(),
                agent_id,
                status: Some(SessionStatus::Active),
                metadata: None,
            })
            .expect("create alpha session");
    }
    for i in 0..2 {
        engine
            .create_session(NewSession {
                project: "beta".into(),
                agent_id,
                status: Some(SessionStatus::Active),
                metadata: None,
            })
            .expect("create beta session");
    }

    let alpha = engine
        .count_sessions(&SessionFilter {
            project: Some("alpha".into()),
            ..SessionFilter::default()
        })
        .expect("count alpha sessions");
    assert_eq!(alpha, 3, "project-filtered count must match the store");

    let beta = engine
        .count_sessions(&SessionFilter {
            project: Some("beta".into()),
            ..SessionFilter::default()
        })
        .expect("count beta sessions");
    assert_eq!(beta, 2, "project-filtered count must match the store");

    let all = engine
        .count_sessions(&SessionFilter::default())
        .expect("count all sessions");
    assert_eq!(all, 5);
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