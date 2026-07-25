//! HNSW (Hierarchical Navigable Small World) vector index.
//!
//! Wraps the [`instant_distance`] HNSW implementation, providing thread-safe
//! approximate nearest-neighbour (ANN) search with cosine similarity.
//!
//! The index stores embeddings in an internal `Vec` and rebuilds the HNSW
//! graph on each insert (batch construction).  Removals are logical (mark
//! and filter) — the embedding data remains in the index but is excluded
//! from search results.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use instant_distance::{Builder, Hnsw, Search};
use serde::{Deserialize, Serialize};

use crate::vector::distance::cosine_similarity;
use crate::vector::error::VectorError;
use crate::vector::snapshot;
use crate::vector::{VectorIndex, VectorIndexResult};

/// Contexter's embedding point type for `instant_distance`.
///
/// Each embedding carries its domain-level `id` string and a dense `vector`
/// of `f32` values.  The `distance` method returns `1 - cosine_similarity`,
/// converting cosine similarity (higher = closer) into a distance metric
/// (lower = closer) suitable for `instant_distance`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Embedding {
    pub(crate) id: String,
    pub(crate) vector: Vec<f32>,
}

impl instant_distance::Point for Embedding {
    fn distance(&self, other: &Self) -> f32 {
        1.0 - cosine_similarity(&self.vector, &other.vector)
    }
}

/// HNSW vector index backed by `instant_distance`.
///
/// All mutation operations (insert, remove) are thread-safe via `RwLock`.
/// The HNSW graph is rebuilt from scratch on each insertion — this is
/// acceptable for the expected scale (tens of thousands of embeddings)
/// and keeps the design simple.
pub struct HnswVectorIndex {
    /// Raw embedding storage (source of truth for rebuilds and snapshots).
    embeddings: RwLock<Vec<Embedding>>,
    /// The HNSW graph, rebuilt after each mutation.
    hnsw: RwLock<Hnsw<Embedding>>,
    /// Expected vector dimension for all embeddings.
    dimension: usize,
    /// Maximum number of connections per element (M parameter from the paper).
    /// Note: currently stored for forward-compatibility; the underlying
    /// `instant_distance` library hardcodes M=32 and does not expose it on
    /// `Builder`. This field is reserved for when the library adds support.
    m: usize,
    /// Number of candidate nearest-neighbours used during construction
    /// (efConstruction parameter from the paper).
    ef_construction: usize,
    /// Number of candidate nearest-neighbours used during search
    /// (ef parameter from the paper).
    ef_search: usize,
    /// Set of logically-deleted IDs (filtered out during search).
    removed: RwLock<HashSet<String>>,
    /// Mutation counter (triggers auto-snapshot when threshold is hit).
    mutation_count: RwLock<u64>,
    /// Number of mutations before an automatic snapshot is taken.
    auto_snapshot_threshold: u64,
    /// Optional path for automatic snapshots.
    snapshot_path: RwLock<Option<PathBuf>>,
}

impl HnswVectorIndex {
    /// Create a new empty index with the given vector dimension and HNSW parameters.
    ///
    /// `m` is the maximum number of connections per element (reserved for future
    /// library support; `instant_distance` currently hardcodes M=32).
    pub fn new(dimension: usize, m: usize, ef_construction: usize, ef_search: usize) -> Self {
        let builder = Builder::default()
            .ef_construction(ef_construction)
            .ef_search(ef_search);
        let (empty_hnsw, _) = builder.build_hnsw(Vec::<Embedding>::new());
        Self {
            embeddings: RwLock::new(Vec::new()),
            hnsw: RwLock::new(empty_hnsw),
            dimension,
            m,
            ef_construction,
            ef_search,
            removed: RwLock::new(HashSet::new()),
            mutation_count: RwLock::new(0),
            auto_snapshot_threshold: 1000,
            snapshot_path: RwLock::new(None),
        }
    }

    /// Set auto-snapshot path and threshold.
    ///
    /// When the mutation counter reaches `threshold`, the index automatically
    /// saves a snapshot to `path`.
    #[allow(unused)]
    pub fn with_auto_snapshot(self, path: PathBuf, threshold: u64) -> Self {
        *self.snapshot_path.write().unwrap_or_else(|e| e.into_inner()) = Some(path);
        Self {
            auto_snapshot_threshold: threshold,
            ..self
        }
    }

