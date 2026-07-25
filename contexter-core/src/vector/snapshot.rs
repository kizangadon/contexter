//! Binary snapshot persistence for the HNSW vector index.
//!
//! The snapshot format is:
//!
//! ```text
//! [header: 32 bytes]
//!   magic:             u32 LE  — 0x484E5357 ("HNSW" in ASCII)
//!   version:           u32 LE  — 1
//!   dimension:         u32 LE
//!   element_count:     u32 LE  — total embedding count (including removed)
//!   m:                 u64 LE  — HNSW M parameter
//!   ef_construction:   u64 LE  — HNSW ef_construction parameter
//! [removed set]
//!   removed_count:     u32 LE
//!   for each removed id:
//!     id_len:          u32 LE
//!     id_bytes:        [u8; id_len]
//! [embeddings]
//!   for each embedding:
//!     id_len:          u32 LE
//!     id_bytes:        [u8; id_len]
//!     vector:          [f32 LE; dimension]
//! ```

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::vector::error::VectorError;
use crate::vector::VectorIndexResult;

/// Magic number: "HNSW" in ASCII (`0x484E5357`).
pub const SNAPSHOT_MAGIC: u32 = 0x484E5357;

/// Current snapshot format version.
pub const SNAPSHOT_VERSION: u32 = 1;

/// Header written before graph adjacency data.
#[derive(Debug, Clone)]
pub struct SnapshotHeader {
    pub magic: u32,
    pub version: u32,
    pub dimension: u32,
    pub element_count: u32,
    pub m: u64,
    pub ef_construction: u64,
}

impl SnapshotHeader {
    /// Size of the serialised header in bytes (4+4+4+4+8+8 = 32).
    pub const SERIALIZED_SIZE: usize = 32;

    /// Validate magic and version.
    pub fn validate(&self) -> VectorIndexResult<()> {
        if self.magic != SNAPSHOT_MAGIC {
            return Err(VectorError::InvalidMagic {
                expected: SNAPSHOT_MAGIC,
                actual: self.magic,
            });
        }
        if self.version != SNAPSHOT_VERSION {
            return Err(VectorError::VersionMismatch(
                self.version,
                SNAPSHOT_VERSION,
            ));
        }
        Ok(())
    }

    /// Serialise header to writer.
    pub fn write<W: Write>(&self, w: &mut W) -> std::io::Result<()> {
        w.write_all(&self.magic.to_le_bytes())?;
        w.write_all(&self.version.to_le_bytes())?;
        w.write_all(&self.dimension.to_le_bytes())?;
        w.write_all(&self.element_count.to_le_bytes())?;
        w.write_all(&self.m.to_le_bytes())?;
        w.write_all(&self.ef_construction.to_le_bytes())?;
        Ok(())
    }

    /// Deserialise header from reader.
    pub fn read<R: Read>(r: &mut R) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        let magic = u32::from_le_bytes(buf);
        r.read_exact(&mut buf)?;
        let version = u32::from_le_bytes(buf);
        r.read_exact(&mut buf)?;
        let dimension = u32::from_le_bytes(buf);
        r.read_exact(&mut buf)?;
        let element_count = u32::from_le_bytes(buf);
        let mut lbuf = [0u8; 8];
        r.read_exact(&mut lbuf)?;
        let m = u64::from_le_bytes(lbuf);
        r.read_exact(&mut lbuf)?;
        let ef_construction = u64::from_le_bytes(lbuf);
        Ok(Self {
            magic,
            version,
            dimension,
            element_count,
            m,
            ef_construction,
        })
    }
}

// ---------------------------------------------------------------------------
// Helper: read a length-prefixed string from a reader.
// ---------------------------------------------------------------------------

pub(crate) fn read_string<R: Read>(r: &mut R) -> std::io::Result<String> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf) as usize;

    // SA-1: Max-length guard (1024 bytes) to prevent OOM on crafted snapshots.
    const MAX_STRING_LEN: usize = 1024;
    if len > MAX_STRING_LEN {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("string length {len} exceeds maximum {MAX_STRING_LEN}"),
        ));
    }

    let mut bytes = vec![0u8; len];
    if len > 0 {
        r.read_exact(&mut bytes)?;
    }

    // SA-4: Strict UTF-8 — reject malformed bytes instead of silently replacing them.
    String::from_utf8(bytes).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid UTF-8 in snapshot string: {e}"),
        )
    })
}

/// Write a length-prefixed string to a writer.
pub(crate) fn write_string<W: Write>(w: &mut W, s: &str) -> std::io::Result<()> {
    let len = s.len() as u32;
    w.write_all(&len.to_le_bytes())?;
    if !s.is_empty() {
        w.write_all(s.as_bytes())?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Top-level save / load convenience functions.
// ---------------------------------------------------------------------------

/// Save header, removed set, and embeddings to a binary file.
pub fn save_snapshot_data(
    path: &Path,
    dimension: usize,
    embeddings: &[(String, Vec<f32>)],
    removed: &std::collections::HashSet<String>,
) -> VectorIndexResult<()> {
    let header = SnapshotHeader {
        magic: SNAPSHOT_MAGIC,
        version: SNAPSHOT_VERSION,
        dimension: dimension as u32,
        element_count: embeddings.len() as u32,
        m: 16,
        ef_construction: 200,
    };

    let file = File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);

    header.write(&mut writer)?;

    // Write removed IDs
    let removed_count = removed.len() as u32;
    writer.write_all(&removed_count.to_le_bytes())?;
    for id in removed.iter() {
        write_string(&mut writer, id)?;
    }

    // Write embeddings
    for (id, vector) in embeddings {
        write_string(&mut writer, id)?;
        for val in vector {
            writer.write_all(&val.to_le_bytes())?;
        }
    }

    writer.flush()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Err(e) = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)) {
            eprintln!("Warning: could not set 0o600 on snapshot file: {e}");
        }
    }

    Ok(())
}

