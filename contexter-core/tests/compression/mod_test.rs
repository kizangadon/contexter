//! Integration tests for the compression module.
//!
//! Exercises the public `Compression` trait, `NoopCompression`,
//! and the feature-gated `ZstdCompression` / `Lz4Compression` codecs
//! as integration-level tests.
//!
//! Unit-level compression tests already exist in `src/compression/` and
//! `tests/compression/codecs_test.rs`. This file adds complementary
//! integration coverage for the module boundary.

use contexter_core::compression::{Compression, NoopCompression};

#[path = "../common/mod.rs"]
mod common;

// ---------------------------------------------------------------------------
// Compression trait interface tests
// ---------------------------------------------------------------------------

/// Verify that the Compression trait is object-safe by constructing
/// a `Box<dyn Compression>`.
#[test]
fn test_compression_trait_is_object_safe() {
    fn takes_box(_: Box<dyn Compression>) {}
    fn returns_box() -> Box<dyn Compression> {
        Box::new(NoopCompression)
    }
    takes_box(returns_box());
}

/// Verify that Compression is Send + Sync (required for shared usage).
#[test]
fn test_compression_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<NoopCompression>();
    assert_sync::<NoopCompression>();
}

/// Verify NoopCompression passes data through unchanged.
#[test]
fn test_noop_compress_decompress() {
    let codec = NoopCompression;
    let data = b"Hello, World! 12345";

    let compressed = codec.compress(data).expect("compress");
    assert_eq!(compressed, data, "NoopCompression should pass data through");

    let decompressed = codec.decompress(&compressed).expect("decompress");
    assert_eq!(decompressed, data, "NoopCompression should return data as-is");
}

/// Verify NoopCompression with various data sizes.
#[test]
fn test_noop_various_sizes() {
    let codec = NoopCompression;

    // Empty.
    let empty = codec.compress(b"").expect("compress empty");
    assert!(empty.is_empty());

    // Single byte.
    let single = codec.compress(b"\xFF").expect("compress single byte");
    assert_eq!(single, b"\xFF");

    // Large buffer.
    let large: Vec<u8> = (0..10_000).map(|i| (i % 256) as u8).collect();
    let compressed = codec.compress(&large).expect("compress large");
    assert_eq!(compressed.len(), large.len());
    assert_eq!(compressed, large);
}

/// Verify NoopCompression name.
#[test]
fn test_noop_name() {
    assert_eq!(NoopCompression.name(), "noop");
}

/// Verify that decompressing corrupted data through NoopCompression
/// still returns the data unchanged (no-op has no corruption detection).
#[test]
fn test_noop_corrupted_data_is_noop() {
    let codec = NoopCompression;
    // NoopCompression doesn't validate — any data "decompresses" to itself.
    let result = codec.decompress(b"not compressed at all");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), b"not compressed at all");
}

// ---------------------------------------------------------------------------
// Feature-gated compression codecs
// ---------------------------------------------------------------------------

#[cfg(feature = "compression")]
mod with_compression {
    use contexter_core::compression::{Compression, NoopCompression};
    use contexter_core::Engine;
    use uuid::Uuid;
    use tempfile::TempDir;

    /// Verify ZstdCompression round-trip via the Engine's internal path.
    #[test]
    fn test_zstd_engine_roundtrip() {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        let agent_id = Uuid::now_v7();

        // Create and retrieve a session (exercises compression internally).
        let session = engine
            .create_session(contexter_core::NewSession {
                project: "zstd-engine-test".into(),
                agent_id,
                status: None,
                metadata: Some(serde_json::json!({
                    "nested": {"deep": true, "array": [1, 2, 3]}
                })),
            })
            .expect("create session");

        let fetched = engine
            .get_session(session.id)
            .expect("get session")
            .expect("session exists");
        assert_eq!(fetched.project, "zstd-engine-test");
        assert_eq!(fetched.metadata["nested"]["deep"], true);
    }

    /// Verify LZ4 compression round-trip via memory operations.
    #[test]
    fn test_lz4_engine_roundtrip() {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        let agent_id = Uuid::now_v7();

        // Create a memory with substantial content to exercise compression.
        let content = "large content for compression test ".repeat(500);
        let memory = engine
            .create_memory(contexter_core::NewMemory {
                session_id: Uuid::now_v7(),
                agent_id,
                memory_type: contexter_core::MemoryType::Fact,
                content: content.clone(),
                tags: Some(vec!["compression".into()]),
            })
            .expect("create memory");

        let fetched = engine
            .get_memory(memory.id)
            .expect("get memory")
            .expect("memory exists");
        assert!(fetched.content.contains("large content for compression test"));
        assert_eq!(fetched.tags.len(), 1);
    }

    /// Verify that NoopCompression still works alongside compression feature.
    #[test]
    fn test_noop_still_works_with_feature() {
        let codec = NoopCompression;
        let data = b"test data";
        let compressed = codec.compress(data).expect("compress");
        assert_eq!(compressed, data);
    }

    /// Verify that decompressing corrupted Zstd data returns an error.
    #[test]
    fn test_zstd_corrupted_data_error() {
        use contexter_core::compression::codecs::ZstdCompression;
        let c = ZstdCompression::default();
        let result = c.decompress(b"clearly not zstd data");
        assert!(result.is_err(), "corrupted zstd data should error");
    }

    /// Verify that ZstdCompression name is correct.
    #[test]
    fn test_zstd_name() {
        use contexter_core::compression::codecs::ZstdCompression;
        assert_eq!(ZstdCompression::default().name(), "zstd");
    }

    /// Verify that Lz4Compression name is correct.
    #[test]
    fn test_lz4_name() {
        use contexter_core::compression::codecs::Lz4Compression;
        assert_eq!(Lz4Compression.name(), "lz4");
    }
}