    /// Return the dimension of vectors in this index.
    #[allow(unused)]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Check if vector contains NaN or Inf.
    fn validate_vector(vector: &[f32]) -> VectorIndexResult<()> {
        if vector.iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(VectorError::InvalidVector);
        }
        Ok(())
    }

    /// Rebuild the HNSW graph from the current embedding list.
    fn rebuild(&self) {
        let embeddings = self.embeddings.read().unwrap_or_else(|e| e.into_inner());
        let builder = Builder::default()
            .ef_construction(self.ef_construction)
            .ef_search(self.ef_search);
        if embeddings.is_empty() {
            let (empty_hnsw, _) = builder.build_hnsw(Vec::<Embedding>::new());
            *self.hnsw.write().unwrap_or_else(|e| e.into_inner()) = empty_hnsw;
            return;
        }
        let points = embeddings.clone();
        let (new_hnsw, _pids) = builder.build_hnsw(points);
        *self.hnsw.write().unwrap_or_else(|e| e.into_inner()) = new_hnsw;
    }

    /// Increment mutation counter and auto-snapshot if threshold hit.
    fn check_auto_snapshot(&self) -> VectorIndexResult<()> {
        let mut mc = self.mutation_count.write().unwrap_or_else(|e| e.into_inner());
        *mc += 1;
        if *mc >= self.auto_snapshot_threshold {
            *mc = 0;
            let path_lock = self.snapshot_path.read().unwrap_or_else(|e| e.into_inner());
            if let Some(path) = path_lock.as_ref() {
                self.save_snapshot(path)?;
            }
        }
        Ok(())
    }

    /// Insert multiple embeddings in a batch, building the HNSW graph once.
    ///
    /// All embeddings are validated before any mutation occurs. If any
    /// embedding has a mismatched dimension or contains NaN/Inf, the
    /// entire batch is rejected and the index is not modified.
    ///
    /// Idempotency: if an ID already exists, it is updated (replaced).
    /// If an ID was previously removed, it is un-deleted.
    pub fn insert_batch(&self, new_embeddings: &[(String, Vec<f32>)]) -> VectorIndexResult<()> {
        if new_embeddings.is_empty() {
            return Ok(());
        }

        // Validate all embeddings before mutating anything.
        for (_id, vector) in new_embeddings {
            Self::validate_vector(vector)?;
            if vector.len() != self.dimension {
                return Err(VectorError::DimensionMismatch(
                    vector.len(),
                    self.dimension,
                ));
            }
        }

        // Extend embedding storage (update existing or insert new).
        {
            let mut embeddings = self.embeddings.write().unwrap_or_else(|e| e.into_inner());
            for (id, vector) in new_embeddings {
                let point = Embedding {
                    id: id.clone(),
                    vector: vector.clone(),
                };
                if let Some(pos) = embeddings.iter().position(|e| e.id == *id) {
                    embeddings[pos] = point;
                } else {
                    embeddings.push(point);
                }
            }
        }

        // Un-delete any previously removed IDs in the batch.
        {
            let mut removed = self.removed.write().unwrap_or_else(|e| e.into_inner());
            for (embedded_id, _) in new_embeddings {
                removed.remove(embedded_id);
            }
        }

        // Build the HNSW graph once from all embeddings.
        self.rebuild();

        // Update mutation counter (batch counts as a single mutation).
        self.check_auto_snapshot()?;

        Ok(())
    }

    // ------------------------------------------------------------------
    // Bincode-based save / load (fast path — preserves HNSW graph)
    // ------------------------------------------------------------------

    /// Persist the full index state (embeddings + HNSW graph + metadata)
    /// to `path` via bincode.
    ///
    /// Writes atomically: data goes to a `.tmp` sibling first, then is
    /// renamed to the final path.
    pub fn save(&self, path: &Path) -> VectorIndexResult<()> {
        // Collect serialised state while read locks are held.
        let bytes = {
            let embeddings = self.embeddings.read().unwrap_or_else(|e| e.into_inner());
            let hnsw = self.hnsw.read().unwrap_or_else(|e| e.into_inner());
            let removed = self.removed.read().unwrap_or_else(|e| e.into_inner());
            let mutation_count = *self
                .mutation_count
                .read()
                .unwrap_or_else(|e| e.into_inner());

            let data = SaveData {
                version: 1,
                dimension: self.dimension as u32,
                m: self.m as u64,
                ef_construction: self.ef_construction as u64,
                ef_search: self.ef_search as u64,
                embeddings: &embeddings,
                hnsw: &hnsw,
                removed: &removed,
                mutation_count,
                auto_snapshot_threshold: self.auto_snapshot_threshold,
            };

            bincode::serialize(&data)?
        };

        // Atomic write: temp file + rename.
        let tmp_path = path.with_extension("tmp");
        std::fs::write(&tmp_path, &bytes)?;
        std::fs::rename(&tmp_path, path)?;

        Ok(())
    }

    /// Load a previously saved bincode snapshot, or create a new empty index
    /// if the file does not exist.
    pub fn load_or_new(
        path: &Path,
        dimension: usize,
        m: usize,
        ef_construction: usize,
        ef_search: usize,
    ) -> VectorIndexResult<Self> {
        if path.exists() {
            Self::load_from(path, dimension, m, ef_construction, ef_search)
        } else {
            Ok(Self::new(dimension, m, ef_construction, ef_search))
        }
    }

    /// Load index state from a bincode snapshot at `path`.
    pub fn load_from(
        path: &Path,
        dimension: usize,
        _m: usize,
        _ef_construction: usize,
        _ef_search: usize,
    ) -> VectorIndexResult<Self> {
        let raw = std::fs::read(path)?;
        let data: LoadData = bincode::deserialize(&raw)?;

        // Validate metadata matches expected parameters.
        if (data.dimension as usize) != dimension {
            return Err(VectorError::DimensionMismatch(
                data.dimension as usize,
                dimension,
            ));
        }

        Ok(Self {
            embeddings: RwLock::new(data.embeddings),
            hnsw: RwLock::new(data.hnsw),
            dimension: data.dimension as usize,
            m: data.m as usize,
            ef_construction: data.ef_construction as usize,
            ef_search: data.ef_search as usize,
            removed: RwLock::new(data.removed),
            mutation_count: RwLock::new(data.mutation_count),
            auto_snapshot_threshold: data.auto_snapshot_threshold,
            snapshot_path: RwLock::new(None),
        })
    }

    /// Spawn a background thread that saves a bincode snapshot every
    /// `interval_secs` seconds.  Stops when `cancel` is set to `true`.
    pub fn periodic_snapshot(
        self: Arc<Self>,
        interval_secs: u64,
        snapshot_path: PathBuf,
        cancel: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            while !cancel.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(interval_secs));
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                if let Err(e) = self.save(&snapshot_path) {
                    eprintln!("[contexter] periodic snapshot error: {e}");
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Bincode serialization helpers
// ---------------------------------------------------------------------------

/// Data written to disk by [`HnswVectorIndex::save`].  Uses references to
/// avoid a `Clone` bound on [`Hnsw`].
#[derive(Serialize)]
struct SaveData<'a> {
    version: u32,
    dimension: u32,
    m: u64,
    ef_construction: u64,
    ef_search: u64,
    embeddings: &'a Vec<Embedding>,
    hnsw: &'a Hnsw<Embedding>,
    removed: &'a HashSet<String>,
    mutation_count: u64,
    auto_snapshot_threshold: u64,
}