/// Load embeddings and removed set from a binary snapshot file.
///
/// Takes an already-opened `File` so that the caller can inspect metadata
/// on the handle (avoiding a TOCTOU window between path-based checks and
/// the actual open).
pub fn load_snapshot_data(
    file: File,
    expected_dimension: usize,
) -> VectorIndexResult<(usize, Vec<(String, Vec<f32>)>, std::collections::HashSet<String>)> {
    let mut reader = std::io::BufReader::new(file);

    let header = SnapshotHeader::read(&mut reader)?;
    header.validate()?;

    if header.dimension as usize != expected_dimension {
        return Err(VectorError::DimensionMismatch(
            header.dimension as usize,
            expected_dimension,
        ));
    }

    // Read removed IDs
    let mut removed_buf = [0u8; 4];
    reader.read_exact(&mut removed_buf)?;
    let removed_count = u32::from_le_bytes(removed_buf);

    let mut removed_set = std::collections::HashSet::new();
    for _ in 0..removed_count {
        let id = read_string(&mut reader)?;
        removed_set.insert(id);
    }

    // Read embeddings
    let count = header.element_count as usize;
    let dimension = header.dimension as usize;
    let mut embeddings = Vec::with_capacity(count);

    for _ in 0..count {
        let id = read_string(&mut reader)?;
        let mut vector = vec![0.0f32; dimension];
        for val in &mut vector {
            let mut fbuf = [0u8; 4];
            reader.read_exact(&mut fbuf)?;
            *val = f32::from_le_bytes(fbuf);
        }
        embeddings.push((id, vector));
    }

    Ok((count, embeddings, removed_set))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_header_roundtrip() {
        let header = SnapshotHeader {
            magic: SNAPSHOT_MAGIC,
            version: SNAPSHOT_VERSION,
            dimension: 128,
            element_count: 42,
            m: 16,
            ef_construction: 200,
        };

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();
        assert_eq!(buf.len(), SnapshotHeader::SERIALIZED_SIZE);

        let mut cursor = std::io::Cursor::new(&buf);
        let decoded = SnapshotHeader::read(&mut cursor).unwrap();
        assert_eq!(decoded.magic, SNAPSHOT_MAGIC);
        assert_eq!(decoded.version, SNAPSHOT_VERSION);
        assert_eq!(decoded.dimension, 128);
        assert_eq!(decoded.element_count, 42);
        assert_eq!(decoded.m, 16);
        assert_eq!(decoded.ef_construction, 200);
    }

    #[test]
    fn test_header_validate_bad_magic() {
        let header = SnapshotHeader {
            magic: 0xDEADBEEF,
            version: SNAPSHOT_VERSION,
            dimension: 128,
            element_count: 0,
            m: 16,
            ef_construction: 200,
        };
        let result = header.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorError::InvalidMagic { .. }));
    }

    #[test]
    fn test_header_validate_bad_version() {
        let header = SnapshotHeader {
            magic: SNAPSHOT_MAGIC,
            version: 999,
            dimension: 128,
            element_count: 0,
            m: 16,
            ef_construction: 200,
        };
        let result = header.validate();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VectorError::VersionMismatch(..)));
    }

    #[test]
    fn test_header_validate_ok() {
        let header = SnapshotHeader {
            magic: SNAPSHOT_MAGIC,
            version: SNAPSHOT_VERSION,
            dimension: 128,
            element_count: 0,
            m: 16,
            ef_construction: 200,
        };
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_save_load_snapshot_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("vectors.snap");

        let embeddings = vec![
            ("id1".to_string(), vec![1.0, 0.0, 0.0]),
            ("id2".to_string(), vec![0.0, 1.0, 0.0]),
            ("id3".to_string(), vec![0.0, 0.0, 1.0]),
        ];
        let mut removed = HashSet::new();
        removed.insert("id2".to_string());

        save_snapshot_data(&path, 3, &embeddings, &removed).unwrap();

        let file = File::open(&path).unwrap();
        let (count, loaded_embs, loaded_rem) =
            load_snapshot_data(file, 3).unwrap();

        assert_eq!(count, 3);
        assert_eq!(loaded_embs.len(), 3);
        assert!(loaded_rem.contains("id2"));
        assert_eq!(loaded_embs[0].0, "id1");
        assert_eq!(loaded_embs[0].1, vec![1.0, 0.0, 0.0]);
    }

    #[test]
    fn test_load_corrupt_snapshot_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.snap");

        // Write garbage data
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOT A SNAPSHOT FILE\x00\x00\x00").unwrap();
        drop(file);

        let file = File::open(&path).unwrap();
        let result = load_snapshot_data(file, 64);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_wrong_dimension_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wrong_dim.snap");

        let embeddings = vec![("id1".to_string(), vec![1.0, 0.0])];
        let removed = HashSet::new();
        save_snapshot_data(&path, 2, &embeddings, &removed).unwrap();

        let file = File::open(&path).unwrap();
        let result = load_snapshot_data(file, 128);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VectorError::DimensionMismatch(2, 128)
        ));
    }
}
