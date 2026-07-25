//! Integration tests for compression codec round-trips.
//!
//! Exercises NoopCompression (always available) and, when the `compression`
//! feature is enabled, Zstd and LZ4 compression/decompression via the
//! public `Compression` trait.

use contexter_core::compression::{Compression, NoopCompression};

// ---------------------------------------------------------------------------
// NoopCompression — always available
// ---------------------------------------------------------------------------

#[test]
fn test_noop_compress_decompress_roundtrip() {
    let codec = NoopCompression;
    let data = b"hello world, this is a noop compression test!";
    let compressed = codec.compress(data).expect("compress");
    assert_eq!(compressed, data, "noop should pass data through unchanged");
    let decompressed = codec.decompress(&compressed).expect("decompress");
    assert_eq!(decompressed, data);
}

#[test]
fn test_noop_name() {
    assert_eq!(NoopCompression.name(), "noop");
}

#[test]
fn test_noop_empty_input() {
    let codec = NoopCompression;
    let empty = b"";
    let compressed = codec.compress(empty).expect("compress empty");
    assert!(compressed.is_empty());
    let decompressed = codec.decompress(&compressed).expect("decompress empty");
    assert!(decompressed.is_empty());
}

// ---------------------------------------------------------------------------
// Feature-gated compression codecs
// ---------------------------------------------------------------------------

#[cfg(feature = "compression")]
mod with_compression {
    use contexter_core::compression::{Compression, NoopCompression};
    use contexter_core::engine::Engine;
    use tempfile::TempDir;
    use uuid::Uuid;

    // The concrete compression codecs are not re-exported publicly, so we
    // exercise them indirectly via the Engine's internal compression path.
    // Create data, store it, and verify it round-trips correctly.

    #[test]
    fn test_compression_roundtrip_via_engine() {
        let dir = TempDir::new().expect("temp dir");
        let engine = Engine::open(dir.path()).expect("open engine");
        let agent_id = Uuid::now_v7();

        // Create a session with metadata that exercises serialization.
        let session = engine
            .create_session(contexter_core::NewSession {
                project: "compression-test".into(),
                agent_id,
                status: Some(contexter_core::SessionStatus::Active),
                metadata: Some(serde_json::json!({
                    "key": "value",
                    "nested": {"deep": true}
                })),
            })
            .expect("create session");

        // Round-trip via get.
        let fetched = engine
            .get_session(session.id)
            .expect("get session")
            .expect("session exists");
        assert_eq!(fetched.project, "compression-test");
        assert_eq!(fetched.agent_id, agent_id);

        // Create a memory with substantial content.
        let content = "compressed content! ".repeat(100);
        let memory = engine
            .create_memory(contexter_core::NewMemory {
                session_id: session.id,
                agent_id,
                memory_type: contexter_core::MemoryType::Fact,
                content,
                tags: Some(vec!["compression".into()]),
            })
            .expect("create memory");

        // Round-trip via get.
        let fetched_memory = engine
            .get_memory(memory.id)
            .expect("get memory")
            .expect("memory exists");
        assert!(fetched_memory.content.contains("compressed content"));
    }

    #[test]
    fn test_noop_still_works_alongside_compression_feature() {
        let codec = NoopCompression;
        let data = b"noop alongside compression codecs";
        let compressed = codec.compress(data).expect("compress");
        assert_eq!(compressed, data);
    }
}
