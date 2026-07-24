//! Per-type LRU hot cache tier (L1) using DashMap for concurrent access.

pub mod dashmap_lru;
pub mod metrics;

pub use dashmap_lru::{CachedValue, DashMapCache};
pub use metrics::CacheTelemetry;

use std::collections::HashMap;
use std::time::Duration;

/// Tuning parameters for [`DashMapCache`].
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Default per-type capacity when no override exists (default: `10_000`).
    pub default_capacity: usize,
    /// Per-entity-type capacity overrides.
    pub per_type_capacity: HashMap<String, usize>,
    /// Optional maximum TTL for cache entries.
    pub max_ttl: Option<Duration>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        let mut per_type = HashMap::new();
        per_type.insert("session".into(), 5_000);
        per_type.insert("memory".into(), 10_000);
        Self {
            default_capacity: 10_000,
            per_type_capacity: per_type,
            max_ttl: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify CacheConfig default values.
    #[test]
    fn cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.default_capacity, 10_000);
        assert!(config.max_ttl.is_none());
        assert_eq!(config.per_type_capacity.get("session"), Some(&5_000));
        assert_eq!(config.per_type_capacity.get("memory"), Some(&10_000));
    }
}
