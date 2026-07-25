use contexter_core::{Engine, EngineConfig, StorageConfig};
use std::sync::Arc;

#[test]
fn test_engine_is_send() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<Engine>();
    assert_sync::<Engine>();
}

#[test]
fn test_engine_arc_compatible() {
    use contexter_core::StorageConfig;
    use tempfile::TempDir;
    let dir = TempDir::new().expect("temp dir");
    let engine = Engine::with_config(EngineConfig {
        storage: StorageConfig {
            path: dir.path().to_path_buf(),
            cache_config: None,
        },
        ..EngineConfig::default()
    })
    .expect("open");
    let _arc = Arc::new(engine);
}
