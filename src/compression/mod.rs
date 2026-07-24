//! Compression and decompression utilities for the Contexter storage engine.
//!
//! When the `compression` feature is enabled, `ZstdCompression` (default level 3)
//! and `Lz4Compression` (standard block mode) are available. Without the feature,
//! a `NoopCompression` fallback is provided.

use crate::error::EngineError;

/// A compression strategy that can compress and decompress byte slices.
pub trait Compression: Send + Sync {
    /// Compress `data` into a smaller byte vector.
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError>;
    /// Decompress `data` back to its original form.
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError>;
    /// Human-readable name of this compression algorithm.
    fn name(&self) -> &'static str;
}

// ---------------------------------------------------------------------------
// Feature-gated implementations
// ---------------------------------------------------------------------------

#[cfg(feature = "compression")]
pub use featured::*;

#[cfg(feature = "compression")]
mod featured {
    use super::*;

    /// Zstandard compression using the `zstd` crate.
    pub struct ZstdCompression {
        pub(crate) level: i32,
    }

    impl ZstdCompression {
        /// Create a new compressor with the given compression level.
        ///
        /// Level 0 means default; level 3 is a good balance point.
        pub fn new(level: i32) -> Self {
            Self { level }
        }
    }

    impl Default for ZstdCompression {
        fn default() -> Self {
            Self { level: 3 }
        }
    }

    impl Compression for ZstdCompression {
        fn compress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
            zstd::encode_all(std::io::Cursor::new(data), self.level)
                .map_err(|e| EngineError::Compression(format!("zstd compress: {e}")))
        }

        fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
            // Refuse to decompress more than 128MB (compression bomb protection).
            // zstd::decode_all uses a streaming decoder, but we add an explicit
            // upper bound as a defence-in-depth measure.
            const MAX_DECOMPRESSED_SIZE: usize = 128 * 1024 * 1024;

            let decompressed = zstd::decode_all(std::io::Cursor::new(data))
                .map_err(|e| EngineError::Compression(format!("zstd decompress: {e}")))?;

            if decompressed.len() > MAX_DECOMPRESSED_SIZE {
                return Err(EngineError::Compression(format!(
                    "Decompressed data too large: {} bytes (max {})",
                    decompressed.len(),
                    MAX_DECOMPRESSED_SIZE
                )));
            }