/// Data read from disk by [`HnswVectorIndex::load_from`].  Owns every field.
#[derive(Deserialize)]
struct LoadData {
    version: u32,
    dimension: u32,
    m: u64,
    ef_construction: u64,
    ef_search: u64,
    embeddings: Vec<Embedding>,
    hnsw: Hnsw<Embedding>,
    removed: HashSet<String>,
    mutation_count: u64,
    auto_snapshot_threshold: u64,
}

impl VectorIndex for HnswVectorIndex {
    fn insert(&self, id: &str, vector: &[f32]) -> VectorIndexResult<()> {
        Self::validate_vector(vector)?;
        if vector.len() != self.dimension {
            return Err(VectorError::DimensionMismatch(
                vector.len(),
                self.dimension,
            ));
        }

        let point = Embedding {
            id: id.to_string(),
            vector: vector.to_vec(),
        };

        // Append to embedding storage, then rebuild the HNSW graph.
        {
            let mut embeddings = self.embeddings.write().unwrap_or_else(|e| e.into_inner());
            // If this ID already exists, replace it (update semantics).
            if let Some(pos) = embeddings.iter().position(|e| e.id == id) {
                embeddings[pos] = point;
            } else {
                embeddings.push(point);
            }
        }

        // Remove from deleted set (undelete on re-insert).
        self.removed.write().unwrap_or_else(|e| e.into_inner()).remove(id);

        self.rebuild();
        self.check_auto_snapshot()?;
        Ok(())
    }

