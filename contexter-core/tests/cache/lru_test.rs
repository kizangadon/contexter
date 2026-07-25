//! Integration tests for the DashMap + LRU cache layer — eviction, hit/miss
//! tracking, and invalidation on updates and deletes.

use std::collections::HashMap;

use contexter_core::{CacheConfig, SessionPatch, SessionStatus};
use uuid::Uuid;

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Cache eviction and hit/miss tracking
// ---------------------------------------------------------------------------

#[test]
fn test_cache_behavior() {
    // Configure a small per-type capacity (5 for sessions).
    let mut per_type = HashMap::new();
    per_type.insert("session".to_string(), 5usize);
    let config = CacheConfig {
        default_capacity: 100,
        per_type_capacity: per_type,
        max_ttl: None,
    };
    let (engine, _dir) = common::setup_engine_with_config(config);
    let agent_id = Uuid::now_v7();

    // Create one session — write-through caches it.
    let s1 = common::create_session(&engine, "cache-test", agent_id);

    // Get should be a cache hit (write-through).
    let tel = engine.cache_telemetry();
    let _ = engine
        .get_session(s1.id)
        .expect("get session")
        .expect("exists");
    let tel2 = engine.cache_telemetry();
    assert_eq!(
        tel2.hits - tel.hits,
        1,
        "get after write-through should be a hit"
    );

    // Fill sessions beyond capacity (5). Creating 5 more = 6 total.
    // The LRU should evict the oldest (s1).
    for _ in 0..5 {
        common::create_session(&engine, "cache-test", Uuid::now_v7());
    }

    // Clear telemetry counters to get a clean reading.
    let tel_before = engine.cache_telemetry();

    // s1 should have been evicted from the cache → miss on get.
    let _ = engine.get_session(s1.id).expect("get session");

    let tel_mid = engine.cache_telemetry();
    assert!(
        tel_mid.misses > tel_before.misses,
        "evicted session should cause a cache miss"
    );

    // But the session still exists in storage.
    let from_storage = engine
        .get_session(s1.id)
        .expect("get session")
        .expect("session should exist in storage");
    assert_eq!(from_storage.id, s1.id);

    // Verify hit ratio tracking is non-zero (at least some ops happened).
    let tel_final = engine.cache_telemetry();
    assert!(
        tel_final.total_ops > 0,
        "cache should have tracked some ops"
    );
    assert!(
        tel_final.hits + tel_final.misses == tel_final.total_ops,
        "hits + misses should equal total_ops"
    );

    // Update session — write-around should invalidate the cache.
    // First warm the cache again.
    let _ = engine.get_session(s1.id).expect("get session");
    let tel_before_update = engine.cache_telemetry();

    let _updated = engine
        .update_session(
            s1.id,
            &SessionPatch {
                turn_count: Some(99),
                ..SessionPatch::default()
            },
        )
        .expect("update session");

    // After update (invalidation), get should be a miss.
    let _ = engine.get_session(s1.id).expect("get session");
    let tel_after_update = engine.cache_telemetry();
    assert!(
        tel_after_update.misses > tel_before_update.misses,
        "update should invalidate cache, causing a miss"
    );

    // Delete — verify cache invalidation.
    engine.delete_session(s1.id).expect("delete session");
    // After delete + invalidation, get returns None.
    let deleted = engine.get_session(s1.id).expect("get after delete");
    assert!(deleted.is_none());
}
