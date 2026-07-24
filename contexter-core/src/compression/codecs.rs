//! Compression codec implementations (Zstd and LZ4).

// Re-export feature-gated implementations.
#[cfg(feature = "compression")]
pub use featured::*;

#[cfg(feature = "compression")]
mod featured {
    use super::super::Compression;
    use crate::error::EngineError;

    /// Zstandard compression using the `zstd` crate.
    pub struct ZstdCompression {
        pub(crate) level: i32,
    }

    impl ZstdCompression {
        /// Create a new compressor with the given compression level.
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

    // -----------------------------------------------------------------------
    // Tests
    // -----------------------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        fn generate_data(size: usize) -> Vec<u8> {
            (0..size).map(|i| (i % 256) as u8).collect()
        }

        fn round_trip<C: Compression>(compressor: &C, data: &[u8]) {
            let compressed = compressor.compress(data).expect("compress");
            let decompressed = compressor.decompress(&compressed).expect("decompress");
            assert_eq!(decompressed, data, "round-trip must preserve data");
        }

        // --------------------------------------------------------------
        // Zstd tests
        // --------------------------------------------------------------

        #[test]
        fn zstd_compress_decompress_roundtrip() {
            let c = ZstdCompression::default();
            // Use data large enough for zstd to achieve compression.
            let data = "hello world, this is test data for zstd compression! ".repeat(20);
            let data_bytes = data.as_bytes();
            let compressed = c.compress(data_bytes).expect("compress");
            assert!(compressed.len() < data_bytes.len(), "compressed should be smaller");
            let decompressed = c.decompress(&compressed).expect("decompress");
            assert_eq!(decompressed, data_bytes);
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
        fn zstd_empty_input_roundtrip() {
            let c = ZstdCompression::default();
            let compressed = c.compress(b"").expect("compress empty");
            let decompressed = c.decompress(&compressed).expect("decompress empty");
            assert!(decompressed.is_empty());
        }

        #[test]
        fn zstd_corrupted_data() {
            let c = ZstdCompression::default();
            let result = c.decompress(b"not zstd data");
            assert!(result.is_err(), "corrupted data should return error");
        }

        #[test]
        fn zstd_rejects_oversized_decompression() {
            let c = ZstdCompression::default();
            let data = vec![0u8; 129 * 1024 * 1024]; // 129MB
            let compressed = c.compress(&data).expect("compress");
            let result = c.decompress(&compressed);
            assert!(result.is_err(), "decompression of 129MB should be rejected");
            let err = result.unwrap_err().to_string();
            assert!(err.contains("too large"), "error should mention size limit");
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
        fn zstd_name_is_correct() {
            let c = ZstdCompression::default();
            assert_eq!(c.name(), "zstd");
        }

        #[test]
        fn zstd_round_trip_empty_via_helper() {
            let c = ZstdCompression::default();
            round_trip(&c, &[]);
        }

        #[test]
        fn zstd_round_trip_16kb() {
            let c = ZstdCompression::default();
            let data = generate_data(16 * 1024);
            round_trip(&c, &data);
        }

        #[test]
        fn zstd_binary_data_roundtrip() {
            let c = ZstdCompression::default();
            let data = vec![0x00, 0xFF, 0xAA, 0x55, 0x01, 0xFE, 0x80, 0x7F];
            round_trip(&c, &data);
        }

        #[test]
        fn zstd_single_byte_roundtrip() {
            let c = ZstdCompression::default();
            for byte in [0x00, 0x01, 0x80, 0xFF] {
                let data = vec![byte];
                round_trip(&c, &data);
            }
        }

        // --------------------------------------------------------------
        // LZ4 tests
        // --------------------------------------------------------------

        #[test]
        fn lz4_compress_decompress_roundtrip() {
            let c = Lz4Compression;
            let data = b"hello world, this is test data for lz4 compression!";
            let compressed = c.compress(data).expect("compress");
            let decompressed = c.decompress(&compressed).expect("decompress");
            assert_eq!(decompressed, data);
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
        fn lz4_empty_input_decompress_returns_empty() {
            let c = Lz4Compression;
            let compressed = c.compress(b"").expect("compress empty");
            let decompressed = c.decompress(&compressed).expect("decompress empty");
            assert!(decompressed.is_empty());
        }

        #[test]
        fn lz4_corrupted_data() {
            let c = Lz4Compression;
            let result = c.decompress(b"not lz4 data");
            assert!(result.is_err(), "corrupted data should return error");
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
        fn lz4_name_is_correct() {
            assert_eq!(Lz4Compression.name(), "lz4");
        }

        #[test]
        fn lz4_round_trip_empty_via_helper() {
            round_trip(&Lz4Compression, &[]);
        }

        #[test]
        fn lz4_round_trip_16kb() {
            let c = Lz4Compression;
            let data = generate_data(16 * 1024);
            round_trip(&c, &data);
        }

        #[test]
        fn lz4_binary_data_roundtrip() {
            let c = Lz4Compression;
            let data = vec![0x00, 0xFF, 0xAA, 0x55, 0x01, 0xFE, 0x80, 0x7F];
            round_trip(&c, &data);
        }

        #[test]
        fn lz4_single_byte_roundtrip() {
            let c = Lz4Compression;
            for byte in [0x00, 0x01, 0x80, 0xFF] {
                let data = vec![byte];
                round_trip(&c, &data);
            }
        }

        #[test]
        fn lz4_compression_actually_reduces_size() {
            let c = Lz4Compression;
            let data = generate_data(64 * 1024); // 64KB of repeated pattern
            let compressed = c.compress(&data).expect("compress");
            assert!(
                compressed.len() < data.len(),
                "compressed size {} should be less than {}",
                compressed.len(),
                data.len()
            );
        }
    }
}