    fn search(&self, query: &[f32], k: usize) -> VectorIndexResult<Vec<(String, f32)>> {
        Self::validate_vector(query)?;
        if query.len() != self.dimension {
            return Err(VectorError::DimensionMismatch(query.len(), self.dimension));
        }
        if k == 0 || self.is_empty() {
            return Ok(Vec::new());
        }

        let query_point = Embedding {
            id: String::new(),
            vector: query.to_vec(),
        };

        let hnsw = self.hnsw.read().unwrap_or_else(|e| e.into_inner());
        let removed = self.removed.read().unwrap_or_else(|e| e.into_inner());
        let mut search = Search::default();

        let actual_k = k.min(self.len());

        let results: Vec<(String, f32)> = hnsw
            .search(&query_point, &mut search)
            .filter(|item| !removed.contains(&item.point.id))
            .take(actual_k)
            .map(|item| {
                // Convert distance back to cosine similarity.
                let similarity = 1.0 - item.distance;
                (item.point.id.clone(), similarity)
            })
            .collect();

        Ok(results)
    }

    fn remove(&self, id: &str) -> VectorIndexResult<()> {
        // Logical deletion: add to the removed set.
        // Physical compaction can be done via a separate rebuild operation.
        self.removed.write().unwrap_or_else(|e| e.into_inner()).insert(id.to_string());
        self.check_auto_snapshot()?;
        Ok(())
    }

    fn save_snapshot(&self, path: &Path) -> VectorIndexResult<()> {
        let embeddings = self.embeddings.read().unwrap_or_else(|e| e.into_inner());
        let removed = self.removed.read().unwrap_or_else(|e| e.into_inner());

        let data: Vec<(String, Vec<f32>)> = embeddings
            .iter()
            .map(|e| (e.id.clone(), e.vector.clone()))
            .collect();

        snapshot::save_snapshot_data(path, self.dimension, &data, &removed)
    }

    fn load_snapshot(&self, path: &Path) -> VectorIndexResult<usize> {
        // Open the file first, then check metadata on the opened handle
        // to eliminate the TOCTOU window between path-based checks and open.
        let file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        if metadata.is_dir() {
            return Err(VectorError::Io(format!(
                "is a directory: {}",
                path.to_string_lossy()
            )));
        }
        if metadata.len() == 0 {
            return Err(VectorError::EmptySnapshot(
                path.to_string_lossy().into_owned(),
            ));
        }

        let (count, data, loaded_removed) =
            snapshot::load_snapshot_data(file, self.dimension)?;

        let embeddings: Vec<Embedding> = data
            .into_iter()
            .map(|(id, vector)| Embedding { id, vector })
            .collect();

        *self.embeddings.write().unwrap_or_else(|e| e.into_inner()) = embeddings;
        *self.removed.write().unwrap_or_else(|e| e.into_inner()) = loaded_removed;

        self.rebuild();

        Ok(count)
    }