            Ok(decompressed)
        }

        fn name(&self) -> &'static str {
            "zstd"
        }
    }

    /// LZ4 block-mode compression using the `lz4` crate.
    pub struct Lz4Compression;

    impl Compression for Lz4Compression {
        fn compress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
            lz4::block::compress(data, None, true)
                .map_err(|e| EngineError::Compression(format!("lz4 compress: {e}")))
        }

        fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
            // Refuse to decompress more than 64MB (compression bomb protection).
            const MAX_DECOMPRESSED_SIZE: usize = 64 * 1024 * 1024;

            let decompressed = lz4::block::decompress(data, None)
                .map_err(|e| EngineError::Compression(format!("lz4 decompress: {e}")))?;

            if decompressed.len() > MAX_DECOMPRESSED_SIZE {
                return Err(EngineError::Compression(format!(
                    "Decompressed data too large: {} bytes (max {})",
                    decompressed.len(),
                    MAX_DECOMPRESSED_SIZE
                )));
            }

            Ok(decompressed)
        }

        fn name(&self) -> &'static str {
            "lz4"
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn generate_data(size: usize) -> Vec<u8> {
            // Use repeatable but varied data
            (0..size).map(|i| (i % 256) as u8).collect()
        }

        fn round_trip<C: Compression>(compressor: &C, data: &[u8]) {
            let compressed = compressor.compress(data).expect("compress");
            let decompressed = compressor.decompress(&compressed).expect("decompress");
            assert_eq!(decompressed, data, "round-trip must preserve data");
        }

        #[test]
        fn zstd_round_trip_1kb() {
            let c = ZstdCompression::default();
            let data = generate_data(1024);
            round_trip(&c, &data);
        }

        #[test]
        fn zstd_round_trip_1mb() {
            let c = ZstdCompression::default();
            let data = generate_data(1024 * 1024);
            round_trip(&c, &data);
        }

        #[test]
        fn zstd_empty_data() {
            let c = ZstdCompression::default();
            round_trip(&c, &[]);
        }

        #[test]
        fn zstd_corrupted_data() {
            let c = ZstdCompression::default();
            let result = c.decompress(b"not zstd data");
            assert!(result.is_err(), "corrupted data should return error");
        }

        #[test]
        fn lz4_round_trip_1kb() {
            let c = Lz4Compression;
            let data = generate_data(1024);
            round_trip(&c, &data);
        }

        #[test]
        fn lz4_round_trip_1mb() {
            let c = Lz4Compression;
            let data = generate_data(1024 * 1024);
            round_trip(&c, &data);
        }

        #[test]
        fn lz4_empty_data() {
            let c = Lz4Compression;
            round_trip(&c, &[]);
        }

        #[test]
        fn lz4_corrupted_data() {
            let c = Lz4Compression;
            let result = c.decompress(b"not lz4 data");
            assert!(result.is_err(), "corrupted data should return error");
        }

        #[test]
        fn zstd_compression_actually_reduces_size() {
            let c = ZstdCompression::default();
            let data = generate_data(64 * 1024); // 64KB of repeated pattern
            let compressed = c.compress(&data).expect("compress");
            assert!(
                compressed.len() < data.len(),
                "compressed size {} should be less than {}",
                compressed.len(),
                data.len()
            );
        }

        #[test]
        fn zstd_rejects_oversized_decompression() {
            // Generate a compressible 129MB payload (just over the 128MB limit)
            // to verify the size check catches it.
            let c = ZstdCompression::default();
            let data = vec![0u8; 129 * 1024 * 1024]; // 129MB of zeros
            let compressed = c.compress(&data).expect("compress");
            let result = c.decompress(&compressed);
            assert!(result.is_err(), "decompression of 129MB should be rejected");
            let err = result.unwrap_err().to_string();
            assert!(err.contains("too large"), "error should mention size limit");
        }

        #[test]
        fn lz4_rejects_oversized_decompression() {
            let c = Lz4Compression;
            let data = vec![0u8; 65 * 1024 * 1024]; // 65MB
            let compressed = c.compress(&data).expect("compress");
            let result = c.decompress(&compressed);
            assert!(result.is_err(), "decompression of 65MB should be rejected");
            let err = result.unwrap_err().to_string();
            assert!(err.contains("too large"), "error should mention size limit");
        }

        #[test]
        fn lz4_empty_input_decompress_returns_empty() {
            // Edge case: empty input after compression of empty data should
            // decompress to an empty vec, not error.
            let c = Lz4Compression;
            let compressed = c.compress(b"").expect("compress empty");
            let decompressed = c.decompress(&compressed).expect("decompress empty");
            assert!(
                decompressed.is_empty(),
                "empty round-trip should yield empty vec"
            );
        }

        #[test]
        fn lz4_name_is_correct() {
            assert_eq!(Lz4Compression.name(), "lz4");
        }

        #[test]
        fn zstd_name_is_correct() {
            let c = ZstdCompression::default();
            assert_eq!(c.name(), "zstd");
        }
    }
}

// ---------------------------------------------------------------------------
// Noop fallback (available without the `compression` feature)
// ---------------------------------------------------------------------------

/// A no-operation compressor that returns data unchanged.
///
/// Used when the `compression` feature is not enabled.
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
mod noop_tests {
    use super::*;

    #[test]
    fn noop_returns_data_unchanged() {
        let c = NoopCompression;
        let data = b"hello world";
        let compressed = c.compress(data).unwrap();
        assert_eq!(compressed, data);
        let decompressed = c.decompress(data).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn noop_name_is_correct() {
        assert_eq!(NoopCompression.name(), "noop");
    }

    #[test]
    fn noop_empty_data() {
        let c = NoopCompression;
        let compressed = c.compress(b"").unwrap();
        assert!(compressed.is_empty());
        let decompressed = c.decompress(b"").unwrap();
        assert!(decompressed.is_empty());
    }
}
