//! Vector embedding store and ANN (approximate nearest-neighbour) index.
//!
//! L3 tier: HNSW-based vector index with binary snapshot persistence.

use std::path::Path;

pub mod distance;
pub mod error;
pub mod hnsw;
pub mod snapshot;

pub use hnsw::HnswVectorIndex;

/// Convenience alias for vector index operations.
pub type VectorIndexResult<T> = Result<T, error::VectorError>;

/// Trait for the vector ANN index.
///
/// Implementors provide approximate nearest-neighbour search over stored
/// embedding vectors. All operations are thread-safe (`Send + Sync`).
pub trait VectorIndex: Send + Sync {
    /// Insert a vector associated with `id`.
    ///
    /// Returns an error if the vector dimension does not match the index
    /// dimension, or if the vector contains NaN or Inf.
    fn insert(&self, id: &str, vector: &[f32]) -> VectorIndexResult<()>;

    /// Search for the `k` nearest neighbours of `query`.
    ///
    /// Returns a list of `(id, similarity)` pairs sorted by descending
    /// similarity (highest first).  Similarity is in [−1, 1] for cosine.
    fn search(&self, query: &[f32], k: usize) -> VectorIndexResult<Vec<(String, f32)>>;

    /// Remove the vector identified by `id`.
    ///
    /// This is a logical deletion — the vector data remains in the index
    /// but is filtered out of future search results.
    fn remove(&self, id: &str) -> VectorIndexResult<()>;

    /// Persist the current index state to a binary snapshot at `path`.
    fn save_snapshot(&self, path: &Path) -> VectorIndexResult<()>;

    /// Load index state from a binary snapshot at `path`.
    ///
    /// Returns the number of elements loaded.
    fn load_snapshot(&self, path: &Path) -> VectorIndexResult<usize>;

    /// Number of active (non-removed) vectors in the index.
    fn len(&self) -> usize;

    /// Whether the index contains zero active vectors.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Public re-export of snapshot types for downstream consumers that need
/// to inspect snapshot headers.
pub use snapshot::SnapshotHeader;

#[cfg(test)]
mod tests {
    /// Verify that the placeholder types from Phase 2 compile.
    #[test]
    fn vector_trait_compiles() {
        fn _assert_send_sync<T: Send + Sync>() {}
        _assert_send_sync::<crate::vector::HnswVectorIndex>();
    }
}
