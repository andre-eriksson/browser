//! Errors related to cache operations.

use thiserror::Error;

/// Represents the result of a cache read operation, indicating whether it was a hit or a miss.
#[derive(Debug, Clone)]
pub enum CacheRead<T: Clone> {
    Hit(T),
    Miss,
}

impl<T: Clone> CacheRead<T> {
    pub fn is_hit(&self) -> bool {
        matches!(self, CacheRead::Hit(_))
    }

    pub fn is_miss(&self) -> bool {
        matches!(self, CacheRead::Miss)
    }
}

#[derive(Error, Debug)]
pub enum CacheError {
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

    #[error("Cache block is corrupted")]
    CorruptedBlock,

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Asset error: {0}")]
    AssetError(#[from] crate::errors::AssetError),
}
