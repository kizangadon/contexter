# Design Preview — Bug-Efficiency

## Fix Plan
1. Add `EFFICIENCY_CF = "efficiency_map"` constant
2. Add `efficiency_cache: Arc<RwLock<HashMap<String, (f64, Instant)>>>` field to DuckDbEngine
3. In `DuckDbEngine::sync("efficiency_map")`, iterate CF entries and populate cache
4. In `get_efficiency_scores()`, check cache first; if expired or missing, fall through to DuckDB query, then repopulate cache
5. Cache TTL from `analytics_cache_ttl_secs` config
