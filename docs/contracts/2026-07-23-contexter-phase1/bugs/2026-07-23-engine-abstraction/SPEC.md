# Bug 1: Engine Abstraction + Generic KV + StorageConfig

## Problem
The `Engine` struct uses a concrete `RocksDbBackend` instead of `Box<dyn StorageBackend>`, lacks generic `store`/`get` KV methods, and has no `StorageConfig` struct. Default data path is `./contexter_data` instead of `~/.contexter/`.

## Fix Requirements
1. Add `SharedBackend` type alias: `pub type SharedBackend = Arc<RwLock<Box<dyn StorageBackend>>>`
2. Change `Engine` to hold `SharedBackend` instead of `RocksDbBackend`
3. Add generic `store(cf: &str, key: &str, value: &[u8])` and `get(cf: &str, key: &str)` to `Engine` + Python API
4. Add `StorageConfig` struct with `path: PathBuf` and optional `cache_config: Option<CacheConfig>`
5. Change default CLI path to `~/.contexter/`
6. Ensure all existing tests still pass
7. Clippy must be clean
