//! Cache telemetry and metrics.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Snapshot of cache performance counters.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheTelemetry {
    /// Total number of `get` calls.
    pub gets: u64,
    /// Total number of `get` calls that resulted in a cache hit.
    pub hits: u64,
    /// Total number of `get` calls that resulted in a cache miss.
    pub misses: u64,
    /// Total number of `store` calls.
    pub stores: u64,
    /// Total number of `invalidate` calls.
    pub invalidations: u64,
    /// Total number of cache operations (convenience, equal to `gets`).
    pub total_ops: u64,
    /// Current number of entries, grouped by entity type.
    pub entries_by_type: HashMap<String, u64>,
}

/// Tracks atomic cache performance counters.
#[derive(Debug, Default)]
pub struct CacheCounters {
    pub gets: AtomicU64,
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub stores: AtomicU64,
    pub invalidations: AtomicU64,
}

impl CacheCounters {
    pub fn snapshot(&self) -> (u64, u64, u64, u64, u64) {
        (
            self.gets.load(Ordering::Relaxed),
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
            self.stores.load(Ordering::Relaxed),
            self.invalidations.load(Ordering::Relaxed),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify CacheTelemetry serialization.
    #[test]
    fn cache_telemetry_serialization() {
        let telemetry = CacheTelemetry {
            gets: 100,
            hits: 80,
            misses: 20,
            stores: 50,
            invalidations: 10,
            total_ops: 100,
            entries_by_type: HashMap::from([("session".into(), 5)]),
        };

        let json = serde_json::to_value(&telemetry).expect("serialize CacheTelemetry");
        assert_eq!(json["gets"], 100);
        assert_eq!(json["hits"], 80);
        assert_eq!(
            json["hits"].as_u64().unwrap() as f64 / json["gets"].as_u64().unwrap() as f64,
            0.8
        );
    }

    /// Verify CacheCounters snapshot starts at zero.
    #[test]
    fn cache_counters_initial_snapshot() {
        let counters = CacheCounters::default();
        let (gets, hits, misses, stores, invalidations) = counters.snapshot();
        assert_eq!(gets, 0);
        assert_eq!(hits, 0);
        assert_eq!(misses, 0);
        assert_eq!(stores, 0);
        assert_eq!(invalidations, 0);
    }
}
