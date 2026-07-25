use crate::error::EngineError;

#[derive(Debug, thiserror::Error)]
pub enum VectorError {
    #[error("Vector dimension {0} does not match index dimension {1}")]
    DimensionMismatch(usize, usize),
    #[error("Snapshot has invalid magic number: expected {expected:#x}, got {actual:#x}")]
    InvalidMagic { expected: u32, actual: u32 },
    #[error("Snapshot version {0} is not supported (max: {1})")]
    VersionMismatch(u32, u32),
    #[error("Vector contains NaN or Inf")]
    InvalidVector,
    #[error("IO error: {0}")]
    Io(String),
    #[error("Bincode serialization error: {0}")]
    Bincode(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Snapshot file is empty: {0}")]
    EmptySnapshot(String),
}

impl From<bincode::Error> for VectorError {
    fn from(e: bincode::Error) -> Self {
        VectorError::Bincode(e.to_string())
    }
}

impl From<std::io::Error> for VectorError {
    fn from(e: std::io::Error) -> Self {
        VectorError::Io(e.to_string())
    }
}

impl From<VectorError> for EngineError {
    fn from(e: VectorError) -> Self {
        EngineError::Internal(format!("vector error: {e}"))
    }
}
