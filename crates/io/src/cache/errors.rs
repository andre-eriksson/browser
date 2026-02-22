use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Cache error: {0}")]
    RuntimeError(String),

    #[error("Cache entry not found for key: {0}")]
    NotFound(String),

    #[error("Failed to serialize/deserialize cache data: {0}")]
    SerializationError(#[from] postcard::Error),

    #[error("Failed to read cache file: {0}")]
    ReadError(String),

    #[error("Failed to write cache file: {0}")]
    WriteError(String),

    #[error("Cache index is corrupted")]
    CorruptedIndex,

    #[error("Cache header is corrupted")]
    CorruptedHeader,

    #[error("Cache header is invalid")]
    InvalidHeader,

    #[error("Cache block is corrupted")]
    CorruptedBlock,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Asset error: {0}")]
    AssetError(#[from] crate::errors::AssetError),
}
