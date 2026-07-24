//! Data compression/decompression abstraction and codecs.

pub mod codecs;

use crate::error::EngineError;

/// Trait for compressing and decompressing byte slices.
///
/// Implementations should be safe to use from multiple threads.
pub trait Compression: Send + Sync {
    /// Compress a slice of bytes, returning a new owned buffer.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError>;

    /// Decompress a previously compressed slice back to its original form.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError>;

    /// Human-readable name of the codec (e.g. `"zstd"`, `"lz4"`, `"noop"`).
    fn name(&self) -> &'static str;
}

/// A no-op codec that passes data through unchanged.
#[derive(Debug, Clone)]
pub struct NoopCompression;

impl Compression for NoopCompression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
        Ok(data.to_vec())
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
        Ok(data.to_vec())
    }

    fn name(&self) -> &'static str {
        "noop"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify NoopCompression passes data through unchanged.
    #[test]
    fn noop_compress_decompress_roundtrip() {
        let data = b"hello world";
        let codec = NoopCompression;
        let compressed = codec.compress(data).expect("compress");
        assert_eq!(compressed, data);
        let decompressed = codec.decompress(&compressed).expect("decompress");
        assert_eq!(decompressed, data);
    }

    /// Verify NoopCompression name.
    #[test]
    fn noop_name() {
        assert_eq!(NoopCompression.name(), "noop");
    }

    /// Verify NoopCompression handles empty data.
    #[test]
    fn noop_empty_data() {
        let codec = NoopCompression;
        let compressed = codec.compress(b"").unwrap();
        assert!(compressed.is_empty());
        let decompressed = codec.decompress(b"").unwrap();
        assert!(decompressed.is_empty());
    }
}
