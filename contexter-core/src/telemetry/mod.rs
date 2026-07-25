//! Telemetry collection and emission (metrics, event bus).
//!
//! (Stub — Phase 2)

pub mod metrics;
pub mod reporter;
pub mod tracing;

use crate::engine::EngineStats;

/// Collects and exposes engine-wide telemetry.
///
/// Wraps [`EngineStats`] and provides a foundation for metrics, reporting,
/// and tracing subsystems (Phase 2).
pub struct TelemetryCollector {
    pub stats: EngineStats,
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            stats: EngineStats::default(),
        }
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// A fresh TelemetryCollector must have all counters at zero.
    #[test]
    fn test_telemetry_collector_new_creates_zeroed_stats() {
        let collector = TelemetryCollector::new();
        let snapshot = collector.stats.snapshot();

        // All seven counters must be present and zero.
        assert_eq!(snapshot["sessions_created"], 0);
        assert_eq!(snapshot["sessions_deleted"], 0);
        assert_eq!(snapshot["memories_created"], 0);
        assert_eq!(snapshot["memories_deleted"], 0);
        assert_eq!(snapshot["searches_completed"], 0);
        assert_eq!(snapshot["store_ops"], 0);
        assert_eq!(snapshot["get_ops"], 0);
    }

    /// The snapshot HashMap must contain exactly the seven expected keys.
    #[test]
    fn test_telemetry_collector_snapshot_has_all_keys() {
        let collector = TelemetryCollector::new();
        let snapshot = collector.stats.snapshot();

        let expected: HashMap<&str, u64> = [
            ("sessions_created", 0),
            ("sessions_deleted", 0),
            ("memories_created", 0),
            ("memories_deleted", 0),
            ("searches_completed", 0),
            ("store_ops", 0),
            ("get_ops", 0),
        ]
        .into();

        assert_eq!(snapshot.len(), expected.len());
        for (key, val) in &expected {
            assert_eq!(
                snapshot.get(*key),
                Some(val),
                "missing or wrong value for key `{key}`"
            );
        }
    }

    /// Verify that stats can be mutated through the TelemetryCollector
    /// (interior mutability via Arc).
    #[test]
    fn test_telemetry_collector_stats_mutable() {
        let collector = TelemetryCollector::new();
        use std::sync::atomic::Ordering;

        collector
            .stats
            .sessions_created
            .fetch_add(1, Ordering::Relaxed);
        collector
            .stats
            .memories_created
            .fetch_add(5, Ordering::Relaxed);

        let snapshot = collector.stats.snapshot();
        assert_eq!(snapshot["sessions_created"], 1);
        assert_eq!(snapshot["memories_created"], 5);
    }

    /// TelemetryCollector must be Send + Sync (it wraps Arc-compatible types).
    #[test]
    fn test_telemetry_collector_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<TelemetryCollector>();
        assert_sync::<TelemetryCollector>();
    }

    /// Default implementation must be equivalent to new().
    #[test]
    fn test_telemetry_collector_default_equals_new() {
        let a = TelemetryCollector::new();
        let b = TelemetryCollector::default();
        assert_eq!(a.stats.snapshot(), b.stats.snapshot());
    }

    /// Arc<TelemetryCollector> can be shared across threads.
    #[test]
    fn test_telemetry_collector_arc_compatible() {
        let collector = Arc::new(TelemetryCollector::new());
        let snapshot = collector.stats.snapshot();
        assert_eq!(snapshot["sessions_created"], 0);
    }
}