    fn len(&self) -> usize {
        let embeddings = self.embeddings.read().unwrap_or_else(|e| e.into_inner());
        let removed = self.removed.read().unwrap_or_else(|e| e.into_inner());
        embeddings.len().saturating_sub(removed.len())
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::fmt::Debug for HnswVectorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HnswVectorIndex")
            .field("dimension", &self.dimension)
            .field("m", &self.m)
            .field("ef_construction", &self.ef_construction)
            .field("ef_search", &self.ef_search)
            .field("active_count", &self.len())
            .field("total_count", &self.embeddings.read().unwrap_or_else(|e| e.into_inner()).len())
            .field("removed_count", &self.removed.read().unwrap_or_else(|e| e.into_inner()).len())
            .field("auto_snapshot_threshold", &self.auto_snapshot_threshold)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Helper: create a small index with fixed synthetic embeddings.
    fn make_test_index() -> HnswVectorIndex {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let vectors: Vec<(&str, [f32; 4])> = vec![
            ("a", [1.0, 0.0, 0.0, 0.0]),
            ("b", [0.0, 1.0, 0.0, 0.0]),
            ("c", [0.0, 0.0, 1.0, 0.0]),
            ("d", [0.0, 0.0, 0.0, 1.0]),
            ("e", [1.0, 1.0, 0.0, 0.0]),
            ("f", [0.0, 1.0, 1.0, 0.0]),
            ("g", [1.0, 0.0, 1.0, 0.0]),
            ("h", [0.5, 0.5, 0.5, 0.5]),
            ("i", [1.0, 0.5, 0.0, 0.0]),
            ("j", [0.0, 0.0, 0.5, 1.0]),
        ];
        for (name, vec) in vectors {
            idx.insert(name, &vec).unwrap();
        }
        idx
    }

    // ------------------------------------------------------------------
    // Basic operations
    // ------------------------------------------------------------------

    #[test]
    fn test_insert_and_search() {
        let idx = make_test_index();
        // Query near 'a' — should return 'a' as the top result.
        let query = [0.9, 0.1, 0.0, 0.0];
        let results = idx.search(&query, 5).unwrap();
        assert!(!results.is_empty(), "search should return results");
        assert_eq!(results[0].0, "a", "nearest should be 'a'");
        assert!(
            (results[0].1 - 1.0).abs() < 0.1,
            "similarity should be ~1.0 for near-identical vector"
        );
    }

    #[test]
    fn test_empty_search() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 5).unwrap();
        assert!(results.is_empty(), "empty index should return empty results");
    }

    #[test]
    fn test_remove_and_search() {
        let idx = make_test_index();
        // 'a' should be the top result for a query near 'a'.
        let query = [1.0, 0.0, 0.0, 0.0];
        let results_before = idx.search(&query, 5).unwrap();
        assert!(!results_before.is_empty());
        assert_eq!(results_before[0].0, "a");

        // Remove 'a' and verify it no longer appears.
        idx.remove("a").unwrap();
        let results_after = idx.search(&query, 5).unwrap();
        assert!(
            !results_after.iter().any(|(id, _)| id == "a"),
            "removed 'a' should not appear in search results"
        );
    }

    #[test]
    fn test_insert_update() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        idx.insert("x", &[1.0, 0.0, 0.0, 0.0]).unwrap();
        // Update 'x' to a different location.
        idx.insert("x", &[0.0, 1.0, 0.0, 0.0]).unwrap();

        // Search near the new location.
        let results = idx.search(&[0.0, 1.0, 0.0, 0.0], 3).unwrap();
        assert_eq!(results[0].0, "x");
        assert!((results[0].1 - 1.0).abs() < 0.01);
    }

    // ------------------------------------------------------------------
    // Error handling
    // ------------------------------------------------------------------

    #[test]
    fn test_dimension_mismatch() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let result = idx.insert("bad", &[1.0, 0.0, 0.0]);
        assert!(
            result.is_err(),
            "insert with wrong dimension should return error"
        );
        assert!(matches!(
            result.unwrap_err(),
            VectorError::DimensionMismatch(3, 4)
        ));
    }

    #[test]
    fn test_search_dimension_mismatch() {
        let idx = make_test_index();
        let result = idx.search(&[1.0, 0.0, 0.0], 5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::DimensionMismatch(3, 4)
        ));
    }

    #[test]
    fn test_nan_vector_rejected() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let result = idx.insert("nan", &[f32::NAN, 0.0, 0.0, 0.0]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorError::InvalidVector));
    }

    #[test]
    fn test_inf_vector_rejected() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let result = idx.insert("inf", &[f32::INFINITY, 0.0, 0.0, 0.0]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorError::InvalidVector));
    }

    // ------------------------------------------------------------------
    // Edge cases
    // ------------------------------------------------------------------

    #[test]
    fn test_k_larger_than_index() {
        let idx = make_test_index(); // 10 items
        let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 100).unwrap();
        assert_eq!(results.len(), 10, "should return all 10 active items");
    }

    #[test]
    fn test_k_zero() {
        let idx = make_test_index();
        let results = idx.search(&[1.0, 0.0, 0.0, 0.0], 0).unwrap();
        assert!(results.is_empty(), "k=0 should return empty");
    }

    #[test]
    fn test_remove_nonexistent() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        // Removing a non-existent ID should be a no-op (not error).
        idx.remove("nonexistent").unwrap();
        assert!(idx.is_empty());
    }

    #[test]
    fn test_len_and_is_empty() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);

        idx.insert("a", &[1.0, 0.0, 0.0, 0.0]).unwrap();
        assert!(!idx.is_empty());
        assert_eq!(idx.len(), 1);

        idx.remove("a").unwrap();
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
    }

    #[test]
    fn test_remove_does_not_affect_other_results() {
        let idx = make_test_index();
        let results_before = idx.search(&[1.0, 0.0, 0.0, 0.0], 10).unwrap();

        idx.remove("a").unwrap();
        let results_after = idx.search(&[1.0, 0.0, 0.0, 0.0], 10).unwrap();

        // All results after removal should still be from the original set
        // (minus 'a').
        let ids_before: HashSet<String> = results_before.iter().map(|(id, _)| id.clone()).collect();
        let ids_after: HashSet<String> = results_after.iter().map(|(id, _)| id.clone()).collect();

        assert!(!ids_after.contains("a"));
        assert!(ids_after.is_subset(&ids_before));
    }

    // ------------------------------------------------------------------
    // Snapshot roundtrip
    // ------------------------------------------------------------------

    #[test]
    fn test_snapshot_roundtrip() {
        let idx = make_test_index();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("roundtrip.snap");

        // Remove one element so we also test removed-set persistence.
        idx.remove("c").unwrap();

        idx.save_snapshot(&path).unwrap();

        // Load into a fresh index.
        let loaded = HnswVectorIndex::new(4, 16, 200, 50);
        let count = loaded.load_snapshot(&path).unwrap();
        assert_eq!(count, 10, "should load all 10 embeddings");

        // Verify that 'c' is removed in the loaded index.
        let query = [0.0, 0.0, 1.0, 0.0];
        let results = loaded.search(&query, 10).unwrap();
        assert!(
            !results.iter().any(|(id, _)| id == "c"),
            "removed 'c' should not appear after load"
        );

        // Verify that the remaining elements produce the same top match
        // as the original index.
        let query_a = [0.9, 0.1, 0.0, 0.0];
        let original_results = idx.search(&query_a, 3).unwrap();
        let loaded_results = loaded.search(&query_a, 3).unwrap();
        assert_eq!(original_results[0].0, loaded_results[0].0);
        assert!(
            (original_results[0].1 - loaded_results[0].1).abs() < 0.01
        );
    }

    #[test]
    fn test_empty_index_snapshot_roundtrip() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.snap");

        idx.save_snapshot(&path).unwrap();

        let loaded = HnswVectorIndex::new(4, 16, 200, 50);
        let count = loaded.load_snapshot(&path).unwrap();
        assert_eq!(count, 0);
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_empty_snapshot_rejected() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.snap");

        // Create an empty file (0 bytes).
        std::fs::write(&path, b"").unwrap();

        let result = idx.load_snapshot(&path);
        assert!(result.is_err());
        assert!(
            matches!(&result.unwrap_err(), VectorError::EmptySnapshot(p) if p.ends_with("empty.snap")),
            "expected EmptySnapshot error for zero-length file"
        );
    }

    #[test]
    fn test_directory_snapshot_rejected() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("subdir");

        // Create a directory instead of a file.
        std::fs::create_dir(&path).unwrap();

        let result = idx.load_snapshot(&path);
        assert!(result.is_err());
        assert!(
            matches!(&result.unwrap_err(), VectorError::Io(msg) if msg.contains("is a directory")),
            "expected Io error for directory"
        );
    }

    #[test]
    fn test_empty_file_metadata_check() {
        // Verify that metadata check catches empty file before File::open.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nonexistent.snap");

        // Non-existent file should still produce an error (from File::open or metadata).
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let result = idx.load_snapshot(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_corrupt_snapshot_rejected() {
        let idx = HnswVectorIndex::new(4, 16, 200, 50);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.snap");

        // Write garbage (not a valid snapshot).
        std::fs::write(&path, b"this is not a vector snapshot file").unwrap();

        let result = idx.load_snapshot(&path);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // Debug formatting
    // ------------------------------------------------------------------

    #[test]
    fn test_debug_format() {
        let idx = make_test_index();
        let debug_str = format!("{:?}", idx);
        assert!(debug_str.contains("dimension"));
        assert!(debug_str.contains("active_count"));
    }
}
