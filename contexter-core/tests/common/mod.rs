//! Shared test helpers for integration tests.
//!
//! Each test file in `tests/` includes this module via `mod common;`
//! to access `setup_engine()`, `setup_engine_with_config()`, and
//! `create_session()`.

use std::collections::HashMap;

use contexter_core::{CacheConfig, Engine, EngineConfig, NewSession, Session, SessionStatus, StorageConfig};
use tempfile::TempDir;
use uuid::Uuid;

/// Create a temporary Engine with default configuration.
pub fn setup_engine() -> (Engine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::open(dir.path()).expect("open engine");
    (engine, dir)
}

/// Create a temporary Engine with a custom cache config.
/// All optional tiers (vector, FTS, analytics) are disabled.
pub fn setup_engine_with_config(config: CacheConfig) -> (Engine, TempDir) {
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::with_config(EngineConfig {
        storage: StorageConfig {
            path: dir.path().to_path_buf(),
            cache_config: Some(config),
        },
        ..EngineConfig::default()
    })
    .expect("open with config");
    (engine, dir)
}

/// Helper to create a session with the given project and agent.
pub fn create_session(engine: &Engine, project: &str, agent_id: Uuid) -> Session {
    engine
        .create_session(NewSession {
            project: project.to_string(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: None,
        })
        .expect("create session")
}

// ---------------------------------------------------------------------------
// Reusable test data factories
// ---------------------------------------------------------------------------

/// Convenience factories for quickly creating domain entities with sensible
/// defaults. Available as `common::fixtures::*`.
#[path = "fixtures.rs"]
pub mod fixtures;
