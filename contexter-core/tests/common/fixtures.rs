//! Reusable test data factories for integration tests.
//!
//! Import via `mod fixtures` from `tests/common/mod.rs`.
//!
//! Each factory creates domain entities with sensible defaults so callers
//! only need to supply the fields that matter for their test case.

use contexter_core::*;
use tempfile::TempDir;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Create a temporary [`Engine`] with default configuration.
///
/// The returned [`TempDir`] is kept alive for the lifetime of the test —
/// dropping it cleans up the RocksDB storage directory.
///
/// # Example
///
/// ```ignore
/// let (engine, _dir) = fixtures::setup_engine();
/// ```
pub fn setup_engine() -> (Engine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::open(dir.path()).expect("open engine");
    (engine, dir)
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// Create a session with a generated agent ID and `Active` status.
///
/// # Example
///
/// ```ignore
/// let (engine, _dir) = fixtures::setup_engine();
/// let session = fixtures::create_session(&engine, "my-project");
/// ```
pub fn create_session(engine: &Engine, project: &str) -> Session {
    engine
        .create_session(NewSession {
            project: project.to_string(),
            agent_id: Uuid::now_v7(),
            status: Some(SessionStatus::Active),
            metadata: None,
        })
        .expect("create session")
}

// ---------------------------------------------------------------------------
// Memory
// ---------------------------------------------------------------------------

/// Create a memory with `Fact` type, a generated agent ID, and no tags.
///
/// # Example
///
/// ```ignore
/// let (engine, _dir) = fixtures::setup_engine();
/// let session = fixtures::create_session(&engine, "p");
/// let mem = fixtures::create_memory(&engine, session.id, "hello");
/// ```
pub fn create_memory(engine: &Engine, session_id: Uuid, content: &str) -> Memory {
    engine
        .create_memory(NewMemory {
            session_id,
            agent_id: Uuid::now_v7(),
            memory_type: MemoryType::Fact,
            content: content.to_string(),
            tags: None,
        })
        .expect("create memory")
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Register an agent with sensible defaults.
///
/// Defaults:
/// - `agent_type`: `"default"`
/// - `description`: `"Agent: {name}"`
/// - `capabilities`: empty vec
/// - `status`: `Active`
/// - `config`: empty JSON object `{}`
///
/// # Example
///
/// ```ignore
/// let (engine, _dir) = fixtures::setup_engine();
/// let agent = fixtures::create_agent(&engine, "my-bot");
/// ```
pub fn create_agent(engine: &Engine, name: &str) -> Agent {
    engine
        .create_agent(NewAgent {
            name: name.to_string(),
            agent_type: "default".into(),
            description: format!("Agent: {name}"),
            capabilities: Some(vec![]),
            status: Some(AgentStatus::Active),
            config: Some(serde_json::Value::Object(Default::default())),
        })
        .expect("create agent")
}

// ---------------------------------------------------------------------------
// Skill
// ---------------------------------------------------------------------------

/// Register a skill with sensible defaults.
///
/// Defaults:
/// - `description`: `"Skill: {name}"`
/// - `category`: `"general"`
/// - `file_path`: `None`
///
/// # Example
///
/// ```ignore
/// let (engine, _dir) = fixtures::setup_engine();
/// let skill = fixtures::create_skill(&engine, "code-review");
/// ```
pub fn create_skill(engine: &Engine, name: &str) -> Skill {
    engine
        .create_skill(NewSkill {
            name: name.to_string(),
            description: format!("Skill: {name}"),
            category: "general".into(),
            file_path: None,
        })
        .expect("create skill")
}
