//! Integration tests for the analytics engine (L5 DuckDB).
//!
//! Tests the `Engine` analytics methods that delegate to the DuckDB engine
//! for efficiency scoring, metric correlation, and data aggregation.

use contexter_core::{
    Engine, EngineConfig, MemoryType, NewMemory, NewSession, SessionPatch, SessionStatus,
    StorageConfig,
};
use tempfile::TempDir;
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create an Engine with analytics enabled.
fn setup_analytics_engine(dir: &TempDir) -> Engine {
    Engine::with_config(EngineConfig {
        storage: StorageConfig {
            path: dir.path().join("rocksdb"),
            cache_config: None,
        },
        enable_analytics: true,
        analytics_cache_ttl_secs: 3600,
        ..EngineConfig::default()
    })
    .expect("create analytics engine")
}

/// Insert two sessions and five memories into the engine, matching the
/// sample-data pattern that the original DuckDbEngine unit tests use.
fn insert_test_data(engine: &Engine) -> (Uuid, Uuid) {
    let agent_id = Uuid::now_v7();

    // Session 1: completed, 10 turns, 60s duration
    let s1 = engine
        .create_session(NewSession {
            project: "contexter".into(),
            agent_id,
            status: Some(SessionStatus::Completed),
            metadata: None,
        })
        .expect("create session 1");
    engine
        .update_session(
            s1.id,
            &SessionPatch {
                turn_count: Some(10),
                duration_ms: Some(60000),
                ..SessionPatch::default()
            },
        )
        .expect("update session 1");

    // Session 2: active, 5 turns, 30s duration
    let s2 = engine
        .create_session(NewSession {
            project: "contexter".into(),
            agent_id,
            status: Some(SessionStatus::Active),
            metadata: None,
        })
        .expect("create session 2");
    engine
        .update_session(
            s2.id,
            &SessionPatch {
                turn_count: Some(5),
                duration_ms: Some(30000),
                ..SessionPatch::default()
            },
        )
        .expect("update session 2");

    // 5 memories: session-1 gets 3, session-2 gets 2
    for (i, (sid, mtype, tags)) in [
        (s1.id, MemoryType::Preference, vec!["important"]),
        (s1.id, MemoryType::Fact, vec!["general"]),
        (s1.id, MemoryType::Episode, vec!["chat"]),
        (s2.id, MemoryType::Fact, vec!["general"]),
        (s2.id, MemoryType::Preference, vec!["important"]),
    ]
    .into_iter()
    .enumerate()
    {
        engine
            .create_memory(NewMemory {
                session_id: sid,
                agent_id,
                memory_type: mtype,
                content: format!("test memory {}", i + 1),
                tags: Some(tags.iter().map(|s| s.to_string()).collect()),
            })
            .expect("create memory");
    }

    (s1.id, s2.id)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Analytics returns error when not enabled.
#[test]
fn test_analytics_disabled_by_default() {
    let (engine, _dir) = common::setup_engine();

    let result = engine.run_analytics();
    assert!(
        result.is_err(),
        "analytics should fail when not enabled"
    );

    let result = engine.get_efficiency_scores();
    assert!(
        result.is_err(),
        "efficiency scores should fail when not enabled"
    );

    let result = engine.get_metric_correlation();
    assert!(
        result.is_err(),
        "metric correlation should fail when not enabled"
    );
}

/// Running a full analytics report returns efficiency and correlation data
/// sourced from the real RocksDB backend.
#[test]
fn test_analytics_run_report() {
    let dir = TempDir::new().expect("temp dir");
    let engine = setup_analytics_engine(&dir);

    insert_test_data(&engine);

    let report = engine.run_analytics().expect("run analytics");

    // 2 sessions → 2 efficiency scores.
    assert_eq!(
        report.efficiency_scores.len(),
        2,
        "expected 2 efficiency scores"
    );

    // 2 samples for correlation.
    assert!(
        report.correlation.sample_count >= 2,
        "expected >= 2 samples in correlation"
    );

    // 5 memories across types.
    assert!(
        !report.memory_count_by_type.is_empty(),
        "expected memory counts by type"
    );

    // Session status counts: 1 completed, 1 active.
    assert!(
        !report.session_count_by_type.is_empty(),
        "expected session counts by status"
    );
}

/// Verify per-session efficiency values from real RocksDB data.
#[test]
fn test_efficiency_scores() {
    let dir = TempDir::new().expect("temp dir");
    let engine = setup_analytics_engine(&dir);

    // Session 1: 3 memories (1 preference) → efficiency = 1/3 ≈ 0.333
    // Session 2: 2 memories (1 preference) → efficiency = 1/2 = 0.5
    let (_s1_id, s2_id) = insert_test_data(&engine);

    let scores = engine
        .get_efficiency_scores()
        .expect("get efficiency scores");

    assert_eq!(scores.len(), 2, "expected 2 sessions");

    // Ordered by efficiency DESC → session-2 (0.5) first.
    assert_eq!(scores[0].session_id, s2_id.to_string());
    assert_eq!(scores[0].project, "contexter");
    assert_eq!(scores[0].total_memories, 2);
    assert_eq!(scores[0].useful_memories, 1);
    assert!(
        (scores[0].efficiency_score - 0.5).abs() < 1e-9,
        "expected 0.5, got {}",
        scores[0].efficiency_score
    );

    // Efficiency scores are ordered DESC, so session-2 is first.
    // Assert session-1 values from the second entry.
    assert_eq!(scores[1].project, "contexter");
    assert_eq!(scores[1].total_memories, 3);
    assert_eq!(scores[1].useful_memories, 1);
    assert!(
        (scores[1].efficiency_score - (1.0 / 3.0)).abs() < 1e-9,
        "expected ~0.333, got {}",
        scores[1].efficiency_score
    );
}

/// Verify metric correlation coefficient is in [-1, 1] when backed by real
/// RocksDB data.
#[test]
fn test_metric_correlation() {
    let dir = TempDir::new().expect("temp dir");
    let engine = setup_analytics_engine(&dir);

    insert_test_data(&engine);

    let correlation = engine
        .get_metric_correlation()
        .expect("get metric correlation");

    // The correlation coefficient must be in [-1.0, 1.0].
    assert!(
        correlation.pearson_r >= -1.0 && correlation.pearson_r <= 1.0,
        "Pearson r must be in [-1, 1], got {}",
        correlation.pearson_r
    );

    // Sample count should be positive (2 sessions = 2 samples).
    assert!(
        correlation.sample_count > 0,
        "sample count should be > 0"
    );
}

/// Get memory counts by type from real RocksDB data.
#[test]
fn test_memory_count_by_type() {
    let dir = TempDir::new().expect("temp dir");
    let engine = setup_analytics_engine(&dir);

    insert_test_data(&engine);

    let counts = engine
        .get_memory_count_by_type()
        .expect("get memory count by type");

    // 5 memories: 2 fact, 2 preference, 1 episode.
    assert!(!counts.is_empty(), "expected memory type counts");

    let total: u64 = counts.iter().map(|(_, c)| c).sum();
    assert_eq!(total, 5, "expected 5 total memories");
}

/// Telemetry aggregation — telemetry events are not yet written through the
/// Engine API, so the aggregation returns empty (correct behaviour until a
/// telemetry record API exists).
#[test]
fn test_telemetry_aggregation() {
    let dir = TempDir::new().expect("temp dir");
    let engine = setup_analytics_engine(&dir);

    let agg = engine
        .get_telemetry_aggregation()
        .expect("get telemetry aggregation");

    // No telemetry API exists yet so results are empty. This assertion
    // serves as a baseline that will need updating once telemetry events
    // are persisted through the Engine.
    assert!(agg.is_empty(), "expected no telemetry without injected events");
}
